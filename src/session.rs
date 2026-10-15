//! Foreground resource ownership and host/plugin teardown callbacks.

use std::ffi::c_void;
use std::fmt;
use std::ptr::NonNull;

use crate::PluginError;

/// The user's intended successor after the current foreground session closes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginCloseIntent {
    /// Destroy the session and stay hidden; a later shortcut recreates its target.
    DismissAndResume,
    /// Destroy the session before the host creates and focuses Launcher.
    ExitToLauncher,
}

/// Host-owned callbacks for one main-thread foreground transition.
///
/// Implementations own the state machine: the SDK does not schedule callbacks or
/// infer native destruction. A native close request, `windowWillClose:`, the
/// project's once-only teardown barrier, and actual deallocation are distinct.
pub trait PluginHostLifecycle {
    /// Commit a successfully prepared surface before it becomes user-visible.
    ///
    /// # Errors
    ///
    /// Reject conflicting creation state; the caller retains cleanup ownership.
    fn commit_creation(&self) -> Result<(), PluginError>;

    /// Commit one close intent after fallible save/flush work succeeds.
    ///
    /// # Errors
    ///
    /// Reject conflicting transitions without discarding the active session.
    fn begin_close(&self, intent: PluginCloseIntent) -> Result<(), PluginError>;

    /// Notify the host after the native callback stack and resource-release barrier.
    ///
    /// The native owner must stop callbacks/access before releasing its graph.
    /// The host invalidates borrowed view state, consumes the barrier once, and
    /// drops its session before creating any successor. Implementations must
    /// supply bounded, non-panicking cleanup without native save/error dialogs.
    fn finish_close(&self, intent: PluginCloseIntent);
}

/// A non-owning, main-thread-only opaque handle to a plugin-owned AppKit `NSView`.
///
/// Copying this handle does not retain the native object or extend its lifetime.
/// The plugin's native owner keeps the view alive for every host access; the host
/// stops using and invalidates its borrow at the agreed teardown boundary.
/// This handle intentionally has no `Send` or `Sync` implementation.
///
/// ```compile_fail
/// use omni_plugin_sdk::PluginNativeView;
///
/// // A compile-time predicate cannot make an AppKit borrow thread-transferable.
/// fn requires_send<T: Send>() {}
/// requires_send::<PluginNativeView>();
/// ```
///
/// ```compile_fail
/// use omni_plugin_sdk::PluginNativeView;
///
/// // Shared cross-thread access is forbidden for the same native lifetime reason.
/// fn requires_sync<T: Sync>() {}
/// requires_sync::<PluginNativeView>();
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginNativeView(NonNull<c_void>);

impl PluginNativeView {
    /// Construct an opaque borrow, rejecting a null native view pointer.
    ///
    /// # Safety
    ///
    /// A non-null `view` must identify a valid initialized AppKit `NSView` on the
    /// main thread. The plugin's native owner must keep its strong reference for
    /// all host uses and coordinate invalidation before that reference is released.
    /// The caller must not transfer this handle to another thread or treat it as
    /// an ownership claim. A null pointer is permitted and produces `None`.
    #[must_use]
    pub unsafe fn from_raw(view: *mut c_void) -> Option<Self> {
        // Preserve only non-null pointer identity; never retain, dereference, or release it.
        NonNull::new(view).map(Self)
    }

    /// Expose the borrowed pointer without transferring native ownership.
    ///
    /// Dereferencing it remains subject to the original lifetime/main-thread
    /// obligations; this accessor cannot prove that a copied handle is still live.
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        // Return the same borrow so callers cannot mistake access for a retain operation.
        self.0.as_ptr()
    }
}

/// Resources created by one explicit activation and owned by its foreground host.
///
/// A concrete session owns its active resource graph and performs bounded cleanup
/// when dropped. Fallible save/flush work belongs in explicit close operations,
/// not in `Drop`. Native destruction and host borrowing still require integration
/// validation; implementing this trait does not enforce a lifecycle automatically.
pub trait PluginSession {
    /// Return the current borrowed native view, or `None` for a viewless session.
    fn native_view(&self) -> Option<PluginNativeView> {
        // Viewless sessions remain valid without manufacturing an unusable native pointer.
        None
    }

    /// Request save/close using the host's selected lifecycle intent.
    ///
    /// # Errors
    ///
    /// The default returns `close_unsupported`. Implementations return a stable
    /// failure while preserving unsaved data and the current usable surface.
    fn request_close(&mut self, _intent: PluginCloseIntent) -> Result<(), PluginError> {
        // Preserve the original refusal contract until a concrete session owns close handling.
        Err(PluginError::new(
            "close_unsupported",
            "插件 session 不支持关闭请求",
        ))
    }
}

/// The synchronous result of activating a validated static registration.
pub enum PluginActivation {
    /// Work completed inline without transferring foreground resource ownership.
    Completed,
    /// Transfer the active session to the host until its teardown barrier completes.
    Session(Box<dyn PluginSession>),
}

impl fmt::Debug for PluginActivation {
    /// Format the outcome without inspecting native/session-private state.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Keep diagnostics independent from potentially sensitive plugin resource graphs.
        match self {
            // Completed work has no session owner to describe.
            Self::Completed => formatter.write_str("Completed"),
            // Session internals remain private even when a caller logs the activation result.
            Self::Session(_) => formatter.write_str("Session(..)"),
        }
    }
}
