//! Minimal capabilities borrowed from the host during plugin activation.

use std::path::Path;
use std::rc::Rc;

use crate::{PluginError, PluginHostLifecycle, PluginNativeView};

/// Shared native capabilities available without importing Host-private modules.
///
/// The host and plugin execute in the same trusted application process. This
/// interface is a Rust ownership boundary, not a sandbox or permission broker.
/// AppKit access must run on the main thread. Implementations must end mutable
/// state borrows before native calls that can reenter application code.
pub trait OmniBarHostContext {
    /// Borrow the authoritative workspace root selected by the host.
    ///
    /// Obtaining the path does not authorize eager scanning or directory creation.
    fn workspace_root(&self) -> &Path;

    /// Borrow the application-owned root for explicitly activated session data.
    ///
    /// The Host owns root selection. Receiving this path creates no directories
    /// and does not permit a plugin to substitute a test root for normal user data.
    fn application_data_root(&self) -> &Path;

    /// Mount a borrowed native view without taking its native ownership.
    ///
    /// # Errors
    ///
    /// Return a stable failure if the host cannot accept the view. The plugin
    /// must preserve the view's owner until mounting and any failure cleanup end.
    /// The pointer is not a native retain; the Host must stop using its mounted
    /// borrow before the plugin releases the corresponding native instance.
    fn mount_native_view(&mut self, view: PluginNativeView) -> Result<(), PluginError>;

    /// Read transient plain text from the system clipboard on explicit demand.
    ///
    /// # Errors
    ///
    /// Return a stable error for unavailable native clipboard access; no text is
    /// returned when the clipboard has no compatible plain-text representation.
    /// `Some` transfers one owned text value to the caller, while `None` is a
    /// successful absence and must not be presented as an access failure.
    fn read_clipboard_text(&mut self) -> Result<Option<String>, PluginError>;

    /// Write the supplied text without creating SDK clipboard history or caches.
    ///
    /// # Errors
    ///
    /// Return a stable failure if the native clipboard rejects the operation.
    fn write_clipboard_text(&mut self, text: &str) -> Result<(), PluginError>;

    /// Share main-thread lifecycle callbacks with the newly activated session.
    ///
    /// The returned `Rc` shares the callback owner, not a hidden surface/session.
    /// Owners must prevent callback cycles and release their references on close.
    /// A callback must return before its caller drops a session still borrowed
    /// by that stack; observing a close request is not a deallocation receipt.
    fn foreground_lifecycle(&self) -> Rc<dyn PluginHostLifecycle>;
}
