// SPDX-FileCopyrightText: 2026 VisorCraft LLC
// SPDX-License-Identifier: GPL-3.0-only

//! Secret-Service-backed API key storage.
//!
//! Grexa never persists an OpenAI-style API key to settings.json. Instead
//! the key is stored in the system keyring via the `keyring` crate, which
//! on Linux talks to `org.freedesktop.secrets` (KWallet, GNOME Keyring,
//! KeePassXC's secret-service plugin, …). The functions in this module
//! are sync, infallible-when-the-keyring-is-up, and return a structured
//! [`SecretError`] otherwise so callers can show the user an actionable
//! message rather than silently falling back to plaintext storage.
//!
//! ## Service / account layout
//!
//! The keyring entry key is fixed:
//!
//! - Service: `io.visorcraft.Grexa.ai`
//! - Account: the AI endpoint base URL (canonicalized via
//!   [`crate::normalize_endpoint_base`])
//!
//! This lets a single user keep distinct keys for `api.openai.com`,
//! `localhost:8000`, and corporate proxies without one overwriting the
//! other.

use keyring::Entry;
use thiserror::Error;

use crate::normalize_endpoint_base;

const SERVICE: &str = "io.visorcraft.Grexa.ai";

#[derive(Debug, Error)]
pub enum SecretError {
    #[error("secret backend unavailable: {0}")]
    Backend(String),
    #[error("no API key stored for {endpoint}")]
    Missing { endpoint: String },
}

impl From<keyring::Error> for SecretError {
    fn from(value: keyring::Error) -> Self {
        match value {
            keyring::Error::NoEntry => SecretError::Missing {
                endpoint: String::new(),
            },
            other => SecretError::Backend(other.to_string()),
        }
    }
}

fn entry_for(endpoint: &str) -> Result<Entry, SecretError> {
    let base = normalize_endpoint_base(endpoint);
    Entry::new(SERVICE, &base).map_err(SecretError::from)
}

/// Store an API key for `endpoint`. Replaces any existing entry. An empty
/// `api_key` deletes the entry (and is the canonical way to "log out").
pub fn store_api_key(endpoint: &str, api_key: &str) -> Result<(), SecretError> {
    let entry = entry_for(endpoint)?;
    if api_key.is_empty() {
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(SecretError::Backend(err.to_string())),
        }
    } else {
        entry
            .set_password(api_key)
            .map_err(|err| SecretError::Backend(err.to_string()))
    }
}

/// Read the API key for `endpoint`. Returns `None` when no entry exists;
/// returns `Err` for any other backend problem (e.g. keyring locked).
pub fn load_api_key(endpoint: &str) -> Result<Option<String>, SecretError> {
    let entry = entry_for(endpoint)?;
    match entry.get_password() {
        Ok(value) => Ok(Some(value)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(err) => Err(SecretError::Backend(err.to_string())),
    }
}

/// Drop a stored credential. Missing entries are treated as success so the
/// "Sign out" UI doesn't need to special-case "never logged in".
pub fn delete_api_key(endpoint: &str) -> Result<(), SecretError> {
    let entry = entry_for(endpoint)?;
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(err) => Err(SecretError::Backend(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// In a CI runner there's no D-Bus session and no live secret-service
    /// daemon, so `store_api_key` returns a `Backend` error. We pin the
    /// shape of the error rather than the round-trip behavior.
    #[test]
    fn missing_backend_surfaces_actionable_error() {
        let result = store_api_key("https://api.example.com/v1", "sk-test");
        match result {
            Ok(()) => {
                // Backend was actually available — clean up so we don't
                // pollute the user's keyring.
                let _ = delete_api_key("https://api.example.com/v1");
            }
            Err(SecretError::Backend(msg)) => {
                assert!(!msg.is_empty(), "backend error must carry a message");
            }
            Err(other) => panic!("unexpected secret error: {other:?}"),
        }
    }
}
