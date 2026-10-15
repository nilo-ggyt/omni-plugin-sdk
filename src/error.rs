//! Static errors crossing the synchronous host/plugin boundary.

use thiserror::Error;

/// A bounded protocol failure with a stable machine-readable code.
///
/// Static strings prevent this boundary from accidentally retaining document
/// bodies, paths, or an arbitrary native error graph. Display exposes the supplied
/// message, while callers use [`Self::code`] for programmatic handling.
///
/// ```
/// use omni_plugin_sdk::PluginError;
///
/// // Keep a stable code independent from the human-facing explanation.
/// let error = PluginError::new("save_failed", "The document could not be saved");
/// assert_eq!(error.code(), "save_failed");
/// assert_eq!(error.to_string(), "The document could not be saved");
/// ```
#[derive(Debug, Error, Eq, PartialEq)]
#[error("{message}")]
pub struct PluginError {
    code: &'static str,
    message: &'static str,
}

impl PluginError {
    /// Construct a failure without allocating or acquiring external resources.
    #[must_use]
    pub const fn new(code: &'static str, message: &'static str) -> Self {
        // Store borrowed static metadata so error propagation cannot extend a session.
        Self { code, message }
    }

    /// Return the stable code without exposing or copying the display message.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        // Error classification remains independent from display language.
        self.code
    }
}
