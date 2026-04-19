use std::path::Path;
use std::sync::{Arc, RwLock};

use super::super::backend::{
    MockVmBackend, UnavailableVmBackend, VmBackend, VmBackendRegistry, VmError,
};
use super::super::handles::PluginSlotId;
use super::super::host::HostRegistry;
use super::super::plugin::{
    discover_vm_plugin_packages, DiscoveredVmPluginPackage, VmPluginPackage,
};
use super::hot_reload_coordinator::HotReloadCoordinator;
use super::vm_plugin_slot_record::VmPluginSlotRecord;

#[derive(Debug)]
pub struct VmPluginManager {
    coordinator: HotReloadCoordinator,
    backends: VmBackendRegistry,
    selected_backend: RwLock<String>,
}

impl VmPluginManager {
    pub fn unavailable() -> Self {
        Self::unavailable_with_host(HostRegistry::default())
    }

    pub fn mock() -> Self {
        Self::mock_with_host(HostRegistry::default())
    }

    pub fn unavailable_with_host(host: HostRegistry) -> Self {
        let registry = VmBackendRegistry::new();
        let unavailable = Arc::new(UnavailableVmBackend);
        let unavailable_name = unavailable.backend_name().to_string();
        registry.register(unavailable);
        Self {
            coordinator: HotReloadCoordinator::new(host),
            backends: registry,
            selected_backend: RwLock::new(unavailable_name),
        }
    }

    pub fn mock_with_host(host: HostRegistry) -> Self {
        let manager = Self::unavailable_with_host(host);
        let mock_name = manager.register_backend(Arc::new(MockVmBackend));
        manager.select_default_backend(&mock_name).unwrap();
        manager
    }

    pub fn with_backend(backend: Arc<dyn VmBackend>, host: HostRegistry) -> Self {
        let backend_name = backend.backend_name().to_string();
        let registry = VmBackendRegistry::new();
        registry.register_named(backend_name.clone(), backend);
        Self {
            coordinator: HotReloadCoordinator::new(host),
            backends: registry,
            selected_backend: RwLock::new(backend_name),
        }
    }

    pub fn with_builtin_backends(host: HostRegistry) -> Self {
        let manager = Self::unavailable_with_host(host);
        manager.register_backend(Arc::new(MockVmBackend));
        manager
    }

    pub fn register_backend(&self, backend: Arc<dyn VmBackend>) -> String {
        let name = backend.backend_name().to_string();
        self.backends.register_named(name.clone(), backend);
        name
    }

    pub fn backend_names(&self) -> Vec<String> {
        self.backends.names()
    }

    pub fn selected_backend_name(&self) -> String {
        self.selected_backend.read().unwrap().clone()
    }

    pub fn select_default_backend(&self, backend_name: &str) -> Result<(), VmError> {
        if !self.backends.contains(backend_name) {
            return Err(VmError::UnknownBackend(backend_name.to_string()));
        }
        *self.selected_backend.write().unwrap() = backend_name.to_string();
        Ok(())
    }

    pub fn discover_packages(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredVmPluginPackage>, VmError> {
        discover_vm_plugin_packages(root)
    }

    pub fn load_package(&self, package: VmPluginPackage) -> Result<PluginSlotId, VmError> {
        let backend_name = self.selected_backend_name();
        self.load_package_with_backend(&backend_name, package)
    }

    pub fn load_package_with_backend(
        &self,
        backend_name: &str,
        package: VmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        let backend = self.backends.resolve(backend_name)?;
        self.coordinator
            .load_package(backend_name, backend.as_ref(), package)
    }

    pub fn load_discovered_package(
        &self,
        package: &DiscoveredVmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        let backend = self.backends.resolve(&package.backend_name)?;
        self.coordinator.load_package_with_source(
            &package.backend_name,
            backend.as_ref(),
            package.package.clone(),
            package.source.clone(),
        )
    }

    pub fn hot_reload_slot(
        &self,
        slot: PluginSlotId,
        package: VmPluginPackage,
    ) -> Result<(), VmError> {
        let backend_name = self.slot(slot)?.backend_name;
        self.hot_reload_slot_with_backend(slot, &backend_name, package)
    }

    pub fn hot_reload_slot_with_backend(
        &self,
        slot: PluginSlotId,
        backend_name: &str,
        package: VmPluginPackage,
    ) -> Result<(), VmError> {
        let backend = self.backends.resolve(backend_name)?;
        self.coordinator
            .hot_reload(slot, backend_name, backend.as_ref(), package)
    }

    pub fn hot_reload_discovered_slot(
        &self,
        slot: PluginSlotId,
        package: &DiscoveredVmPluginPackage,
    ) -> Result<(), VmError> {
        let backend = self.backends.resolve(&package.backend_name)?;
        self.coordinator.hot_reload_with_source(
            slot,
            &package.backend_name,
            backend.as_ref(),
            package.package.clone(),
            package.source.clone(),
        )
    }

    pub fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError> {
        self.coordinator.unload_slot(slot).map(|_| ())
    }

    pub fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        self.coordinator.slot(slot)
    }

    pub fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        self.coordinator.list_slots()
    }

    pub fn coordinator(&self) -> &HotReloadCoordinator {
        &self.coordinator
    }

    pub fn host_registry(&self) -> HostRegistry {
        self.coordinator.host()
    }
}
