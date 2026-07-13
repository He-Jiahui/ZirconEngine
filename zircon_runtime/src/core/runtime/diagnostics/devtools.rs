use std::sync::{Mutex, MutexGuard};

use crate::core::{CoreHandle, LifecycleState, ServiceKind, StartupMode};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDevtoolsSnapshot {
    pub modules: Vec<RuntimeDevtoolsModuleSnapshot>,
    pub services: Vec<RuntimeDevtoolsServiceSnapshot>,
    pub scene_hooks: Vec<RuntimeDevtoolsSceneHookSnapshot>,
    pub plugin_catalog: Vec<RuntimeDevtoolsPluginCatalogEntry>,
    pub native_backend_status: RuntimeDevtoolsBackendStatus,
    pub vm_backend_status: RuntimeDevtoolsBackendStatus,
    pub diagnostics_summary: RuntimeDevtoolsDiagnosticsSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsModuleSnapshot {
    pub name: String,
    pub description: String,
    pub lifecycle: LifecycleState,
    pub service_count: usize,
    pub driver_count: usize,
    pub manager_count: usize,
    pub plugin_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsServiceSnapshot {
    pub name: String,
    pub owner_module: String,
    pub kind: ServiceKind,
    pub startup_mode: StartupMode,
    pub lifecycle: LifecycleState,
    pub dependencies: Vec<String>,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsSceneHookSnapshot {
    pub id: String,
    pub plugin_id: String,
    pub stage: String,
    pub order: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsPluginCatalogEntry {
    pub package_id: String,
    pub display_name: String,
    pub crate_name: String,
    pub capabilities: Vec<String>,
    pub target_modes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDevtoolsBackendStatus {
    pub backend: String,
    pub available: bool,
    pub loaded_plugin_count: usize,
}

impl Default for RuntimeDevtoolsBackendStatus {
    fn default() -> Self {
        Self {
            backend: String::new(),
            available: false,
            loaded_plugin_count: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDevtoolsDiagnosticsSummary {
    pub series_count: usize,
    pub tagged_subsystems: Vec<String>,
}

pub fn collect_runtime_devtools_snapshot(core: &CoreHandle) -> RuntimeDevtoolsSnapshot {
    let diagnostics = super::collect_runtime_diagnostics(core);
    RuntimeDevtoolsSnapshot {
        modules: collect_module_snapshots(core),
        services: collect_service_snapshots(core),
        scene_hooks: collect_scene_hook_snapshots(core),
        plugin_catalog: collect_plugin_catalog_entries(core),
        native_backend_status: RuntimeDevtoolsBackendStatus {
            backend: "native_dynamic".to_string(),
            available: true,
            loaded_plugin_count: 0,
        },
        vm_backend_status: RuntimeDevtoolsBackendStatus {
            backend: "vm".to_string(),
            available: false,
            loaded_plugin_count: 0,
        },
        diagnostics_summary: RuntimeDevtoolsDiagnosticsSummary {
            series_count: diagnostics.store.series.len(),
            tagged_subsystems: tagged_subsystems(&diagnostics.store),
        },
    }
}

fn collect_module_snapshots(core: &CoreHandle) -> Vec<RuntimeDevtoolsModuleSnapshot> {
    let modules = lock_poison_recovered(&core.inner.modules);
    let mut snapshots = modules
        .values()
        .map(|entry| {
            let descriptor = entry.descriptor();
            RuntimeDevtoolsModuleSnapshot {
                name: descriptor.name.clone(),
                description: descriptor.description.clone(),
                lifecycle: entry.lifecycle,
                service_count: entry.service_names.len(),
                driver_count: descriptor.drivers.len(),
                manager_count: descriptor.managers.len(),
                plugin_count: descriptor.plugins.len(),
            }
        })
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| left.name.cmp(&right.name));
    snapshots
}

fn collect_service_snapshots(core: &CoreHandle) -> Vec<RuntimeDevtoolsServiceSnapshot> {
    let services = lock_poison_recovered(&core.inner.services);
    let mut snapshots = services
        .iter()
        .map(|(name, entry)| RuntimeDevtoolsServiceSnapshot {
            name: name.to_string(),
            owner_module: name.module_name().to_string(),
            kind: name.service_kind(),
            startup_mode: entry.startup_mode,
            lifecycle: entry.lifecycle,
            dependencies: entry
                .dependencies
                .iter()
                .map(|dependency| dependency.to_string())
                .collect(),
            active: entry.instance.is_some(),
        })
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| left.name.cmp(&right.name));
    snapshots
}

fn collect_scene_hook_snapshots(core: &CoreHandle) -> Vec<RuntimeDevtoolsSceneHookSnapshot> {
    let mut snapshots = lock_poison_recovered(&core.inner.scene_hook_snapshots).clone();
    snapshots.sort_by(|left, right| {
        left.stage
            .cmp(&right.stage)
            .then(left.order.cmp(&right.order))
            .then(left.id.cmp(&right.id))
    });
    snapshots
}

fn collect_plugin_catalog_entries(core: &CoreHandle) -> Vec<RuntimeDevtoolsPluginCatalogEntry> {
    let mut entries = lock_poison_recovered(&core.inner.devtools_plugin_catalog_entries).clone();
    entries.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    entries
}

fn tagged_subsystems(store: &super::DiagnosticStoreSnapshot) -> Vec<String> {
    let mut tags = store
        .series
        .iter()
        .flat_map(|series| series.subsystem_tags.iter().cloned())
        .collect::<Vec<_>>();
    tags.sort();
    tags.dedup();
    tags
}

fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::Arc;

    use crate::core::runtime::ServiceObject;
    use crate::core::{
        CoreRuntime, DriverDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
    };

    use super::{collect_runtime_devtools_snapshot, RuntimeDevtoolsPluginCatalogEntry};

    #[test]
    fn devtools_snapshot_lists_modules_services_and_builtin_catalog() {
        let runtime = CoreRuntime::new();
        runtime
            .register_module(
                ModuleDescriptor::new("diagnostic_test", "Diagnostics Test").with_driver(
                    DriverDescriptor::new(
                        RegistryName::new("diagnostic_test.Driver.Clock").unwrap(),
                        StartupMode::Lazy,
                        Vec::new(),
                        Arc::new(|_| Ok(Arc::new(7_u32) as ServiceObject)),
                    ),
                ),
            )
            .unwrap();
        runtime.replace_devtools_plugin_catalog_entries(vec![RuntimeDevtoolsPluginCatalogEntry {
            package_id: "physics".to_string(),
            display_name: "Physics".to_string(),
            crate_name: "zircon_plugin_physics_runtime".to_string(),
            capabilities: vec!["runtime.plugin.physics".to_string()],
            target_modes: vec!["ClientRuntime".to_string()],
        }]);

        let snapshot = collect_runtime_devtools_snapshot(&runtime.handle());

        let module = snapshot
            .modules
            .iter()
            .find(|module| module.name == "diagnostic_test")
            .expect("module snapshot should be projected from the module registry");
        assert_eq!(module.service_count, 1);
        assert_eq!(module.driver_count, 1);
        let service = snapshot
            .services
            .iter()
            .find(|service| service.name == "diagnostic_test.Driver.Clock")
            .expect("service snapshot should be projected from the registry key");
        assert_eq!(service.owner_module, "diagnostic_test");
        assert_eq!(service.kind, ServiceKind::Driver);
        assert!(snapshot
            .plugin_catalog
            .iter()
            .any(|plugin| plugin.package_id == "physics"));
        assert_eq!(snapshot.native_backend_status.backend, "native_dynamic");
        assert_eq!(snapshot.vm_backend_status.backend, "vm");
    }

    #[test]
    fn devtools_snapshot_recovers_poisoned_runtime_registry_locks() {
        let runtime = CoreRuntime::new();
        let handle = runtime.handle();

        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.modules.lock().unwrap();
            panic!("poison devtools modules registry");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.services.lock().unwrap();
            panic!("poison devtools services registry");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.scene_hook_snapshots.lock().unwrap();
            panic!("poison devtools scene hook diagnostics snapshots");
        }));
        let _ = panic::catch_unwind(AssertUnwindSafe(|| {
            let _guard = handle.inner.devtools_plugin_catalog_entries.lock().unwrap();
            panic!("poison devtools plugin catalog entries");
        }));

        let snapshot = collect_runtime_devtools_snapshot(&handle);
        assert!(snapshot.modules.is_empty());
        assert!(snapshot.services.is_empty());
        assert!(snapshot.scene_hooks.is_empty());
        assert!(snapshot.plugin_catalog.is_empty());
    }
}
