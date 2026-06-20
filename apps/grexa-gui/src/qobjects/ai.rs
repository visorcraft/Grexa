// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `AiController` — drives the AI chat panel.
//!
//! Owns the OpenAI-compatible chat exchange. Requests run on a worker
//! thread (the `ureq`-backed [`AiSearchClient`] is blocking) with
//! [`cxx_qt::Threading`] to hop the response back to the GUI thread.
//! The API key never round-trips through QML — it is stored in /
//! retrieved from the Secret Service via [`grexa_ai::secret`].
//!
//! **Opt-in enforcement.** Every send/test path reads
//! `SettingsStore::ai_search_enabled` and short-circuits when false.
//! The QML toggle is the source of truth; turning the panel off
//! genuinely silences the controller. The audit
//! (`docs/SECURITY.md`) explicitly required this — secret storage
//! alone is not enough.
//!
//! **Per-endpoint key scoping.** `set_api_key` stores the key with
//! the current `endpoint` qproperty as the account, so a user who
//! switches between `api.openai.com` and a corporate proxy keeps
//! distinct keys (the audit's promise — see
//! `crates/grexa-ai/src/secret.rs` module docs).

use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use grexa_ai::{
    AiConversationTurn, AiRole, AiSearchClient, AiSearchConfig, AiSearchContext, AiSearchResponse,
    EvidenceMatch, pack_evidence,
    secret::{delete_api_key, load_api_key, store_api_key},
};

use grexa_core::{
    DEFAULT_AI_SUMMARY_BUDGET_CHARS, MAX_AI_SUMMARY_BUDGET_CHARS, MIN_AI_SUMMARY_BUDGET_CHARS,
};

use super::workspace_handle::with_workspace;

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, endpoint)]
        #[qproperty(QString, model)]
        #[qproperty(bool, has_api_key)]
        #[qproperty(bool, busy)]
        #[qproperty(QString, last_response)]
        #[qproperty(QString, last_error)]
        type AiController = super::AiControllerRust;

        /// Send a user prompt. Refuses (sets `last_error`) when
        /// `ai_search_enabled` is false or no endpoint is configured.
        /// On success, runs the request off-thread and updates
        /// `last_response` + emits `response_ready`.
        #[qinvokable]
        fn send_message(self: Pin<&mut AiController>, prompt: &QString);

        /// Summarize the on-screen search results. `evidence_json` comes from
        /// `SearchController::current_evidence_json`; `total_matches` is the full
        /// match count so the summary can warn when the on-screen row cap or the
        /// excerpt budget left some matches out. Same opt-in / endpoint / busy
        /// guards as `send_message`; the summary lands in `last_response` and
        /// emits `response_ready`.
        #[qinvokable]
        fn summarize_results(
            self: Pin<&mut AiController>,
            evidence_json: &QString,
            total_matches: i32,
        );

        /// Store a new API key keyed by the *current* `endpoint`.
        /// Returns false when no endpoint is set.
        #[qinvokable]
        fn set_api_key(self: Pin<&mut AiController>, key: &QString) -> bool;

        /// Remove the stored API key for the current endpoint.
        #[qinvokable]
        fn clear_api_key(self: Pin<&mut AiController>) -> bool;

        /// Hit `/v1/models` against the configured endpoint. Same
        /// opt-in + endpoint guards as `send_message`.
        #[qinvokable]
        fn test_endpoint(self: Pin<&mut AiController>);

        /// Refresh `has_api_key` from the Secret Service against
        /// the current endpoint.
        #[qinvokable]
        fn refresh_key_state(self: Pin<&mut AiController>);

        /// Load endpoint + model from the persisted settings. Should
        /// be called at startup and whenever Settings.apply() runs.
        #[qinvokable]
        fn reload_from_settings(self: Pin<&mut AiController>);

        #[qsignal]
        fn response_ready(self: Pin<&mut AiController>);
    }

    impl cxx_qt::Threading for AiController {}
}

#[derive(Default)]
pub struct AiControllerRust {
    endpoint: QString,
    model: QString,
    has_api_key: bool,
    busy: bool,
    last_response: QString,
    last_error: QString,
}

/// Returns `Some(())` if the AI search panel is enabled in settings,
/// `None` otherwise. The audit makes this gate mandatory.
fn ai_enabled() -> bool {
    with_workspace(|w| {
        w.settings
            .load()
            .map(|s| s.ai_search_enabled)
            .unwrap_or(false)
    })
}

impl ffi::AiController {
    fn send_message(mut self: Pin<&mut Self>, prompt: &QString) {
        if !ai_enabled() {
            self.as_mut().set_last_error(QString::from(
                "AI search is disabled. Enable it in Settings → AI Search.",
            ));
            return;
        }
        let endpoint = self.as_ref().rust().endpoint.to_string();
        if endpoint.trim().is_empty() {
            self.as_mut()
                .set_last_error(QString::from("AI endpoint is not configured."));
            return;
        }

        let prompt_str = prompt.to_string();
        let model = self.as_ref().rust().model.to_string();
        let api_key = load_api_key(&endpoint).ok().flatten();

        if self.as_ref().rust().busy {
            self.as_mut()
                .set_last_error(QString::from("A request is already in progress."));
            return;
        }
        self.as_mut().set_busy(true);
        self.as_mut().set_last_error(QString::default());

        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let config = AiSearchConfig {
                endpoint,
                api_key,
                model: trim_to_option(model),
            };
            let context = AiSearchContext {
                search_path: String::new(),
                search_query: prompt_str.clone(),
                filter_suggestions: Vec::new(),
                regex_search: false,
                files_search: false,
            };
            let conversation = vec![AiConversationTurn {
                role: AiRole::User,
                content: prompt_str,
            }];
            let client = AiSearchClient::new();
            let response = client.send_chat(&config, &context, &conversation);
            let _ = thread.queue(move |pin| finish_chat(pin, response));
        });
    }

    fn summarize_results(mut self: Pin<&mut Self>, evidence_json: &QString, total_matches: i32) {
        if !ai_enabled() {
            self.as_mut().set_last_error(QString::from(
                "AI search is disabled. Enable it in Settings → AI Search.",
            ));
            return;
        }
        let endpoint = self.as_ref().rust().endpoint.to_string();
        if endpoint.trim().is_empty() {
            self.as_mut()
                .set_last_error(QString::from("AI endpoint is not configured."));
            return;
        }
        let matches: Vec<EvidenceMatch> =
            serde_json::from_str(&evidence_json.to_string()).unwrap_or_default();
        if matches.is_empty() {
            self.as_mut()
                .set_last_error(QString::from("There are no results to summarize."));
            return;
        }
        if self.as_ref().rust().busy {
            self.as_mut()
                .set_last_error(QString::from("A request is already in progress."));
            return;
        }
        let model = self.as_ref().rust().model.to_string();
        let api_key = load_api_key(&endpoint).ok().flatten();
        // Excerpt budget is user-tunable via the Settings slider; clamp the
        // stored value defensively in case settings.json was hand-edited.
        let budget = with_workspace(|w| {
            w.settings
                .load()
                .map(|s| s.ai_summary_budget_chars)
                .unwrap_or(DEFAULT_AI_SUMMARY_BUDGET_CHARS)
        })
        .clamp(MIN_AI_SUMMARY_BUDGET_CHARS, MAX_AI_SUMMARY_BUDGET_CHARS)
            as usize;
        let total_matches = total_matches.max(0) as usize;
        self.as_mut().set_busy(true);
        self.as_mut().set_last_error(QString::default());

        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let config = AiSearchConfig {
                endpoint,
                api_key,
                model: trim_to_option(model),
            };
            let context = AiSearchContext {
                search_path: String::new(),
                search_query: String::new(),
                filter_suggestions: Vec::new(),
                regex_search: false,
                files_search: false,
            };
            let conversation = vec![AiConversationTurn {
                role: AiRole::User,
                content: "Summarize these search results: the main findings grouped by theme, each with file:line citations, and note anything notable or surprising.".to_string(),
            }];
            let client = AiSearchClient::new();
            let response =
                client.send_chat_with_evidence(&config, &context, &matches, budget, &conversation);
            // Re-pack (deterministic) to read what actually fit, so the summary
            // can disclose the on-screen row cap and excerpt-budget truncation
            // rather than silently dropping matches. ponytail: pack_evidence is
            // microseconds on a ~12 KB budget — cheaper than threading the stats
            // back out of the client.
            let packed = pack_evidence(&matches, budget);
            let notice = coverage_notice(
                matches.len(),
                matches.iter().map(|m| m.lines.len()).sum(),
                packed.files_included,
                packed.lines_included,
                total_matches,
            );
            let _ = thread.queue(move |pin| {
                let mut response = response;
                if response.success && !notice.is_empty() {
                    response.message = format!("{notice}\n\n{}", response.message);
                }
                finish_chat(pin, response);
            });
        });
    }

    fn set_api_key(mut self: Pin<&mut Self>, key: &QString) -> bool {
        let endpoint = self.as_ref().rust().endpoint.to_string();
        if endpoint.trim().is_empty() {
            self.as_mut()
                .set_last_error(QString::from("Set an AI endpoint before saving an API key."));
            return false;
        }
        let ok = store_api_key(&endpoint, &key.to_string()).is_ok();
        self.as_mut().set_has_api_key(ok);
        ok
    }

    fn clear_api_key(mut self: Pin<&mut Self>) -> bool {
        let endpoint = self.as_ref().rust().endpoint.to_string();
        if endpoint.trim().is_empty() {
            return false;
        }
        let ok = delete_api_key(&endpoint).is_ok();
        if ok {
            self.as_mut().set_has_api_key(false);
        }
        ok
    }

    fn test_endpoint(mut self: Pin<&mut Self>) {
        if !ai_enabled() {
            self.as_mut().set_last_error(QString::from(
                "AI search is disabled. Enable it in Settings → AI Search.",
            ));
            return;
        }
        let endpoint = self.as_ref().rust().endpoint.to_string();
        if endpoint.trim().is_empty() {
            self.as_mut()
                .set_last_error(QString::from("AI endpoint is not configured."));
            return;
        }
        let api_key = load_api_key(&endpoint).ok().flatten();
        if self.as_ref().rust().busy {
            self.as_mut()
                .set_last_error(QString::from("A request is already in progress."));
            return;
        }
        self.as_mut().set_busy(true);
        self.as_mut().set_last_error(QString::default());

        let thread = self.qt_thread();
        std::thread::spawn(move || {
            let config = AiSearchConfig {
                endpoint,
                api_key,
                model: None,
            };
            let client = AiSearchClient::new();
            let response = client.test_endpoint(&config);
            let _ = thread.queue(move |pin| finish_test(pin, response));
        });
    }

    fn refresh_key_state(mut self: Pin<&mut Self>) {
        let endpoint = self.as_ref().rust().endpoint.to_string();
        let has_key = if endpoint.trim().is_empty() {
            false
        } else {
            load_api_key(&endpoint).ok().flatten().is_some()
        };
        self.as_mut().set_has_api_key(has_key);
    }

    fn reload_from_settings(mut self: Pin<&mut Self>) {
        let (endpoint, model) = with_workspace(|w| {
            let s = w.settings.load().unwrap_or_default();
            (s.ai_search_endpoint, s.ai_search_model)
        });
        self.as_mut().set_endpoint(QString::from(&endpoint));
        self.as_mut().set_model(QString::from(&model));
        // After updating endpoint, check whether a key is stored
        // for the new endpoint.
        let has_key = if endpoint.trim().is_empty() {
            false
        } else {
            load_api_key(&endpoint).ok().flatten().is_some()
        };
        self.as_mut().set_has_api_key(has_key);
    }
}

fn finish_chat(mut pin: Pin<&mut ffi::AiController>, response: AiSearchResponse) {
    pin.as_mut().set_busy(false);
    if response.success {
        pin.as_mut()
            .set_last_response(QString::from(&response.message));
        pin.as_mut().response_ready();
    } else {
        pin.as_mut()
            .set_last_error(QString::from(&response.error_message));
    }
}

fn finish_test(mut pin: Pin<&mut ffi::AiController>, response: AiSearchResponse) {
    pin.as_mut().set_busy(false);
    if response.success {
        pin.as_mut()
            .set_last_response(QString::from(&response.message));
    } else {
        pin.as_mut()
            .set_last_error(QString::from(&response.error_message));
    }
}

fn trim_to_option(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// One-line coverage notice prepended to a "Summarize results" answer, or `""`
/// when every matched line made it into the prompt. Surfaces both truncation
/// points: the on-screen row cap (`total_matches` vs the `evidence_lines`
/// actually handed to the model) and the excerpt budget (`evidence_lines` vs
/// the `covered_lines` that fit). Without it, a 4000-match search silently gets
/// a summary of whatever fit and the user is never told the rest was dropped.
fn coverage_notice(
    evidence_files: usize,
    evidence_lines: usize,
    covered_files: usize,
    covered_lines: usize,
    total_matches: usize,
) -> String {
    let rows_capped = total_matches > evidence_lines;
    let budget_truncated = covered_lines < evidence_lines;
    if !rows_capped && !budget_truncated {
        return String::new();
    }
    if rows_capped {
        format!(
            "_Heads up: your search has {total_matches} matches; this summary considers only the first {evidence_lines}, and {covered_lines} of those fit the excerpt budget. Raise the budget in Settings → AI Search, or narrow the search._"
        )
    } else {
        format!(
            "_Heads up: only {covered_files} of {evidence_files} matched files ({covered_lines}/{evidence_lines} lines) fit the excerpt budget. Raise it in Settings → AI Search to include the rest._"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::coverage_notice;

    #[test]
    fn no_notice_when_everything_fits() {
        assert!(coverage_notice(3, 10, 3, 10, 10).is_empty());
    }

    #[test]
    fn flags_budget_truncation() {
        let n = coverage_notice(5, 20, 2, 8, 20);
        assert!(n.contains("excerpt budget"), "{n}");
        assert!(n.contains("2 of 5"), "{n}");
    }

    #[test]
    fn flags_row_cap() {
        let n = coverage_notice(10, 400, 10, 400, 1500);
        assert!(n.contains("1500 matches"), "{n}");
        assert!(n.contains("first 400"), "{n}");
    }
}
