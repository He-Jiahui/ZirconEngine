use super::*;
use crate::asset::AssetUri;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ExportProfile, ExportTargetPlatform, RuntimeProfileId};

#[test]
fn single_pass_linked_runtime_projection_preserves_contract() {
    let profile = ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate);
    let first = ProjectPluginSelection::runtime_plugin("duplicate-runtime-a", true, false)
        .with_runtime_crate("zircon_plugin_shared_runtime");
    let second = ProjectPluginSelection::runtime_plugin("duplicate-runtime-b", true, false)
        .with_runtime_crate("zircon_plugin_shared_runtime");
    let projection = linked_runtime_plugin_projection(&[&first, &second], &profile);

    assert_eq!(projection.crate_links.len(), 1);
    assert_eq!(
        projection.crate_links[0].crate_name,
        "zircon_plugin_shared_runtime"
    );
    assert_eq!(projection.crate_links[0].path, "shared/runtime");
    assert!(projection.package_ids.contains("duplicate-runtime-a"));
    assert!(projection.package_ids.contains("duplicate-runtime-b"));
}

#[test]
fn export_generation_builds_each_manifest_validation_view_once() {
    let source = include_str!("../from_project_manifest.rs");
    let descriptor_rebuild = ["RuntimePluginDescriptor", "::builtin_catalog"].concat();
    let catalog_call = ["builtin_runtime_plugin_catalog", "();"].concat();
    let direct_catalog_build = ["RuntimePluginCatalog", "::builtin"].concat();
    let shared_catalog = ["RuntimePluginCatalog", "::builtin_shared()"].concat();
    assert!(!source.contains(&descriptor_rebuild));
    assert_eq!(source.matches(&catalog_call).count(), 1);
    assert_eq!(source.matches(&direct_catalog_build).count(), 1);
    assert!(source.contains(&shared_catalog));
    reset_builtin_catalog_build_count();
    begin_projection_build_observation();
    let mut manifest = ProjectManifest::new(
        "availability-projection",
        AssetUri::parse("res://scenes/main.scene.toml").expect("fixture asset URI"),
        1,
    );
    manifest.export_profiles = vec![ExportProfile::new(
        "client",
        RuntimeTargetMode::ClientRuntime,
        ExportTargetPlatform::Windows,
        RuntimeProfileId::Client2d,
    )
    .with_strategy(ExportPackagingStrategy::SourceTemplate)];

    let _ = ExportBuildPlan::from_project_manifest(&manifest, "client")
        .expect("export generation should succeed");

    assert_eq!(builtin_catalog_build_count(), 1);
    assert_eq!(observed_projection_builds(), 2);
}

#[test]
fn completed_plugin_manifest_is_reused_for_feature_resolution() {
    let source = include_str!("../from_project_manifest.rs");
    let completing_api = ["feature_dependency_report", "(&completed_plugins"].concat();
    let completed_api = ["feature_dependency_report_for_", "completed_manifest"].concat();

    assert!(!source.contains(&completing_api));
    assert!(source.contains(&completed_api));
}

#[test]
fn missing_profile_returns_typed_plan_error() {
    let manifest = ProjectManifest::new(
        "typed-error-contract",
        AssetUri::parse("res://scenes/main.scene.toml").expect("fixture asset URI"),
        1,
    );

    let error = ExportBuildPlan::from_project_manifest(&manifest, "missing-profile")
        .expect_err("unknown profile must fail");

    assert_eq!(
        error,
        ExportBuildPlanError::MissingProfile {
            profile_name: "missing-profile".to_string(),
        }
    );
}
