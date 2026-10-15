//! Native, statically linked plugin contracts shared by Omni Bar and its plugins.
//!
//! The SDK defines immutable descriptors, borrowed invocation data, minimal host
//! capabilities, and foreground-session ownership. It does not discover packages,
//! load libraries, create processes, scan files, or retain plugin resources.
//! Host and plugin must use one Cargo identity for this SDK and exactly the same
//! [`PLUGIN_PROTOCOL_VERSION`]. Registration and activation are ordinary Rust
//! calls in one application, not a stable dynamic-library or C ABI.
//!
//! Private modules separate contract responsibilities while the root re-exports
//! preserve the original consumer-facing API.

#![deny(rustdoc::broken_intra_doc_links)]

mod error;
mod host;
mod protocol;
mod registration;
mod session;

pub use error::PluginError;
pub use host::OmniBarHostContext;
pub use protocol::{
    ModifierFlags, PLUGIN_PROTOCOL_VERSION, PluginCommandDescriptor, PluginDescriptor,
    PluginInvocation,
};
pub use registration::{PluginActivationHandler, PluginCommandTarget, PluginRegistration};
pub use session::{
    PluginActivation, PluginCloseIntent, PluginHostLifecycle, PluginNativeView, PluginSession,
};
