use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};

use crate::core::{CoreHandle, LifecycleState, ServiceKind, StartupMode};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDevtoolsSnapshot {
    pub modules: Vec<RuntimeDevtoolsModuleSnapshot>,
    pub services: Vec<RuntimeDevtoolsServiceSnapshot>,
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

pub(crate) fn project_runtime_devtools_snapshot(
    core: &CoreHandle,
    diagnostics: &super::RuntimeDiagnosticsSnapshot,
) -> RuntimeDevtoolsSnapshot {
    RuntimeDevtoolsSnapshot {
        modules: collect_module_snapshots(core),
        services: collect_service_snapshots(core),
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
    drop(modules);
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
    drop(services);
    snapshots.sort_by(|left, right| left.name.cmp(&right.name));
    snapshots
}

fn collect_plugin_catalog_entries(core: &CoreHandle) -> Vec<RuntimeDevtoolsPluginCatalogEntry> {
    let mut entries = lock_poison_recovered(&core.inner.devtools_plugin_catalog_entries).clone();
    entries.sort_by(|left, right| left.package_id.cmp(&right.package_id));
    entries
}

fn tagged_subsystems(store: &super::DiagnosticStoreSnapshot) -> Vec<String> {
    let mut unique_tags = HashSet::<&str>::new();
    for series in &store.series {
        unique_tags.extend(series.subsystem_tags.iter().map(String::as_str));
    }
    let mut tags = unique_tags
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    tags.sort_unstable();
    tags
}

fn lock_poison_recovered<T>(lock: &Mutex<T>) -> MutexGuard<'_, T> {
    lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    use crate::core::runtime::diagnostics::{
        DiagnosticPath, DiagnosticSeriesSnapshot, DiagnosticStoreSnapshot,
    };
    use crate::core::runtime::ServiceObject;
    use crate::core::{
        CoreRuntime, DriverDescriptor, ModuleDescriptor, RegistryName, ServiceKind, StartupMode,
    };

    use super::{
        project_runtime_devtools_snapshot, tagged_subsystems, RuntimeDevtoolsPluginCatalogEntry,
    };

    fn diagnostic_series(index: usize, subsystem_tags: &[&str]) -> DiagnosticSeriesSnapshot {
        DiagnosticSeriesSnapshot {
            path: DiagnosticPath::new(format!("runtime.devtools.metric.{index}")),
            unit: None,
            subsystem_tags: subsystem_tags
                .iter()
                .map(|tag| (*tag).to_string())
                .collect(),
            current: None,
            smoothed: None,
            min: None,
            max: None,
            history: Vec::new(),
        }
    }

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

        let snapshot = project_runtime_devtools_snapshot(
            &runtime.handle(),
            &super::super::RuntimeDiagnosticsSnapshot::default(),
        );

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
            let _guard = handle.inner.devtools_plugin_catalog_entries.lock().unwrap();
            panic!("poison devtools plugin catalog entries");
        }));

        let snapshot = project_runtime_devtools_snapshot(
            &handle,
            &super::super::RuntimeDiagnosticsSnapshot::default(),
        );
        assert!(snapshot.modules.is_empty());
        assert!(snapshot.services.is_empty());
        assert!(snapshot.plugin_catalog.is_empty());
    }

    #[test]
    fn devtools_projection_releases_registry_locks_before_sorting() {
        let source = include_str!("devtools.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("devtools implementation");

        assert!(implementation.contains("drop(modules);"));
        assert!(implementation.contains("drop(services);"));
        assert!(implementation.contains(".map(String::as_str)"));
        assert!(!implementation.contains("subsystem_tags.iter().cloned()"));
    }

    #[test]
    fn optimization_wave_20260824e_runtime03_devtools_tags_are_sorted_and_deduplicated() {
        let store = DiagnosticStoreSnapshot {
            series: vec![
                diagnostic_series(0, &["render", "frame", "render"]),
                diagnostic_series(1, &["physics", "frame"]),
                diagnostic_series(2, &["animation", "render"]),
            ],
        };

        assert_eq!(
            tagged_subsystems(&store),
            ["animation", "frame", "physics", "render"]
        );
    }

    #[test]
    fn optimization_wave_20260824e_runtime03_devtools_tag_projection_bounds_temporary_items() {
        let source = include_str!("devtools.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("devtools implementation");
        let projection = implementation
            .split("fn tagged_subsystems")
            .nth(1)
            .and_then(|source| source.split("fn lock_poison_recovered").next())
            .expect("tagged subsystem projection");

        assert!(projection.contains("HashSet::<&str>::new()"));
        assert!(!projection.contains(".flat_map("));
        assert!(!projection.contains("tags.dedup()"));
    }

    #[test]
    #[ignore = "managed release evidence"]
    fn optimization_wave_20260824e_runtime03_devtools_tag_projection_evidence() {
        const SERIES_COUNT: usize = 100_000;
        const TAGS_PER_SERIES: usize = 4;
        const UNIQUE_TAG_COUNT: usize = 4;
        const TARGET: Duration = Duration::from_secs(1);

        let tags = ["runtime", "frame", "render", "shared"];
        let store = DiagnosticStoreSnapshot {
            series: (0..SERIES_COUNT)
                .map(|index| diagnostic_series(index, &tags))
                .collect(),
        };

        let started = Instant::now();
        let projected = tagged_subsystems(&store);
        let elapsed = started.elapsed();
        let temporary_items_before = SERIES_COUNT * TAGS_PER_SERIES;
        let temporary_items_after = projected.len();
        let reduction_percent =
            (1.0 - temporary_items_after as f64 / temporary_items_before as f64) * 100.0;

        assert_eq!(projected, ["frame", "render", "runtime", "shared"]);
        assert_eq!(temporary_items_after, UNIQUE_TAG_COUNT);
        assert!(elapsed <= TARGET, "elapsed={elapsed:?} target={TARGET:?}");
        println!(
            "RUNTIME03_DEVTOOLS_TAG_BENCH_V1 series={} tags_per_series={} temporary_items_before={} temporary_items_after={} temporary_item_reduction_percent={:.4} elapsed_ns={} target_ns={}",
            SERIES_COUNT,
            TAGS_PER_SERIES,
            temporary_items_before,
            temporary_items_after,
            reduction_percent,
            elapsed.as_nanos(),
            TARGET.as_nanos()
        );
    }
}
