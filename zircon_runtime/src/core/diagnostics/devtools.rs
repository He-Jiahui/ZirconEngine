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
        plugin_catalog: collect_plugin_catalog_entries(),
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
    let modules = core.inner.modules.lock().unwrap();
    let mut snapshots = modules
        .values()
        .map(|entry| RuntimeDevtoolsModuleSnapshot {
            name: entry.descriptor.name.clone(),
            description: entry.descriptor.description.clone(),
            lifecycle: entry.lifecycle,
            driver_count: entry.descriptor.drivers.len(),
            manager_count: entry.descriptor.managers.len(),
            plugin_count: entry.descriptor.plugins.len(),
        })
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| left.name.cmp(&right.name));
    snapshots
}

fn collect_service_snapshots(core: &CoreHandle) -> Vec<RuntimeDevtoolsServiceSnapshot> {
    let services = core.inner.services.lock().unwrap();
    let mut snapshots = services
        .values()
        .map(|entry| RuntimeDevtoolsServiceSnapshot {
            name: entry.name.to_string(),
            owner_module: entry.owner_module.clone(),
            kind: entry.kind,
            startup_mode: entry.startup_mode,
            lifecycle: entry.lifecycle,
            dependencies: entry
                .dependencies
                .iter()
                .map(|dependency| dependency.name.to_string())
                .collect(),
            active: entry.instance.is_some(),
        })
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| left.name.cmp(&right.name));
    snapshots
}

fn collect_scene_hook_snapshots(core: &CoreHandle) -> Vec<RuntimeDevtoolsSceneHookSnapshot> {
    let hooks = core.inner.scene_hooks.lock().unwrap();
    let mut snapshots = hooks
        .iter()
        .map(|hook| {
            let descriptor = hook.descriptor();
            RuntimeDevtoolsSceneHookSnapshot {
                id: descriptor.id.clone(),
                plugin_id: descriptor.plugin_id.clone(),
                stage: format!("{:?}", descriptor.stage),
                order: descriptor.order,
            }
        })
        .collect::<Vec<_>>();
    snapshots.sort_by(|left, right| {
        left.stage
            .cmp(&right.stage)
            .then(left.order.cmp(&right.order))
            .then(left.id.cmp(&right.id))
    });
    snapshots
}

fn collect_plugin_catalog_entries() -> Vec<RuntimeDevtoolsPluginCatalogEntry> {
    let mut entries = crate::plugin::RuntimePluginDescriptor::builtin_catalog()
        .into_iter()
        .map(|descriptor| RuntimeDevtoolsPluginCatalogEntry {
            package_id: descriptor.package_id,
            display_name: descriptor.display_name,
            crate_name: descriptor.crate_name,
            capabilities: descriptor.capabilities,
            target_modes: descriptor
                .target_modes
                .into_iter()
                .map(|mode| format!("{:?}", mode))
                .collect(),
        })
        .collect::<Vec<_>>();
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::{
        CoreRuntime, DriverDescriptor, ModuleDescriptor, RegistryName, ServiceObject, StartupMode,
    };

    use super::collect_runtime_devtools_snapshot;

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

        let snapshot = collect_runtime_devtools_snapshot(&runtime.handle());

        assert!(snapshot
            .modules
            .iter()
            .any(|module| module.name == "diagnostic_test" && module.driver_count == 1));
        assert!(snapshot
            .services
            .iter()
            .any(|service| service.name == "diagnostic_test.Driver.Clock"));
        assert!(snapshot
            .plugin_catalog
            .iter()
            .any(|plugin| plugin.package_id == "physics"));
        assert_eq!(snapshot.native_backend_status.backend, "native_dynamic");
        assert_eq!(snapshot.vm_backend_status.backend, "vm");
    }
}
