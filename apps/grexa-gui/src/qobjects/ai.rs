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
    secret::{delete_api_key, load_api_key, store_api_key},
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
