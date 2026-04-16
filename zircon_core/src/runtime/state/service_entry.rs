use super::super::descriptors::{DependencySpec, RegistryName, ServiceFactory};
use crate::lifecycle::{LifecycleState, ServiceKind, StartupMode};
use crate::types::ServiceObject;

pub(crate) struct ServiceEntry {
    pub(crate) name: RegistryName,
    pub(crate) owner_module: String,
    pub(crate) kind: ServiceKind,
    pub(crate) startup_mode: StartupMode,
    pub(crate) dependencies: Vec<DependencySpec>,
    pub(crate) factory: ServiceFactory,
    pub(crate) lifecycle: LifecycleState,
    pub(crate) instance: Option<ServiceObject>,
}
