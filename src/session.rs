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

/// The Host-authorized release boundary for an otherwise unchanged close intent.
///
/// This is an explicit protocol distinction, not a timing workaround. A plugin
/// cannot infer application termination from a native close callback, invisibility,
/// a timeout, or the presence of an OS-owned reference. The Host alone authorizes
/// [`Self::ApplicationTermination`] after committing an actual pending quit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluginCloseScope {
    /// Require complete native weak-graph release before any ordinary successor.
    ///
    /// All observed plugin nodes, including the closed native window, must be
    /// deallocated. Canonical documents and every session-owned resource must also
    /// be released; a detached empty window is insufficient for this scope.
    ForegroundTransition,
    /// End the process after payload release and a validated terminal window receipt.
    ///
    /// Canonical/document/session payload, callbacks, files, mappings and native
    /// content must be released. A closed, detached, empty native window may remain
    /// leased solely by the OS until process exit. That receipt is explicitly not
    /// native deallocation and cannot authorize a successor or continued residency.
    ApplicationTermination,
}

/// One explicit close intent and its Host-authorized release scope.
///
/// Save/flush succeeds before the initial request is accepted. Repeated requests
/// preserve `intent`; the only permitted scope change is
/// [`PluginCloseScope::ForegroundTransition`] to
/// [`PluginCloseScope::ApplicationTermination`] after actual pending quit. The
/// plugin asks [`PluginHostLifecycle::begin_close`] to validate that promotion
/// before replacing its accepted copy. Rejection preserves the previous request.
/// A terminal request cannot be downgraded or reused for an ordinary successor.
///
/// The SDK carries this contract but does not implement or verify a Host state
/// machine. Both participants validate the same accepted request and once-only
/// completion; public fields do not themselves confer termination authority.
///
/// ```
/// use omni_plugin_sdk::{PluginCloseIntent, PluginCloseRequest, PluginCloseScope};
///
/// // An ordinary dismissal declares both independent pieces of lifecycle policy.
/// let request = PluginCloseRequest {
///     intent: PluginCloseIntent::DismissAndResume,
///     scope: PluginCloseScope::ForegroundTransition,
/// };
/// assert_eq!(request.scope, PluginCloseScope::ForegroundTransition);
/// ```
///
/// ```compile_fail
/// use omni_plugin_sdk::{PluginCloseIntent, PluginCloseRequest};
///
/// // Protocol 2 has no inferred/default release scope for an old intent-only call.
/// let request = PluginCloseRequest { intent: PluginCloseIntent::DismissAndResume };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginCloseRequest {
    /// The original dismissal/resume or Launcher successor intent, unchanged during quit.
    pub intent: PluginCloseIntent,
    /// The explicit release contract authorized by the Host's current transaction.
    pub scope: PluginCloseScope,
}

/// Host-owned callbacks for one main-thread foreground or termination transaction.
///
/// Implementations own the state machine: the SDK does not schedule callbacks or
/// infer native destruction. A native close request, `windowWillClose:`, the
/// project's once-only teardown barrier, and actual deallocation are distinct.
/// Termination has its own explicit receipt and never weakens ordinary teardown.
pub trait PluginHostLifecycle {
    /// Commit a successfully prepared surface before it becomes user-visible.
    ///
    /// # Errors
    ///
    /// Reject conflicting creation state; the caller retains cleanup ownership.
    fn commit_creation(&self) -> Result<(), PluginError>;

    /// Validate and commit an initial request, identical repeat, or allowed promotion.
    ///
    /// Initial close follows successful fallible save/flush. An accepted request's
    /// intent never changes. A repeated identical request is idempotent; the sole
    /// scope promotion is foreground to termination after the Host has committed
    /// actual pending quit. No timeout or native reference can authorize promotion.
    ///
    /// # Errors
    ///
    /// Reject a conflicting intent, downgrade, unauthorized terminal scope, or
    /// stale transaction without replacing the prior accepted request/resources.
    fn begin_close(&self, request: PluginCloseRequest) -> Result<(), PluginError>;

    /// Notify the Host once after the accepted request's exact release boundary.
    ///
    /// The native owner stops callbacks/access, invalidates mounted borrows and
    /// releases canonical/session payload before completion. Foreground scope
    /// requires actual complete weak-graph release. Terminal scope requires the
    /// same payload release and a validated closed/detached/empty window receipt;
    /// an OS-only window lease may remain until process exit and is not called
    /// deallocation. The Host validates the exact accepted request, ignores stale
    /// or duplicate completion, and consumes its barrier only once.
    ///
    /// The Host releases its session after outstanding Rust borrows/callback
    /// frames have returned. Foreground completion may then create its ordinary
    /// successor; terminal completion may only advance the already pending quit.
    /// Neither receipt can be silently reclassified as the other scope. Cleanup
    /// remains bounded/non-panicking and presents no save/error dialogs here.
    fn finish_close(&self, request: PluginCloseRequest);
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
/// not in `Drop`. Canonical documents, histories, file descriptors, mappings, callbacks
/// and other plugin payload end with this owner. An explicitly validated terminal
/// OS window lease cannot retain that payload or become a hidden resident session.
/// Native destruction and Host borrowing still require integration validation;
/// implementing this trait does not enforce a lifecycle automatically.
pub trait PluginSession {
    /// Return the current borrowed native view, or `None` for a viewless session.
    fn native_view(&self) -> Option<PluginNativeView> {
        // Viewless sessions remain valid without manufacturing an unusable native pointer.
        None
    }

    /// Request save/close using the Host's explicit intent and release scope.
    ///
    /// Save/flush failure preserves the active surface and unsaved data before
    /// calling `begin_close`. After acceptance, identical repeats do not save or
    /// close twice; an authorized scope promotion preserves the original intent
    /// and first asks the lifecycle owner to validate the promoted request.
    /// Native close itself is not completion; finish with the exact accepted
    /// request only after its scope-specific release boundary is satisfied.
    ///
    /// ```compile_fail
    /// use omni_plugin_sdk::{PluginCloseIntent, PluginSession};
    ///
    /// // An old protocol intent cannot silently acquire an ordinary or terminal scope.
    /// fn close_old(session: &mut dyn PluginSession) {
    ///     let _ = session.request_close(PluginCloseIntent::DismissAndResume);
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// The default returns `close_unsupported`. Implementations return a stable
    /// failure while preserving unsaved data and the current usable surface.
    fn request_close(&mut self, _request: PluginCloseRequest) -> Result<(), PluginError> {
        // An unsupported session refuses both scopes explicitly; it never invents release authority.
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
