//! GUI controllers — the Rust-side state that the QML shell binds to.
//!
//! Each controller is a plain struct + free functions. The full QML
//! integration (signals, models) lands in the cxx-qt iteration; this
//! module is the contract the QML side targets and the place where the
//! sidecar version exchanges JSON.

use std::rc::Rc;

use grexa_ai::AiSearchClient;
use grexa_containers::SystemCommandRunner;
use grexa_core::{AppPaths, CancelToken, SettingsStore};
use grexa_i18n::{Bundle, Locale};

/// The bundle of singletons the GUI relies on. The QML shell holds a
/// single `Controllers` instance and routes user events through it.
/// Single-threaded by design — Qt's event loop is single-threaded and the
/// `Bundle` type isn't `Send`/`Sync`. Fields are exposed so individual
/// QML pages can borrow only what they need; the `dead_code` allow is
/// because the full QML bindings land in a follow-up Phase 4 PR.
#[allow(dead_code)]
pub struct Controllers {
    pub paths: AppPaths,
    pub settings: SettingsStore,
    pub bundle: Rc<Bundle>,
    pub cancel: CancelToken,
    pub ai: AiSearchClient,
    pub command_runner: SystemCommandRunner,
}

#[allow(dead_code)]
impl Controllers {
    pub fn new() -> anyhow::Result<Self> {
        let paths = AppPaths::from_env();
        let settings = SettingsStore::new(&paths);
        let loaded = settings.load().unwrap_or_default();
        let bundle = Rc::new(
            Bundle::for_locale(Locale::from_tag(&loaded.ui_language))
                .or_else(|_| Bundle::for_locale(Locale::English))
                .map_err(|err| anyhow::anyhow!("locale bundle failed: {err}"))?,
        );
        Ok(Self {
            paths,
            settings,
            bundle,
            cancel: CancelToken::new(),
            ai: AiSearchClient::new(),
            command_runner: SystemCommandRunner,
        })
    }

    /// Convenience accessor for the GUI translation surface.
    #[allow(dead_code)]
    pub fn t(&self, key: &str) -> String {
        self.bundle
            .t(key)
            .unwrap_or_else(|_| format!("[missing:{key}]"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controllers_bootstrap_with_default_settings() {
        let controllers = Controllers::new().expect("bootstrap");
        // The fallback English catalog always has app-name; assert end-to-end
        // wiring from settings → locale → bundle.
        assert_eq!(controllers.t("app-name"), "Grexa");
        assert!(!controllers.cancel.is_cancelled());
    }

    #[test]
    fn missing_key_returns_placeholder() {
        let controllers = Controllers::new().expect("bootstrap");
        assert!(controllers.t("definitely-not-present").contains("missing"));
    }
}
