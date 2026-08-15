use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard, Weak};

use crate::core::framework::script::ScriptHostValue;
use crate::core::{CoreError, CoreHandle, CoreRuntime, JobScheduler, PluginContext};

use super::super::backend::{BuiltinVmBackendFamily, VmBackendFamily, VmBackendRegistry, VmError};
use super::super::gc_bridge::{VmGcBudget, VmGcStepReport};
use super::super::handles::PluginSlotId;
use super::super::host::{
    register_builtin_host_modules, HostExportRegistry, HostRegistry, VmPluginHostContext,
    VmPluginSlotLifecycle, VM_PLUGIN_RUNTIME_NAME,
};
use super::super::host_interface::{
    VmBehaviorNodeRegistration, VmCallbackHandle, VmEditorOperationRegistration,
    VmHostInterfaceError, VmHostInterfaceRegistry, VmInterfaceCaller, VmRpcHandlerRegistration,
    VmSystemRegistration, VmSystemStage,
};
use super::super::plugin::{
    DiscoveredVmPluginPackage, VmPluginDiscoveryRequest, VmPluginDiscoveryWorker, VmPluginPackage,
    VmPluginPackageSource, VmPluginPayloadCache,
};
use super::super::reflection::{VmReflectionCatalog, VM_REFLECTION_WORLD_EXTENSION_NAME};
use super::hot_reload_coordinator::HotReloadCoordinator;
use super::vm_plugin_slot_record::VmPluginSlotRecord;

const DEFAULT_BACKEND_SELECTOR: &str = "builtin:unavailable";

#[derive(Debug)]
pub struct VmPluginManager {
    self_ref: Weak<VmPluginManager>,
    plugin_context: PluginContext,
    host_registry: HostRegistry,
    host_exports: HostExportRegistry,
    host_interfaces: VmHostInterfaceRegistry,
    reflection_catalog: VmReflectionCatalog,
    coordinator: HotReloadCoordinator,
    discovery_worker: VmPluginDiscoveryWorker,
    payload_cache: VmPluginPayloadCache,
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
        let host_exports = HostExportRegistry::new(host.clone());
        register_builtin_host_modules(&host_exports, &host)
            .expect("builtin script host modules should be valid");
        Self::with_plugin_context_and_host_exports(plugin_context, host, host_exports)
    }

    pub fn with_plugin_context_and_host_exports(
        plugin_context: PluginContext,
        host: HostRegistry,
        host_exports: HostExportRegistry,
    ) -> Arc<Self> {
        let discovery_worker = plugin_context
            .core
            .upgrade()
            .map(|core| {
                let io_pool = core.task_pools().io().clone();
                VmPluginDiscoveryWorker::with_io_pool(
                    Default::default(),
                    io_pool.clone(),
                    JobScheduler::from_pool(io_pool),
                )
            })
            .unwrap_or_default();
        let manager = Arc::new_cyclic(|weak| Self {
            self_ref: weak.clone(),
            plugin_context,
            host_registry: host,
            host_exports,
            host_interfaces: VmHostInterfaceRegistry::default(),
            reflection_catalog: VmReflectionCatalog::default(),
            coordinator: HotReloadCoordinator::new(),
            discovery_worker,
            payload_cache: VmPluginPayloadCache::default(),
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
        self.selected_backend_read().clone()
    }

    pub fn select_default_backend(&self, backend_name: &str) -> Result<(), VmError> {
        self.backends.resolve(backend_name)?;
        *self.selected_backend_write() = backend_name.to_string();
        Ok(())
    }

    pub fn discover_packages(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<Vec<DiscoveredVmPluginPackage>, VmError> {
        if self.discovery_worker.is_current_io_worker() {
            return Err(VmError::Operation(
                "synchronous plugin discovery cannot wait from its own I/O worker; use submit_package_discovery"
                    .to_string(),
            ));
        }
        self.submit_package_discovery(root)?.wait()
    }

    pub fn submit_package_discovery(
        &self,
        root: impl AsRef<Path>,
    ) -> Result<VmPluginDiscoveryRequest, VmError> {
        self.discovery_worker.submit(root.as_ref().to_path_buf())
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
        let result = self
            .coordinator
            .load_package(backend_name, backend.as_ref(), package, &host);
        self.publish_active_interfaces();
        result
    }

    pub fn load_discovered_package(
        &self,
        package: &DiscoveredVmPluginPackage,
    ) -> Result<PluginSlotId, VmError> {
        let backend = self.backends.resolve(&package.backend_name)?;
        let materialized = self.payload_cache.materialize(package)?;
        let host =
            self.build_host_context(&package.backend_name, &materialized, package.source.clone());
        let result = self.coordinator.load_package(
            &package.backend_name,
            backend.as_ref(),
            materialized,
            &host,
        );
        self.publish_active_interfaces();
        result
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
        let result =
            self.coordinator
                .hot_reload(slot, backend_name, backend.as_ref(), package, &host);
        self.publish_active_interfaces();
        result
    }

    pub fn hot_reload_discovered_slot(
        &self,
        slot: PluginSlotId,
        package: &DiscoveredVmPluginPackage,
    ) -> Result<(), VmError> {
        let backend = self.backends.resolve(&package.backend_name)?;
        let materialized = self.payload_cache.materialize(package)?;
        let host =
            self.build_host_context(&package.backend_name, &materialized, package.source.clone());
        let result = self.coordinator.hot_reload(
            slot,
            &package.backend_name,
            backend.as_ref(),
            materialized,
            &host,
        );
        self.publish_active_interfaces();
        result
    }

    pub fn unload_slot(&self, slot: PluginSlotId) -> Result<(), VmError> {
        let result = self.coordinator.unload_slot(slot).map(|_| ());
        self.publish_active_interfaces();
        result
    }

    pub fn slot(&self, slot: PluginSlotId) -> Result<VmPluginSlotRecord, VmError> {
        self.coordinator.slot(slot)
    }

    pub fn slot_for_package_name(&self, package_name: &str) -> Result<PluginSlotId, VmError> {
        let Some(matches) = self.host_interfaces.active_slots_for_package(package_name) else {
            return Err(VmError::Operation(format!(
                "vm plugin package {package_name} is not loaded"
            )));
        };
        let Some(slot) = matches.first().copied() else {
            return Err(VmError::Operation(format!(
                "vm plugin package {package_name} is not loaded"
            )));
        };
        if matches.len() > 1 {
            return Err(VmError::Operation(format!(
                "vm plugin package name {package_name} is ambiguous across active slots {:?}",
                matches.iter().map(|slot| slot.get()).collect::<Vec<_>>()
            )));
        }
        Ok(slot)
    }

    pub fn call_slot_export(
        &self,
        slot: PluginSlotId,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        self.coordinator
            .call_slot_export(slot, module_name, export_name, arguments)
    }

    pub fn call_package_export(
        &self,
        package_name: &str,
        module_name: &str,
        export_name: &str,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmError> {
        let slot = self.slot_for_package_name(package_name)?;
        self.call_slot_export(slot, module_name, export_name, arguments)
    }

    /// Resolves a package export once into a stable, generation-refreshing callback handle.
    pub fn resolve_package_callback(
        &self,
        package_name: &str,
        module_name: &str,
        export_name: &str,
    ) -> Result<VmCallbackHandle, VmHostInterfaceError> {
        let slot = self
            .slot_for_package_name(package_name)
            .map_err(VmHostInterfaceError::CallbackFailed)?;
        let (generation, capabilities) =
            self.host_interfaces.active_owner(slot).ok_or_else(|| {
                VmHostInterfaceError::CallbackFailed(VmError::MissingSlot(slot.get()))
            })?;
        let caller = VmInterfaceCaller::new(slot, generation, capabilities);
        self.host_interfaces
            .intern_callback(&caller, module_name, export_name)
    }

    pub fn gc_step(&self, budget: VmGcBudget) -> Result<VmGcStepReport, VmError> {
        self.coordinator.gc_step(budget)
    }

    /// Invokes a stable callback handle against the owning slot's active generation.
    pub fn invoke_callback(
        &self,
        handle: &mut VmCallbackHandle,
        arguments: &[ScriptHostValue],
    ) -> Result<Option<ScriptHostValue>, VmHostInterfaceError> {
        let generation = self
            .coordinator
            .generation(handle.slot)
            .map_err(VmHostInterfaceError::CallbackFailed)?;
        let (module, function) = self.host_interfaces.resolve_callback(handle, generation)?;
        self.call_slot_export(handle.slot, module.as_ref(), function.as_ref(), arguments)
            .map_err(VmHostInterfaceError::CallbackFailed)
    }

    /// Runs every active VM system registered for `stage` in deterministic order.
    pub fn run_registered_systems(
        &self,
        stage: VmSystemStage,
        delta_seconds: f32,
    ) -> Result<usize, VmHostInterfaceError> {
        let systems = self.host_interfaces.systems_snapshot(stage);
        let system_count = systems.len();
        for system in systems.iter() {
            let mut callback = system.callback;
            self.invoke_callback(
                &mut callback,
                &[ScriptHostValue::Float(f64::from(delta_seconds))],
            )?;
        }
        Ok(system_count)
    }

    /// Returns active VM system descriptors for one scheduler stage.
    pub fn registered_systems(&self, stage: VmSystemStage) -> Vec<VmSystemRegistration> {
        self.host_interfaces.active_systems(stage)
    }

    /// Returns active behavior-node descriptors for AI adapters.
    pub fn registered_behavior_nodes(&self) -> Vec<VmBehaviorNodeRegistration> {
        self.host_interfaces.active_behavior_nodes()
    }

    /// Returns active RPC-handler descriptors for networking adapters.
    pub fn registered_rpc_handlers(&self) -> Vec<VmRpcHandlerRegistration> {
        self.host_interfaces.active_rpc_handlers()
    }

    /// Returns active editor-operation descriptors for editor adapters.
    pub fn registered_editor_operations(&self) -> Vec<VmEditorOperationRegistration> {
        self.host_interfaces.active_editor_operations()
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

    pub fn host_exports(&self) -> HostExportRegistry {
        self.host_exports.clone()
    }

    /// Returns the shared VM extension registry used by package host contexts.
    pub fn host_interfaces(&self) -> VmHostInterfaceRegistry {
        self.host_interfaces.clone()
    }

    /// Returns the shared VM reflection catalog used by package generations and Worlds.
    pub fn reflection_catalog(&self) -> VmReflectionCatalog {
        self.reflection_catalog.clone()
    }

    pub fn base_plugin_context(&self) -> &PluginContext {
        &self.plugin_context
    }

    pub(crate) fn active_generation(&self, slot: PluginSlotId) -> Result<u32, VmError> {
        self.host_interfaces
            .active_generation(slot)
            .ok_or(VmError::MissingSlot(slot.get()))
    }

    #[cfg(test)]
    pub(crate) fn active_plugin_snapshot(
        &self,
    ) -> Arc<super::super::host_interface::VmHostInterfaceActiveSnapshot> {
        self.host_interfaces.active_snapshot()
    }

    fn publish_active_interfaces(&self) {
        self.host_interfaces
            .publish_active_slots(self.coordinator.active_slots());
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

        VmPluginHostContext::new(
            plugin,
            package.manifest.capabilities.clone(),
            backend_selector.to_string(),
            source,
            self.host_registry.clone(),
            self.host_exports.clone(),
            self.host_interfaces.clone(),
            self.reflection_catalog.clone(),
            Default::default(),
            Arc::new(ManagerSlotLifecycle::new(self.self_ref.clone())),
        )
    }

    fn selected_backend_read(&self) -> RwLockReadGuard<'_, String> {
        self.selected_backend
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn selected_backend_write(&self) -> RwLockWriteGuard<'_, String> {
        self.selected_backend
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub(crate) fn install_reflection_world_extension(
        &self,
        core: &CoreHandle,
    ) -> Result<(), CoreError> {
        self.reflection_catalog.bind_core(core);
        let plan = self
            .reflection_catalog
            .world_runtime_extension_plan()
            .map_err(|error| {
                CoreError::Initialization(
                    VM_REFLECTION_WORLD_EXTENSION_NAME.to_string(),
                    error.to_string(),
                )
            })?;
        crate::scene::install_world_runtime_extension_plan(core, plan)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_and_system_dispatch_avoid_wide_record_clones() {
        let source = include_str!("vm_plugin_manager.rs")
            .split_once("#[cfg(test)]")
            .unwrap()
            .0;
        let callback = source.split("pub fn invoke_callback").nth(1).unwrap();
        let callback = callback
            .split("pub fn run_registered_systems")
            .next()
            .unwrap();
        let systems = source
            .split("pub fn run_registered_systems")
            .nth(1)
            .unwrap();
        let systems = systems.split("pub fn registered_systems").next().unwrap();

        assert!(callback.contains(".coordinator"));
        assert!(callback.contains(".generation(handle.slot)"));
        assert!(!callback.contains("self.slot(handle.slot)"));
        assert!(systems.contains("let system_count = systems.len();"));
        assert!(systems.contains("for mut system in systems"));
        assert!(!systems.contains("systems.iter().cloned()"));
    }

    #[test]
    fn vm_plugin_manager_selected_backend_accessors_recover_poisoned_lock() {
        let manager = VmPluginManager::with_builtin_backends(HostRegistry::default());
        let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut selected = manager.selected_backend.write().unwrap();
            *selected = DEFAULT_BACKEND_SELECTOR.to_string();
            panic!("poison vm plugin manager selected backend lock");
        }));
        assert!(poison.is_err());

        assert_eq!(manager.selected_backend_name(), DEFAULT_BACKEND_SELECTOR);
        manager
            .select_default_backend("builtin:mock")
            .expect("poisoned selected backend lock should recover for writes");
        assert_eq!(manager.selected_backend_name(), "builtin:mock");
    }
}
