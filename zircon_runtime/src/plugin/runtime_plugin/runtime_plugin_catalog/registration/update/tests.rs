use std::sync::Arc;

use crate::core::framework::platform::RuntimeTargetMode;
use crate::plugin::{
    PluginFeatureBundleManifest, PluginFeatureDependency, PluginModuleManifest,
    PluginPackageManifest, RuntimePluginCatalog, RuntimePluginFeatureRegistrationReport,
    RuntimePluginRegistrationReport,
};

#[test]
fn candidate_update_appends_replaces_and_removes_in_one_published_generation() {
    let mut catalog = RuntimePluginCatalog::from_registration_reports(
        [
            package_report("a", "a.runtime"),
            package_report("b", "b.runtime"),
        ],
        [feature_report("a"), feature_report("b")],
    );
    let previous_projection = Arc::clone(&catalog.projection);
    let previous_metrics = catalog.projection_metrics();
    let mut update = catalog.update();

    update.append_registration(package_report("c", "c.runtime"));
    assert!(update.replace_registration(package_report("a", "a.next.runtime")));
    assert!(update.remove_registration("b"));
    update.append_feature_registration(feature_report("c"));
    assert!(update.replace_feature_registration(feature_report("a")));
    assert!(update.remove_feature_registration("b.feature", "b"));
    let outcome = update.publish();

    assert!(outcome.is_published(), "{:?}", outcome.diagnostics());
    assert_eq!(outcome.metrics().candidate_projection_builds, 1);
    assert_eq!(outcome.metrics().candidate_diagnostic_builds, 1);
    assert_eq!(outcome.metrics().published_generations, 1);
    assert_eq!(
        catalog
            .registrations()
            .iter()
            .map(|registration| registration.package_manifest.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "c"]
    );
    assert_eq!(
        catalog
            .feature_registrations()
            .iter()
            .map(|registration| registration.manifest.id.as_str())
            .collect::<Vec<_>>(),
        ["a.feature", "c.feature"]
    );
    assert_eq!(
        catalog.provider_package_id_for_runtime_module("a.next.runtime"),
        Some("a".to_string())
    );
    assert_eq!(
        catalog.provider_package_id_for_runtime_module("a.runtime"),
        None
    );
    assert_eq!(
        catalog.projection_metrics().catalog_generation,
        previous_metrics.catalog_generation + 1
    );
    assert_eq!(
        catalog.projection_metrics().projection_builds,
        previous_metrics.projection_builds + 1
    );
    assert_eq!(
        previous_projection.provider_for_runtime_module("a.runtime"),
        Some("a")
    );
}

#[test]
fn rejected_candidate_preserves_last_good_rows_projection_and_generation() {
    let mut catalog = RuntimePluginCatalog::from_registration_reports(
        [package_report("a", "a.runtime")],
        [feature_report("a")],
    );
    let previous_projection = Arc::clone(&catalog.projection);
    let previous_metrics = catalog.projection_metrics();
    let previous_diagnostics = catalog.diagnostics().to_vec();
    let manifest = catalog.project_manifest();
    let previous_plan =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);
    let previous_plan_metrics = catalog.project_plan_metrics();
    let mut update = catalog.update();

    update.append_feature_registration(feature_report("a"));
    let outcome = update.publish();

    assert!(!outcome.is_published());
    assert_eq!(outcome.metrics().candidate_projection_builds, 1);
    assert_eq!(outcome.metrics().candidate_diagnostic_builds, 1);
    assert_eq!(outcome.metrics().published_generations, 0);
    assert!(outcome
        .diagnostics()
        .iter()
        .any(|diagnostic| diagnostic.contains("duplicate optional feature id a.feature")));
    assert_eq!(catalog.registrations().len(), 1);
    assert_eq!(catalog.feature_registrations().len(), 1);
    assert_eq!(catalog.projection_metrics(), previous_metrics);
    assert_eq!(catalog.diagnostics(), previous_diagnostics);
    assert!(Arc::ptr_eq(&catalog.projection, &previous_projection));
    let current_plan =
        catalog.runtime_extensions_for_project(&manifest, RuntimeTargetMode::ClientRuntime);
    assert!(Arc::ptr_eq(&previous_plan, &current_plan));
    assert_eq!(catalog.project_plan_metrics(), previous_plan_metrics);
}

#[test]
fn empty_candidate_is_unchanged_without_build_or_publish() {
    let mut catalog = RuntimePluginCatalog::from_registration_reports(
        [package_report("a", "a.runtime")],
        [feature_report("a")],
    );
    let previous_metrics = catalog.projection_metrics();

    let outcome = catalog.update().publish();

    assert!(outcome.is_unchanged());
    assert_eq!(outcome.metrics().candidate_projection_builds, 0);
    assert_eq!(outcome.metrics().candidate_diagnostic_builds, 0);
    assert_eq!(outcome.metrics().published_generations, 0);
    assert_eq!(outcome.metrics().candidate_registration_rows_indexed, 0);
    assert_eq!(
        outcome
            .metrics()
            .candidate_feature_registration_rows_indexed,
        0
    );
    assert_eq!(catalog.projection_metrics(), previous_metrics);
}

#[test]
fn candidate_update_indexes_each_source_row_once_at_scale() {
    for size in [1_usize, 100, 10_000] {
        let registrations = (0..size)
            .map(|index| {
                let package_id = format!("scale_{index}");
                package_report(&package_id, &format!("{package_id}.runtime"))
            })
            .collect::<Vec<_>>();
        let feature_registrations = (0..size)
            .map(|index| feature_report(&format!("scale_{index}")))
            .collect::<Vec<_>>();
        let mut catalog =
            RuntimePluginCatalog::from_registration_reports(registrations, feature_registrations);
        let mutation_count = (size / 100).max(1);
        let mut update = catalog.update();

        for index in 0..mutation_count {
            let package_id = format!("scale_{index}");
            assert!(update.replace_registration(package_report(
                &package_id,
                &format!("{package_id}.next.runtime"),
            )));
            assert!(update.replace_feature_registration(feature_report(&package_id)));
        }
        let outcome = update.publish();
        let metrics = outcome.metrics();

        assert!(
            outcome.is_published(),
            "size={size}: {:?}",
            outcome.diagnostics()
        );
        assert_eq!(metrics.candidate_registration_rows_indexed, size);
        assert_eq!(metrics.candidate_feature_registration_rows_indexed, size);
        assert_eq!(metrics.candidate_registration_rows, size);
        assert_eq!(metrics.candidate_feature_registration_rows, size);
        assert_eq!(metrics.candidate_projection_builds, 1);
        assert_eq!(metrics.candidate_diagnostic_builds, 1);
        assert_eq!(metrics.published_generations, 1);
    }
}

fn package_report(package_id: &str, module_name: &str) -> RuntimePluginRegistrationReport {
    RuntimePluginRegistrationReport::from_native_package_manifest(
        PluginPackageManifest::new(package_id, package_id).with_runtime_module(
            PluginModuleManifest::runtime(
                module_name,
                format!("zircon_plugin_{package_id}_runtime"),
            )
            .with_capabilities([format!("runtime.plugin.{package_id}")]),
        ),
    )
}

fn feature_report(owner: &str) -> RuntimePluginFeatureRegistrationReport {
    RuntimePluginFeatureRegistrationReport::from_native_feature_manifest(
        PluginFeatureBundleManifest::new(
            format!("{owner}.feature"),
            format!("{owner}.feature"),
            owner,
        )
        .with_dependency(PluginFeatureDependency::primary(
            owner,
            format!("runtime.plugin.{owner}"),
        ))
        .with_capability(format!("runtime.feature.{owner}.feature")),
        None,
    )
}
