use super::super::descriptors::{DependencySpec, PluginFactory, RegistryName, ServiceFactory};
use crate::core::lifecycle::{LifecycleState, ServiceKind, StartupMode};
use crate::core::types::ServiceObject;

#[derive(Clone)]
pub(crate) enum ServiceEntryFactory {
    Service(ServiceFactory),
    Plugin(PluginFactory),
}

pub(crate) struct ServiceEntry {
    pub(crate) name: RegistryName,
    pub(crate) owner_module: String,
    pub(crate) kind: ServiceKind,
    pub(crate) startup_mode: StartupMode,
    pub(crate) dependencies: Vec<DependencySpec>,
    pub(crate) factory: ServiceEntryFactory,
    pub(crate) lifecycle: LifecycleState,
    pub(crate) instance: Option<ServiceObject>,
}
