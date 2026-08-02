//! Resource foundation layer: locators, ids, typed handles, registry, and runtime state.

mod data;
mod error;
mod event_stream;
pub mod io;
mod lease;
mod manager;
mod readiness_generation;
mod registry;
mod runtime;
mod snapshot;

pub use data::ResourceData;
pub use error::{ResourceRegistryError, ResourceResult};
pub(crate) use event_stream::approximate_event_bytes;
pub use event_stream::{
    ResourceEventGap, ResourceEventReceiver, ResourceEventRecvError, ResourceEventRecvTimeoutError,
    ResourceEventStreamDiagnostics, ResourceEventTryRecvError,
};
pub use io::{ResourceIo, ResourceIoError};
pub use lease::ResourceLease;
pub use manager::{ResourceManager, ResourceRegistryReadGuard};
pub(crate) use readiness_generation::ResourceReadinessRow;
pub use readiness_generation::{
    ResourceReadinessGeneration, ResourceReadinessGenerationDiagnostics, ResourceReadinessState,
};
pub use registry::ResourceRegistry;
pub use runtime::{Resource, ResourceRuntimeInfo, RuntimeResourceState};
pub use snapshot::ResourceSnapshot;
pub use zircon_runtime_interface::resource::*;

#[cfg(test)]
mod tests;
