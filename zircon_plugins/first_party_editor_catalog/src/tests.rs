use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

use super::first_party_editor_plugin_registrations_for_manifest;

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn selected_navigation_provider_projects_typed_runtime_consumer() {
    let manifest = manifest_with_navigation(true, RuntimeTargetMode::EditorHost);

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
    let registration = &registrations[0];
    assert_eq!(registration.package_manifest.id, "navigation");
    assert!(registration.is_success());
    assert_eq!(registration.runtime_event_consumers.manifests().len(), 1);
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn disabled_or_wrong_target_navigation_provider_is_not_projected() {
    let disabled = manifest_with_navigation(false, RuntimeTargetMode::EditorHost);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &disabled,
    )
    .is_empty());

    let runtime_only = manifest_with_navigation(true, RuntimeTargetMode::ClientRuntime);
    assert!(first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &runtime_only,
    )
    .is_empty());
}

#[cfg(feature = "navigation-editor-plugin")]
#[test]
fn duplicate_project_selections_produce_one_editor_registration() {
    let selection =
        ProjectPluginSelection::runtime_plugin(RuntimePluginId::Navigation, true, false)
            .with_target_modes([RuntimeTargetMode::EditorHost]);
    let manifest = ProjectPluginManifest {
        selections: vec![selection.clone(), selection],
    };

    let registrations = first_party_editor_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );

    assert_eq!(registrations.len(), 1);
}

#[cfg(feature = "navigation-editor-plugin")]
fn manifest_with_navigation(enabled: bool, target: RuntimeTargetMode) -> ProjectPluginManifest {
    ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            enabled,
            false,
        )
        .with_target_modes([target])],
    }
}
