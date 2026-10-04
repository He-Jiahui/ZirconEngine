#[cfg(feature = "first-party-editor-catalog")]
use zircon_editor::EditorPluginRegistrationReport;
#[cfg(feature = "first-party-editor-catalog")]
use zircon_runtime::core::framework::project::PluginSelectionResolutionReport;
#[cfg(feature = "first-party-editor-catalog")]
use zircon_runtime::core::framework::project::ProjectPluginManifest;

#[cfg(feature = "first-party-editor-catalog")]
use super::ResolvedProductHostConfig;

#[cfg(feature = "first-party-editor-catalog")]
pub fn first_party_editor_plugin_registrations_for_config(
    config: &ResolvedProductHostConfig,
) -> PluginSelectionResolutionReport<EditorPluginRegistrationReport> {
    let manifest = config
        .project_plugin_manifest()
        .cloned()
        .unwrap_or_default();
    first_party_editor_plugin_registrations_for_manifest(config.target_mode(), &manifest)
}

#[cfg(feature = "first-party-editor-catalog")]
pub fn first_party_editor_plugin_registrations_for_manifest(
    target_mode: zircon_runtime::core::framework::platform::RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> PluginSelectionResolutionReport<EditorPluginRegistrationReport> {
    first_party_editor_plugin_registrations_for_manifest_impl(target_mode, manifest)
}

#[cfg(feature = "first-party-editor-catalog")]
fn first_party_editor_plugin_registrations_for_manifest_impl(
    target_mode: zircon_runtime::core::framework::platform::RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> PluginSelectionResolutionReport<EditorPluginRegistrationReport> {
    zircon_first_party_editor_catalog::first_party_editor_plugin_registrations_for_manifest(
        target_mode,
        manifest,
    )
}

#[cfg(all(test, feature = "first-party-editor-catalog"))]
#[path = "tests/first_party_editor_plugins.rs"]
mod tests;
