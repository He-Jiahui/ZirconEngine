use std::sync::Arc;

use super::super::descriptors::{PluginFactory, RegistryName, ServiceFactory};
use crate::core::lifecycle::{LifecycleState, StartupMode};
use crate::core::types::ServiceObject;

#[derive(Clone)]
pub(crate) enum ServiceEntryFactory {
    Service(ServiceFactory),
    Plugin(PluginFactory),
}

pub(crate) struct ServiceEntry {
    pub(crate) startup_mode: StartupMode,
    // Dependencies are immutable after registration; sharing the canonical name
    // slice keeps resolution from rebuilding a Vec while holding the service lock.
    pub(crate) dependencies: Arc<[RegistryName]>,
    pub(crate) factory: ServiceEntryFactory,
    pub(crate) lifecycle: LifecycleState,
    pub(crate) instance: Option<ServiceObject>,
}
