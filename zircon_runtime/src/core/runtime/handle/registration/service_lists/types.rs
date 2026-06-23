use std::sync::Arc;

use super::super::super::super::descriptors::RegistryName;

pub(in crate::core::runtime::handle::registration) struct ModuleServiceLists {
    pub(in crate::core::runtime::handle::registration) service_names: Arc<[RegistryName]>,
    pub(in crate::core::runtime::handle::registration) startup_service_names: Arc<[RegistryName]>,
    pub(in crate::core::runtime::handle::registration) shutdown_service_names: Arc<[RegistryName]>,
}
