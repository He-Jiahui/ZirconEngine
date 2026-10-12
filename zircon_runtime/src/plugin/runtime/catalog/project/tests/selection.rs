use super::CompiledRuntimePluginBaseSelection;
use crate::core::framework::platform::RuntimeTargetMode;
use crate::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};
use crate::plugin::{PluginPackageManifest, PluginPackageRole, RuntimePluginRegistrationReport};

fn required_selection(id: &str) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(id, true, true)],
    }
}

#[test]
fn required_unknown_selection_is_an_explicit_fatal_admission_result() {
    let selection = CompiledRuntimePluginBaseSelection::compile(
        &[],
        &[],
        &required_selection("third_party.missing"),
        RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        selection.fatal_diagnostic.as_deref(),
        Some("required runtime plugin selection `third_party.missing` has no catalog registration")
    );
}

#[test]
fn required_selection_only_blocks_its_declared_target() {
    let mut manifest = required_selection("third_party.missing");
    manifest.selections[0].target_modes = vec![RuntimeTargetMode::EditorHost];

    let client = CompiledRuntimePluginBaseSelection::compile(
        &[],
        &[],
        &manifest,
        RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(client.fatal_diagnostic, None);
    assert!(client.ordered_plugin_registration_indices.is_empty());

    let editor = CompiledRuntimePluginBaseSelection::compile(
        &[],
        &[],
        &manifest,
        RuntimeTargetMode::EditorHost,
    );
    assert_eq!(
        editor.fatal_diagnostic.as_deref(),
        Some("required runtime plugin selection `third_party.missing` has no catalog registration")
    );
}

#[test]
fn required_disabled_provider_blocks_selection_readiness() {
    let package = PluginPackageManifest::new("physics", "Physics");
    let mut registration = RuntimePluginRegistrationReport::from_native_package_manifest(package);
    registration.project_selection.enabled = false;
    let selection = CompiledRuntimePluginBaseSelection::compile(
        &[registration],
        &[],
        &required_selection("physics"),
        RuntimeTargetMode::ClientRuntime,
    );

    assert!(selection
        .fatal_diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("has no provider for target")));
}

#[test]
fn required_alias_selection_matches_canonical_catalog_registration() {
    let package = PluginPackageManifest::new("sound", "Sound");
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(package);
    let mut selection = CompiledRuntimePluginBaseSelection::compile(
        &[registration],
        &[],
        &required_selection("audio"),
        RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(selection.fatal_diagnostic, None);
    assert_eq!(&*selection.ordered_plugin_registration_indices, &[0]);

    let (effective_enabled_plugins, _) = selection.take_feature_dependency_inputs();
    assert!(effective_enabled_plugins.contains("audio"));
    assert!(!effective_enabled_plugins.contains("sound"));
}

#[test]
fn required_same_target_alias_duplicate_is_fatal() {
    let package = PluginPackageManifest::new("sound", "Sound");
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(package);
    let manifest = ProjectPluginManifest {
        selections: vec![
            ProjectPluginSelection::runtime_plugin("sound", true, true),
            ProjectPluginSelection::runtime_plugin("audio", true, true),
        ],
    };

    let selection = CompiledRuntimePluginBaseSelection::compile(
        &[registration],
        &[],
        &manifest,
        RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        selection.fatal_diagnostic.as_deref(),
        Some("required runtime plugin selection `audio` is Duplicate")
    );
}

#[test]
fn required_aliases_on_distinct_targets_remain_independent() {
    let package = PluginPackageManifest::new("sound", "Sound");
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(package);
    let mut client = ProjectPluginSelection::runtime_plugin("sound", true, true);
    client.target_modes = vec![RuntimeTargetMode::ClientRuntime];
    let mut editor = ProjectPluginSelection::runtime_plugin("audio", true, true);
    editor.target_modes = vec![RuntimeTargetMode::EditorHost];
    let manifest = ProjectPluginManifest {
        selections: vec![client, editor],
    };

    for target in [
        RuntimeTargetMode::ClientRuntime,
        RuntimeTargetMode::EditorHost,
    ] {
        let selection = CompiledRuntimePluginBaseSelection::compile(
            std::slice::from_ref(&registration),
            &[],
            &manifest,
            target,
        );
        assert_eq!(selection.fatal_diagnostic, None, "target {target:?}");
        assert_eq!(&*selection.ordered_plugin_registration_indices, &[0]);
    }
}

#[test]
fn required_carrier_role_is_not_a_product_catalog_provider() {
    let package = PluginPackageManifest::new("fixture", "Fixture")
        .with_package_role(PluginPackageRole::TestFixture);
    let registration = RuntimePluginRegistrationReport::from_native_package_manifest(package);
    let selection = CompiledRuntimePluginBaseSelection::compile(
        &[registration],
        &[],
        &required_selection("fixture"),
        RuntimeTargetMode::ClientRuntime,
    );

    assert!(selection
        .fatal_diagnostic
        .as_deref()
        .is_some_and(|diagnostic| diagnostic.contains("has no provider for target")));
    assert!(selection.ordered_plugin_registration_indices.is_empty());
}
