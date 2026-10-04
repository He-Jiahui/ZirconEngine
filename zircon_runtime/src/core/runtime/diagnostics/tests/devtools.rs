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
    let source = include_str!("../devtools.rs");
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
    let source = include_str!("../devtools.rs");
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
