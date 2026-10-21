//! Native, statically linked plugin contracts shared by Omni Bar and its plugins.
//!
//! The SDK defines immutable descriptors, borrowed invocation data, minimal host
//! capabilities, and foreground-session ownership. It does not discover packages,
//! load libraries, create processes, scan files, or retain plugin resources.
//! Host and plugin must use one Cargo identity for this SDK and exactly the same
//! [`PLUGIN_PROTOCOL_VERSION`]. Registration and activation are ordinary Rust
//! calls in one application, not a stable dynamic-library or C ABI.
//!
//! Private modules separate contract responsibilities; root re-exports expose
//! one protocol-3 API. Close scope states the required project-resource release
//! boundary without creating a second lifecycle implementation inside the SDK.
//!
//! Protocol values carry metadata; registration owns synchronous dispatch; host
//! capabilities expose explicit external operations; sessions describe the one
//! foreground owner. Resource receipts remain the responsibility of the real
//! Host and native plugin implementations rather than portable SDK models.

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
    PluginActivation, PluginCloseIntent, PluginCloseRequest, PluginCloseScope, PluginHostLifecycle,
    PluginNativeView, PluginSession,
};
