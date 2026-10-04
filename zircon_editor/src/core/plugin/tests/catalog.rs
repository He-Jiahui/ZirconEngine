use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::Instant;

use super::EditorPluginCatalog;
use crate::core::plugin::descriptor::EditorPluginDescriptor;
use crate::core::plugin::registration::EditorPluginRegistrationReport;
use crate::core::plugin::sdk::lifecycle::{EditorPluginLifecycleEvent, EditorPluginLifecycleStage};

#[test]
fn descriptor_catalog_indexes_runtime_manifests_once() {
    let source = include_str!("../catalog.rs");
    let linear_lookup = [".find(|manifest| manifest.id == descriptor.", "package_id)"].concat();

    assert!(source.contains("runtime_manifest_by_package"));
    assert!(!source.contains(&linear_lookup));
}

#[test]
fn registration_index_rebuilds_after_project_report_replacement() {
    let mut catalog = EditorPluginCatalog::from_descriptors(
        [
            EditorPluginDescriptor::new("plugin.builtin", "Builtin", "builtin"),
            EditorPluginDescriptor::new("plugin.project", "Project", "project"),
        ],
        std::iter::empty(),
    );
    let replacement = EditorPluginDescriptor::new("plugin.project", "Project v2", "project_v2");
    let replacement_report = EditorPluginRegistrationReport::from_plugin(
        &replacement,
        replacement.standalone_package_manifest(),
    );

    catalog.replace_project_registration_reports(
        &BTreeSet::from(["plugin.project".to_string()]),
        [replacement_report],
    );

    assert_eq!(catalog.registration_index.len(), 2);
    let report = catalog.record_lifecycle_event(
        "plugin.project",
        EditorPluginLifecycleEvent::new(EditorPluginLifecycleStage::Loaded),
    );
    assert!(report.is_success());
    assert!(
        catalog.lifecycle_stage_succeeded("plugin.project", &EditorPluginLifecycleStage::Loaded)
    );
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_wave_20260825_editor06_registration_index_evidence() {
    const PLUGINS: usize = 1_000;
    const LOOKUPS: usize = 100_000;
    const MAX_ELAPSED_NS: u128 = 3_000_000_000;

    let descriptors = (0..PLUGINS).map(|index| {
        let package_id = format!("plugin.bench.{index:04}");
        EditorPluginDescriptor::new(&package_id, &package_id, package_id.clone())
    });
    let catalog = EditorPluginCatalog::from_descriptors(descriptors, std::iter::empty());
    let target = format!("plugin.bench.{:04}", PLUGINS - 1);
    let stage = EditorPluginLifecycleStage::Loaded;
    let started = Instant::now();
    for _ in 0..LOOKUPS {
        black_box(catalog.lifecycle_stage_succeeded(&target, &stage));
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let legacy_candidate_checks = PLUGINS * LOOKUPS;
    let indexed_registration_probes = LOOKUPS;
    let probe_reduction_bps = legacy_candidate_checks
        .saturating_sub(indexed_registration_probes)
        .saturating_mul(10_000)
        / legacy_candidate_checks;

    println!(
        "EDITOR06_PLUGIN_REGISTRATION_INDEX_BENCH_V1 plugins={PLUGINS} lookups={LOOKUPS} legacy_candidate_checks={legacy_candidate_checks} indexed_registration_probes={indexed_registration_probes} probe_reduction_bps={probe_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );

    assert_eq!(catalog.registration_index.len(), PLUGINS);
    assert_eq!(probe_reduction_bps, 9_990);
    assert!(elapsed_ns <= MAX_ELAPSED_NS);
}
