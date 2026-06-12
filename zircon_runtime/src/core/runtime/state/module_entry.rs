use std::sync::Arc;

use crate::core::LifecycleState;

use super::super::descriptors::{ModuleDescriptor, RegistryName};

pub(crate) struct ModuleEntry {
    #[allow(dead_code)]
    pub(crate) descriptor: ModuleDescriptor,
    // Immutable after registration; this is the complete owner list for module
    // local lifecycle and diagnostics paths.
    pub(crate) service_names: Arc<[RegistryName]>,
    // Immediate services are prefiltered once so activation does not scan the
    // service table just to discover startup work.
    pub(crate) startup_service_names: Arc<[RegistryName]>,
    // Shutdown order is precomputed once from the same owner list so deactivation
    // does not rebuild or sort service names on every lifecycle transition.
    pub(crate) shutdown_service_names: Arc<[RegistryName]>,
    pub(crate) lifecycle: LifecycleState,
}
