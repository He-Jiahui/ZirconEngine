#[cfg(feature = "target-editor-host")]
use zircon_editor::EditorPluginRegistrationReport;
#[cfg(feature = "target-editor-host")]
use zircon_runtime::core::framework::project::ProjectPluginManifest;

#[cfg(feature = "target-editor-host")]
use super::EntryConfig;

#[cfg(feature = "target-editor-host")]
pub fn first_party_editor_plugin_registrations_for_config(
    config: &EntryConfig,
) -> Vec<EditorPluginRegistrationReport> {
    let manifest = config.project_plugin_manifest().unwrap_or_default();
    first_party_editor_plugin_registrations_for_manifest(config.target_mode, &manifest)
}

#[cfg(feature = "target-editor-host")]
pub fn first_party_editor_plugin_registrations_for_manifest(
    target_mode: zircon_runtime::core::framework::platform::RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<EditorPluginRegistrationReport> {
    first_party_editor_plugin_registrations_for_manifest_impl(target_mode, manifest)
}

#[cfg(all(
    feature = "target-editor-host",
    feature = "first-party-navigation-editor-plugin"
))]
fn first_party_editor_plugin_registrations_for_manifest_impl(
    target_mode: zircon_runtime::core::framework::platform::RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<EditorPluginRegistrationReport> {
    zircon_first_party_editor_catalog::first_party_editor_plugin_registrations_for_manifest(
        target_mode,
        manifest,
    )
}

#[cfg(all(
    feature = "target-editor-host",
    not(feature = "first-party-navigation-editor-plugin")
))]
fn first_party_editor_plugin_registrations_for_manifest_impl(
    target_mode: zircon_runtime::core::framework::platform::RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<EditorPluginRegistrationReport> {
    let _ = (target_mode, manifest);
    Vec::new()
}

#[cfg(all(
    test,
    feature = "target-editor-host",
    feature = "first-party-navigation-editor-plugin"
))]
mod tests {
    use zircon_runtime::builtin::RuntimePluginId;
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

    use super::first_party_editor_plugin_registrations_for_manifest;

    #[test]
    fn app_composition_projects_selected_navigation_editor_provider() {
        let manifest = ProjectPluginManifest {
            selections: vec![ProjectPluginSelection::runtime_plugin(
                RuntimePluginId::Navigation,
                true,
                false,
            )
            .with_target_modes([RuntimeTargetMode::EditorHost])],
        };

        let registrations = first_party_editor_plugin_registrations_for_manifest(
            RuntimeTargetMode::EditorHost,
            &manifest,
        );

        assert_eq!(registrations.len(), 1);
        assert_eq!(registrations[0].package_manifest.id, "navigation");
        assert_eq!(
            registrations[0].runtime_event_consumers.manifests().len(),
            1
        );
    }
}
