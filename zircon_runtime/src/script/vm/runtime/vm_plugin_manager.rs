use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, Weak};

use crate::core::{CoreRuntime, PluginContext};

use super::super::backend::{BuiltinVmBackendFamily, VmBackendFamily, VmBackendRegistry, VmError};
use super::super::handles::PluginSlotId;
use super::super::host::{
    HostRegistry, VmPluginHostContext, VmPluginSlotLifecycle, VM_PLUGIN_RUNTIME_NAME,
};
use super::super::plugin::{
    discover_vm_plugin_packages, DiscoveredVmPluginPackage, VmPluginPackage, VmPluginPackageSource,
};
use super::hot_reload_coordinator::HotReloadCoordinator;
use super::vm_plugin_slot_record::VmPluginSlotRecord;

const DEFAULT_BACKEND_SELECTOR: &str = "builtin:unavailable";

#[derive(Debug)]
pub struct VmPluginManager {
    self_ref: Weak<VmPluginManager>,
    plugin_context: PluginContext,
    host_registry: HostRegistry,
    coordinator: HotReloadCoordinator,
    backends: VmBackendRegistry,
    selected_backend: RwLock<String>,
}

#[derive(Debug)]
struct ManagerSlotLifecycle {
    manager: Weak<VmPluginManager>,
}

impl ManagerSlotLifecycle {
    fn new(manager: Weak<VmPluginManager>) -> Self {
        Self { manager }
    }

    fn upgrade(&self) -> Result<Arc<VmPluginManager>, VmError> {
        self.manager.upgrade().ok_or_else(|| {
            VmError::Operation("vm plugin manager lifecycle facade is no longer available".into())
        })
    }
}

impl VmPluginSlotLifecycle for ManagerSlotLifecycle {
    fn load_package(
        &self,
        backend_selector: &str,
        package: VmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        self.upgrade()?
            .load_package_with_backend(backend_selector, package)
    }

    fn hot_reload_slot(&self, slot: PluginSlotId, package: VmPluginPackage) -> Result<(), VmError> {
        self.upgrade()?.hot_reload_slot(slot, package)
    }

    fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError> {
        self.upgrade()?.unload_slot(slot)
    }

    fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        self.upgrade()?.slot(slot)
    }

    fn list_slots(&self) -> Vec<VmPluginSlotRecord> {
        self.manager
            .upgrade()
            .map(|manager| manager.list_slots())
            .unwrap_or_default()
    }
}

impl VmPluginManager {
    pub fn unavailable() -> Arc<Self> {
        Self::unavailable_with_host(HostRegistry::default())
    }

    pub fn mock() -> Arc<Self> {
        Self::mock_with_host(HostRegistry::default())
    }

    pub fn unavailable_with_host(host: HostRegistry) -> Arc<Self> {
        Self::with_builtin_backends(host)
    }

    pub fn mock_with_host(host: HostRegistry) -> Arc<Self> {
        let manager = Self::with_builtin_backends(host);
        manager.select_default_backend("builtin:mock").unwrap();
        manager
    }

    pub fn with_builtin_backends(host: HostRegistry) -> Arc<Self> {
        Self::with_plugin_context(Self::detached_plugin_context(), host)
    }

    pub fn with_plugin_context(plugin_context: PluginContext, host: HostRegistry) -> Arc<Self> {
        let manager = Arc::new_cyclic(|weak| Self {
            self_ref: weak.clone(),
            plugin_context,
            host_registry: host,
            coordinator: HotReloadCoordinator::new(),
            backends: VmBackendRegistry::new(),
            selected_backend: RwLock::new(DEFAULT_BACKEND_SELECTOR.to_string()),
        });
        manager.register_family(Arc::new(BuiltinVmBackendFamily));
        manager
    }

    pub fn register_family(&self, family: Arc<dyn VmBackendFamily>) -> String {
        self.backends.register_family(family)
    }

    pub fn backend_names(&self) -> Vec<String> {
        self.backends.names()
    }

    pub fn selected_backend_name(&self) -> String {
        self.selected_backend.read().unwrap().clone()
    }

    pub fn select_default_backend(&self, backend_name: &str) -> Result<(), VmError> {
        self.backends.resolve(backend_name)?;
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
        let host =
            self.build_host_context(backend_name, &package, VmPluginPackageSource::default());
        self.coordinator
            .load_package(backend_name, backend.as_ref(), package, &host)
    }

    pub fn load_discovered_package(
        &self,
        package: &DiscoveredVmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        let backend = self.backends.resolve(&package.backend_name)?;
        let host = self.build_host_context(
            &package.backend_name,
            &package.package,
            package.source.clone(),
        );
        self.coordinator.load_package(
            &package.backend_name,
            backend.as_ref(),
            package.package.clone(),
            &host,
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
        let host =
            self.build_host_context(backend_name, &package, VmPluginPackageSource::default());
        self.coordinator
            .hot_reload(slot, backend_name, backend.as_ref(), package, &host)
    }

    pub fn hot_reload_discovered_slot(
        &self,
        slot: PluginSlotId,
        package: &DiscoveredVmPluginPackage,
    ) -> Result<(), VmError> {
        let backend = self.backends.resolve(&package.backend_name)?;
        let host = self.build_host_context(
            &package.backend_name,
            &package.package,
            package.source.clone(),
        );
        self.coordinator.hot_reload(
            slot,
            &package.backend_name,
            backend.as_ref(),
            package.package.clone(),
            &host,
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
        self.host_registry.clone()
    }

    pub fn base_plugin_context(&self) -> &PluginContext {
        &self.plugin_context
    }

    fn detached_plugin_context() -> PluginContext {
        let runtime = CoreRuntime::new();
        PluginContext {
            plugin_name: VM_PLUGIN_RUNTIME_NAME.to_string(),
            core: runtime.handle().downgrade(),
            package_root: None,
            source_root: None,
            data_root: None,
        }
    }

    fn build_host_context(
        &self,
        backend_selector: &str,
        package: &VmPluginPackage,
        source: VmPluginPackageSource,
    ) -> VmPluginHostContext {
        let (package_root, source_root, data_root) = derive_plugin_roots(&source);
        let mut plugin = self.plugin_context.clone();
        plugin.package_root = package_root;
        plugin.source_root = source_root;
        plugin.data_root = data_root;

        VmPluginHostContext {
            plugin,
            capabilities: package.manifest.capabilities.clone(),
            backend_selector: backend_selector.to_string(),
            package_source: source,
            host_registry: self.host_registry.clone(),
            slot_lifecycle: Arc::new(ManagerSlotLifecycle::new(self.self_ref.clone())),
        }
    }
}

fn derive_plugin_roots(
    source: &VmPluginPackageSource,
) -> (Option<PathBuf>, Option<PathBuf>, Option<PathBuf>) {
    let package_root = source.package_root.clone().or_else(|| {
        source
            .manifest_path
            .as_ref()
            .and_then(|path| path.parent().map(Path::to_path_buf))
    });
    let source_root = source.manifest_path.as_ref().and_then(|path| {
        path.parent()
            .map(Path::to_path_buf)
            .or_else(|| package_root.clone())
    });
    let data_root = package_root.as_ref().map(|root| root.join("data"));
    (package_root, source_root, data_root)
}
