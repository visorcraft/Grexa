// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! `AiController` — drives the AI chat panel.
//!
//! Owns the OpenAI-compatible chat exchange. Requests run on a worker
//! thread (the `ureq`-backed [`AiSearchClient`] is blocking) with
//! [`cxx_qt::Threading`] to hop the response back to the GUI thread.
//! The API key never round trips through QML — it is stored in /
//! retrieved from the Secret Service via [`grexa_ai::secret`].

use std::pin::Pin;

use cxx_qt::{CxxQtType, Threading};
use cxx_qt_lib::QString;
use grexa_ai::{
    AiConversationTurn, AiRole, AiSearchClient, AiSearchConfig, AiSearchContext, AiSearchResponse,
    secret::{delete_api_key, load_api_key, store_api_key},
};

/// Secret-Service service id used by every key operation.
const SECRET_SCOPE: &str = "io.visorcraft.Grexa";

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

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

        /// Send a user prompt. Resolves on a worker thread; updates
        /// `last_response` or `last_error` and emits `response_ready`
        /// when the call completes.
        #[qinvokable]
        fn send_message(self: Pin<&mut AiController>, prompt: &QString);

        /// Store a new API key in the Secret Service. Returns true on
        /// success.
        #[qinvokable]
        fn set_api_key(self: Pin<&mut AiController>, key: &QString) -> bool;

        /// Remove the stored API key.
        #[qinvokable]
        fn clear_api_key(self: Pin<&mut AiController>) -> bool;

        /// Hit `/v1/models` against the configured endpoint and report
        /// success or failure on `last_response` / `last_error`.
        #[qinvokable]
        fn test_endpoint(self: Pin<&mut AiController>);

        /// Refresh `has_api_key` from the Secret Service.
        #[qinvokable]
        fn refresh_key_state(self: Pin<&mut AiController>);

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

impl ffi::AiController {
    fn send_message(mut self: Pin<&mut Self>, prompt: &QString) {
        let prompt_str = prompt.to_string();
        let endpoint = self.as_ref().rust().endpoint.to_string();
        let model = self.as_ref().rust().model.to_string();
        let api_key = load_api_key(SECRET_SCOPE).ok().flatten();

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
        let ok = store_api_key(SECRET_SCOPE, &key.to_string()).is_ok();
        self.as_mut().set_has_api_key(ok);
        ok
    }

    fn clear_api_key(mut self: Pin<&mut Self>) -> bool {
        let ok = delete_api_key(SECRET_SCOPE).is_ok();
        if ok {
            self.as_mut().set_has_api_key(false);
        }
        ok
    }

    fn test_endpoint(mut self: Pin<&mut Self>) {
        let endpoint = self.as_ref().rust().endpoint.to_string();
        let api_key = load_api_key(SECRET_SCOPE).ok().flatten();
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
        let has_key = load_api_key(SECRET_SCOPE).ok().flatten().is_some();
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
