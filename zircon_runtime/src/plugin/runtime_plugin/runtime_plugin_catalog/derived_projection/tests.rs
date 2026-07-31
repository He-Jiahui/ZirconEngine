use std::collections::HashSet;

use crate::builtin::RuntimePluginId;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::ProjectPluginManifest;
use crate::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
    PluginPackageManifest, RuntimePluginCatalog, RuntimePluginDescriptor, RuntimePluginFeature,
    RuntimePluginFeatureRegistrationReport, RuntimePluginRegistrationReport,
};

#[test]
fn catalog_generation_builds_one_projection_for_all_consumers() {
    for row_count in [1, 100, 1_000] {
        let catalog = catalog_fixture(row_count);
        let mut manifest = enabled_catalog_manifest(&catalog);

        let _ = catalog.complete_project_manifest(&manifest, RuntimeTargetMode::ClientRuntime);
        let _ = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
        let _ = catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);
        let _ = catalog.project_selection_for_package("plugin_0000");
        let module_name = format!("plugin_{:04}.runtime", row_count - 1);
        let _ = catalog.provider_package_id_for_runtime_module(&module_name);
        let first_feature = manifest.selections[0].features.remove(0);
        let _ = catalog.feature_manifest_for_selection("plugin_0000", &first_feature);

        let stats = catalog.projection_stats();
        assert_eq!(stats.projection_builds, 1);
        assert_eq!(stats.registrations_scanned, row_count);
        assert_eq!(stats.feature_registrations_scanned, row_count);
        assert_eq!(stats.feature_definitions_projected, row_count);
        assert_eq!(stats.runtime_modules_indexed, row_count);
        assert_eq!(stats.feature_dependency_edges_indexed, row_count * 2 - 1);
        let metrics = catalog.projection_metrics();
        assert_eq!(metrics.catalog_generation, 1);
        assert_eq!(metrics.projection_builds, 1);
        assert!(metrics.indexed_entry_count >= row_count);
        assert!(metrics.indexed_string_bytes >= row_count);
    }
}

#[test]
fn register_and_register_feature_advance_one_projection_generation_each() {
    let mut catalog = catalog_fixture(1);
    let initial_projection = catalog.projection_metrics();
    let descriptor = RuntimePluginDescriptor::builder(
        "plugin_registered",
        "Registered",
        RuntimePluginId::Sound,
        "zircon_plugin_registered_runtime",
    )
    .with_capability("runtime.plugin.plugin_registered")
    .build();

    assert!(catalog.register(&descriptor).is_published());
    let plugin_projection = catalog.projection_metrics();
    assert!(catalog.register_feature(&RegisteredFeature).is_published());
    let feature_projection = catalog.projection_metrics();

    assert_eq!(initial_projection.catalog_generation, 1);
    assert_eq!(initial_projection.projection_builds, 1);
    assert_eq!(plugin_projection.catalog_generation, 2);
    assert_eq!(plugin_projection.projection_builds, 2);
    assert_eq!(feature_projection.catalog_generation, 3);
    assert_eq!(feature_projection.projection_builds, 3);
}

#[test]
fn batch_registration_publishes_one_projection_generation_for_all_candidate_reports() {
    let mut catalog = catalog_fixture(1);
    let initial_projection = catalog.projection_metrics();
    let mut registrations = Vec::new();
    let mut feature_registrations = Vec::new();
    for index in 1..=100 {
        let plugin_id = format!("plugin_{index:04}");
        let feature = feature_manifest(index);
        let package = PluginPackageManifest::new(&plugin_id, &plugin_id)
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    format!("{plugin_id}.runtime"),
                    format!("zircon_{plugin_id}_runtime"),
                )
                .with_capabilities([format!("runtime.plugin.{plugin_id}")]),
            )
            .with_optional_feature(feature.clone());
        registrations.push(RuntimePluginRegistrationReport::from_native_package_manifest(package));
        feature_registrations.push(
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(feature, None),
        );
    }

    let outcome = catalog.register_reports_batch(registrations, feature_registrations);

    assert!(outcome.is_published(), "{:?}", outcome.diagnostics());
    let projection = catalog.projection_metrics();
    assert_eq!(
        projection.catalog_generation,
        initial_projection.catalog_generation + 1
    );
    assert_eq!(
        projection.projection_builds,
        initial_projection.projection_builds + 1
    );
    let stats = catalog.projection_stats();
    assert_eq!(
        stats.projection_builds,
        projection.projection_builds as usize
    );
    assert_eq!(stats.registrations_scanned, 101);
    assert_eq!(stats.feature_registrations_scanned, 101);
}

#[test]
fn batch_registration_builds_candidate_projection_and_diagnostics_before_publishing() {
    let registration = include_str!("../registration/plugin.rs");
    assert!(registration.contains("let mut update = self.update();"));
    assert!(registration.contains("update.publish()"));
    assert!(!registration.contains("self.registrations.extend"));

    let update = include_str!("../registration/update.rs");
    assert!(update.contains("RuntimePluginCatalogProjection::build("));
    assert!(update.contains("collect_catalog_diagnostics("));
    assert!(update.contains("if !diagnostics.is_empty()"));
    assert!(update.contains("self.catalog.projection = projection;"));
}

#[test]
fn feature_resolution_visits_each_feature_and_dependency_once() {
    for row_count in [1, 100, 1_000] {
        let catalog = catalog_fixture(row_count);
        let manifest = enabled_catalog_manifest(&catalog);

        let (report, stats) = catalog
            .feature_dependency_report_with_stats(&manifest, RuntimeTargetMode::ClientRuntime);

        assert!(report.blocked_features.is_empty());
        assert_eq!(report.available_features.len(), row_count);
        assert_eq!(stats.feature_status_evaluations, row_count);
        assert_eq!(stats.dependency_edges_scanned, row_count * 2 - 1);
        assert_eq!(stats.ready_queue_pushes, 0);
        assert_eq!(stats.ready_queue_pops, 0);
    }
}

#[test]
fn feature_capability_projection_borrows_manifest_rows_until_owned_index_insertion() {
    let iterator = include_str!("../feature_capabilities/feature.rs");
    assert!(iterator.contains("impl Iterator<Item = &str> + '_"));
    assert!(!iterator.contains(".iter().cloned()"));

    let projection = include_str!("../derived_projection.rs")
        .split_once("fn index_feature_capabilities")
        .expect("feature capability indexer should exist")
        .1;
    assert!(projection.contains("if !seen.insert(capability)"));
    assert!(projection.contains(".entry(capability.to_string())"));
    assert!(!projection.contains("seen.insert(capability.clone())"));

    let resolution = include_str!("../feature_resolution.rs");
    assert!(resolution.contains("available_capabilities.insert(capability.to_string())"));
    assert!(!resolution.contains("available_capabilities.insert(capability.clone())"));
}

#[test]
fn feature_resolution_preserves_fixed_point_pass_order_for_backward_edges() {
    let mut registrations = Vec::new();
    let mut feature_registrations = Vec::new();
    for index in 0..3 {
        let plugin_id = format!("plugin_{index:04}");
        let mut feature = PluginFeatureBundleManifest::new(
            format!("{plugin_id}.feature"),
            format!("{plugin_id}.feature"),
            &plugin_id,
        )
        .with_dependency(PluginFeatureDependency::primary(
            &plugin_id,
            format!("runtime.plugin.{plugin_id}"),
        ))
        .with_capability(format!("runtime.feature.{plugin_id}.feature"));
        if index == 0 {
            feature = feature.with_dependency(PluginFeatureDependency::required(
                "plugin_0001",
                "runtime.feature.plugin_0001.feature",
            ));
        }
        let package = PluginPackageManifest::new(&plugin_id, &plugin_id)
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    format!("{plugin_id}.runtime"),
                    format!("zircon_{plugin_id}_runtime"),
                )
                .with_capabilities([format!("runtime.plugin.{plugin_id}")]),
            )
            .with_optional_feature(feature.clone());
        registrations.push(RuntimePluginRegistrationReport::from_native_package_manifest(package));
        feature_registrations.push(
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(feature, None),
        );
    }
    let catalog =
        RuntimePluginCatalog::from_registration_reports(registrations, feature_registrations);
    let manifest = enabled_catalog_manifest(&catalog);

    let (report, stats) =
        catalog.feature_dependency_report_with_stats(&manifest, RuntimeTargetMode::ClientRuntime);

    assert_eq!(
        report.available_features,
        [
            "plugin_0001.feature",
            "plugin_0002.feature",
            "plugin_0000.feature",
        ]
    );
    assert_eq!(stats.feature_status_evaluations, 3);
    assert_eq!(stats.dependency_edges_scanned, 4);
    assert_eq!(stats.ready_queue_pushes, 1);
    assert_eq!(stats.ready_queue_pops, 1);
}

#[test]
fn immediate_blockers_drop_capabilities_published_by_earlier_ready_features() {
    let ready_feature =
        PluginFeatureBundleManifest::new("plugin_a.ready", "plugin_a.ready", "plugin_a")
            .with_dependency(PluginFeatureDependency::primary(
                "plugin_a",
                "runtime.plugin.plugin_a",
            ))
            .with_capability("runtime.feature.plugin_a.ready");
    let blocked_feature =
        PluginFeatureBundleManifest::new("plugin_b.blocked", "plugin_b.blocked", "plugin_b")
            .with_dependency(PluginFeatureDependency::primary(
                "plugin_b",
                "runtime.feature.plugin_a.ready",
            ))
            .with_capability("runtime.feature.plugin_b.blocked");
    let registrations = [
        RuntimePluginRegistrationReport::from_native_package_manifest(
            PluginPackageManifest::new("plugin_a", "plugin_a")
                .with_runtime_module(
                    PluginModuleManifest::runtime(
                        "plugin_a.runtime",
                        "zircon_plugin_plugin_a_runtime",
                    )
                    .with_capabilities(["runtime.plugin.plugin_a"]),
                )
                .with_optional_feature(ready_feature),
        ),
        RuntimePluginRegistrationReport::from_native_package_manifest(
            PluginPackageManifest::new("plugin_b", "plugin_b")
                .with_runtime_module(PluginModuleManifest::runtime(
                    "plugin_b.runtime",
                    "zircon_plugin_plugin_b_runtime",
                ))
                .with_optional_feature(blocked_feature),
        ),
    ];
    let catalog = RuntimePluginCatalog::from_registration_reports(registrations, []);
    let mut manifest = enabled_catalog_manifest(&catalog);
    manifest
        .selections
        .iter_mut()
        .find(|selection| selection.id == "plugin_b")
        .expect("plugin_b selection")
        .enabled = false;

    let report = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);

    assert_eq!(report.available_features, ["plugin_a.ready"]);
    let blocked = report
        .blocked_features
        .iter()
        .find(|blocked| blocked.feature_id == "plugin_b.blocked")
        .expect("plugin_b feature should remain blocked by its disabled owner");
    assert_eq!(blocked.missing_plugins, ["plugin_b"]);
    assert!(
        blocked.missing_capabilities.is_empty(),
        "capabilities published earlier in stable order must not remain in the final block"
    );
}

#[test]
fn completion_preserves_duplicate_selection_and_first_feature_match_semantics() {
    let catalog = catalog_fixture(1);
    let mut manifest = catalog.project_manifest();
    let selection = &mut manifest.selections[0];
    selection.runtime_crate = None;
    let mut duplicate_feature = selection.features[0].clone();
    selection.features[0].runtime_crate = None;
    duplicate_feature.runtime_crate = None;
    selection.features.push(duplicate_feature);
    let mut duplicate_selection = selection.clone();
    duplicate_selection.features.clear();
    manifest.selections.push(duplicate_selection);

    let completed = catalog.complete_project_manifest(&manifest, RuntimeTargetMode::ClientRuntime);

    assert!(completed.selections[0].runtime_crate.is_some());
    assert!(completed.selections[1].runtime_crate.is_some());
    assert!(completed.selections[0].features[0].runtime_crate.is_some());
    assert!(completed.selections[0].features[1].runtime_crate.is_none());
}

#[test]
fn duplicate_packages_keep_first_module_projection_and_merge_base_capabilities() {
    let first = PluginPackageManifest::new("duplicate", "duplicate").with_runtime_module(
        PluginModuleManifest::runtime("duplicate.first", "duplicate_first")
            .with_capabilities(["runtime.duplicate.first"]),
    );
    let second = PluginPackageManifest::new("duplicate", "duplicate").with_runtime_module(
        PluginModuleManifest::runtime("duplicate.second", "duplicate_second")
            .with_capabilities(["runtime.duplicate.second"]),
    );
    let catalog = RuntimePluginCatalog::from_registration_reports(
        [
            RuntimePluginRegistrationReport::from_native_package_manifest(first),
            RuntimePluginRegistrationReport::from_native_package_manifest(second),
        ],
        [],
    );

    assert_eq!(
        catalog.provider_package_id_for_runtime_module("duplicate.first"),
        Some("duplicate".to_string())
    );
    assert_eq!(
        catalog.provider_package_id_for_runtime_module("duplicate.second"),
        None
    );
    let capabilities = catalog.projection.base_capabilities_for_target(
        &HashSet::from(["duplicate".to_string()]),
        RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        capabilities,
        HashSet::from([
            "runtime.duplicate.first".to_string(),
            "runtime.duplicate.second".to_string(),
        ])
    );
}

#[test]
fn catalog_projection_preserves_completion_report_and_extension_order_as_json_bytes() {
    let catalog = catalog_fixture(3);
    let manifest = enabled_catalog_manifest(&catalog);
    let completed = catalog.complete_project_manifest(&manifest, RuntimeTargetMode::ClientRuntime);
    let report = catalog.feature_dependency_report(&manifest, RuntimeTargetMode::ClientRuntime);
    let extensions =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);
    let signature = serde_json::json!({
        "completed": completed
            .selections
            .iter()
            .map(|selection| serde_json::json!({
                "id": selection.id,
                "features": selection
                    .features
                    .iter()
                    .map(|feature| feature.id.as_str())
                    .collect::<Vec<_>>(),
            }))
            .collect::<Vec<_>>(),
        "available": report.available_features,
        "blocked": report
            .blocked_features
            .iter()
            .map(|blocked| blocked.feature_id.as_str())
            .collect::<Vec<_>>(),
        "extension_modules": extensions
            .registry
            .modules()
            .iter()
            .map(|module| module.name.as_str())
            .collect::<Vec<_>>(),
        "providers": [
            catalog.provider_package_id_for_runtime_module("plugin_0000.runtime"),
            catalog.provider_package_id_for_runtime_module("plugin_0002.runtime"),
        ],
    });

    assert_eq!(
        serde_json::to_vec(&signature).expect("catalog projection signature JSON"),
        br#"{"available":["plugin_0000.feature","plugin_0001.feature","plugin_0002.feature"],"blocked":[],"completed":[{"features":["plugin_0000.feature"],"id":"plugin_0000"},{"features":["plugin_0001.feature"],"id":"plugin_0001"},{"features":["plugin_0002.feature"],"id":"plugin_0002"}],"extension_modules":["plugin_0000.runtime","plugin_0001.runtime","plugin_0002.runtime","plugin_0000.feature.runtime","plugin_0001.feature.runtime","plugin_0002.feature.runtime"],"providers":["plugin_0000","plugin_0002"]}"#,
    );
}

fn catalog_fixture(row_count: usize) -> RuntimePluginCatalog {
    let mut registrations = Vec::with_capacity(row_count);
    let mut feature_registrations = Vec::with_capacity(row_count);
    for index in 0..row_count {
        let plugin_id = format!("plugin_{index:04}");
        let feature = feature_manifest(index);
        let package = PluginPackageManifest::new(&plugin_id, &plugin_id)
            .with_runtime_module(
                PluginModuleManifest::runtime(
                    format!("{plugin_id}.runtime"),
                    format!("zircon_{plugin_id}_runtime"),
                )
                .with_capabilities([format!("runtime.plugin.{plugin_id}")]),
            )
            .with_optional_feature(feature.clone());
        registrations.push(RuntimePluginRegistrationReport::from_native_package_manifest(package));
        feature_registrations.push(
            RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(feature, None),
        );
    }
    RuntimePluginCatalog::from_registration_reports(registrations, feature_registrations)
}

fn feature_manifest(index: usize) -> PluginFeatureBundleManifest {
    let plugin_id = format!("plugin_{index:04}");
    let feature_id = format!("{plugin_id}.feature");
    let mut feature = PluginFeatureBundleManifest::new(&feature_id, &feature_id, &plugin_id)
        .with_dependency(PluginFeatureDependency::primary(
            &plugin_id,
            format!("runtime.plugin.{plugin_id}"),
        ))
        .with_capability(format!("runtime.feature.{feature_id}"))
        .with_runtime_module(
            PluginModuleManifest::runtime(
                format!("{feature_id}.runtime"),
                format!("zircon_{plugin_id}_feature_runtime"),
            )
            .with_capabilities([format!("runtime.feature.{feature_id}")]),
        );
    if index > 0 {
        let previous_plugin = format!("plugin_{:04}", index - 1);
        feature = feature.with_dependency(PluginFeatureDependency::required(
            previous_plugin.clone(),
            format!("runtime.feature.{previous_plugin}.feature"),
        ));
    }
    feature
}

fn enabled_catalog_manifest(catalog: &RuntimePluginCatalog) -> ProjectPluginManifest {
    let mut manifest = catalog.project_manifest();
    for selection in &mut manifest.selections {
        selection.enabled = true;
        for feature in &mut selection.features {
            feature.enabled = true;
        }
    }
    manifest
}

struct RegisteredFeature;

impl RuntimePluginFeature for RegisteredFeature {
    fn manifest(&self) -> PluginFeatureBundleManifest {
        PluginFeatureBundleManifest::new(
            "plugin_registered.feature",
            "Registered feature",
            "plugin_registered",
        )
        .with_dependency(PluginFeatureDependency::primary(
            "plugin_registered",
            "runtime.plugin.plugin_registered",
        ))
        .with_capability("runtime.feature.plugin_registered.feature")
    }
}
