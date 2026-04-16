use std::sync::Arc;

use super::{
    host_registry::HostRegistry, hot_reload_coordinator::HotReloadCoordinator,
    mock_vm_backend::MockVmBackend, unavailable_vm_backend::UnavailableVmBackend,
};

#[derive(Debug)]
pub struct VmPluginManager {
    coordinator: HotReloadCoordinator,
}

impl VmPluginManager {
    pub fn unavailable() -> Self {
        let host = HostRegistry::default();
        Self {
            coordinator: HotReloadCoordinator::new(Arc::new(UnavailableVmBackend), host),
        }
    }

    pub fn mock() -> Self {
        let host = HostRegistry::default();
        Self {
            coordinator: HotReloadCoordinator::new(Arc::new(MockVmBackend), host),
        }
    }

    pub fn coordinator(&self) -> &HotReloadCoordinator {
        &self.coordinator
    }
}
