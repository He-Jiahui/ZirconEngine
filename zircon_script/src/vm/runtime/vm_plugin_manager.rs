use std::sync::Arc;

use crate::{HostRegistry, HotReloadCoordinator, UnavailableVmBackend};

use super::super::backend::MockVmBackend;

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
