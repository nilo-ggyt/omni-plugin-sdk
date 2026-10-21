//! Static handler registration, exact validation, and dormant resume identities.

use std::fmt;

use crate::{
    OmniBarHostContext, PLUGIN_PROTOCOL_VERSION, PluginActivation, PluginDescriptor, PluginError,
    PluginInvocation,
};

/// An ordinary synchronous Rust activation function with caller-borrowed inputs.
///
/// No task, thread, serializer, process, or loader is created by this signature.
/// The Host calls native plugins on its AppKit main thread; the SDK neither
/// moves the invocation to that thread nor retains its borrowed inputs.
pub type PluginActivationHandler = for<'a> fn(
    &mut dyn OmniBarHostContext,
    &PluginInvocation<'a>,
) -> Result<PluginActivation, PluginError>;

/// Immutable metadata and one handler linked into the application at build time.
///
/// This value is suitable for a static Host catalog and does not retain active
/// plugin resources. Equality compares descriptor metadata, preserving the
/// catalog contract rather than relying on function-pointer identity. The Host
/// still rejects duplicate plugin/command identities when assembling that catalog.
#[derive(Clone, Copy)]
pub struct PluginRegistration {
    descriptor: &'static PluginDescriptor,
    activation_handler: PluginActivationHandler,
}

impl PluginRegistration {
    /// Associate static metadata with its linked handler without calling plugin code.
    #[must_use]
    pub const fn new(
        descriptor: &'static PluginDescriptor,
        activation_handler: PluginActivationHandler,
    ) -> Self {
        // Keep the catalog immutable and dormant until an explicit activation arrives.
        Self {
            descriptor,
            activation_handler,
        }
    }

    /// Borrow discoverability metadata without allocating or activating resources.
    #[must_use]
    pub const fn descriptor(self) -> &'static PluginDescriptor {
        // Expose the existing static reference; callers never become session owners here.
        self.descriptor
    }

    /// Validate the exact protocol/command and call the handler on the current thread.
    ///
    /// # Errors
    ///
    /// Return `protocol_version_mismatch` or `command_not_found` before invoking
    /// plugin code. Once validated, propagate the handler's error unchanged; the
    /// plugin/host owners remain responsible for rollback of acquired resources.
    /// An error does not transfer a session to the Host or authorize a retry of
    /// side effects; any partially created native graph keeps its actual owner
    /// until that owner has completed failure cleanup.
    pub fn activate(
        self,
        host: &mut dyn OmniBarHostContext,
        invocation: &PluginInvocation<'_>,
    ) -> Result<PluginActivation, PluginError> {
        // Release obligations change with the protocol, so reject before any Host capability is borrowed.
        if self.descriptor.protocol_version != PLUGIN_PROTOCOL_VERSION {
            // Exact rejection prevents accidental negotiation or an old protocol fallback.
            return Err(PluginError::new(
                "protocol_version_mismatch",
                "插件协议版本与宿主不一致",
            ));
        }

        // Search labels and short codes are not command identities; only this descriptor can authorize dispatch.
        if self.descriptor.command(invocation.command_id).is_none() {
            // Preserve the caller's Host state rather than dispatching an unknown action.
            return Err(PluginError::new("command_not_found", "找不到插件命令"));
        }

        // Dispatch inline after both guards; ownership moves only through the explicit result.
        (self.activation_handler)(host, invocation)
    }
}

impl fmt::Debug for PluginRegistration {
    /// Show immutable registration metadata without formatting executable handler state.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Leave the handler opaque because its address is not a stable public identity.
        formatter
            .debug_struct("PluginRegistration")
            .field("descriptor", &self.descriptor)
            .finish_non_exhaustive()
    }
}

impl PartialEq for PluginRegistration {
    /// Compare the established descriptor identity rather than executable addresses.
    fn eq(&self, other: &Self) -> bool {
        // Host resume-state comparisons rely on matching static protocol metadata.
        self.descriptor == other.descriptor
    }
}

impl Eq for PluginRegistration {}

/// A dormant resume target that never retains an active plugin session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PluginCommandTarget {
    /// Registration from the Host's statically assembled catalog.
    pub registration: PluginRegistration,
    /// Declared command to activate when the user explicitly resumes the target.
    pub command_id: &'static str,
}
