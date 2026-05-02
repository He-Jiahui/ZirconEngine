//! Resource foundation layer: locators, ids, typed handles, registry, and runtime state.

mod data;
mod io;
mod lease;
mod manager;
mod registry;
mod runtime;

pub use data::ResourceData;
pub use io::{ResourceIo, ResourceIoError};
pub use lease::ResourceLease;
pub use manager::ResourceManager;
pub use registry::ResourceRegistry;
pub use runtime::{Resource, ResourceRuntimeInfo, RuntimeResourceState};
pub use zircon_runtime_interface::resource::*;

#[cfg(test)]
mod tests;
