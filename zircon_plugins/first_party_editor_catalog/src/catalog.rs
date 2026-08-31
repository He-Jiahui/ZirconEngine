use std::collections::HashSet;

use zircon_editor::EditorPluginRegistrationReport;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;

type EditorPluginRegistrationProvider = fn() -> EditorPluginRegistrationReport;

pub fn first_party_editor_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> Vec<EditorPluginRegistrationReport> {
    if target_mode != RuntimeTargetMode::EditorHost {
        return Vec::new();
    }

    let mut seen = HashSet::with_capacity(manifest.selections.len());
    let mut registrations = Vec::with_capacity(manifest.selections.len());
    for selection in manifest.enabled_for_target(target_mode) {
        let Some(plugin_id) = RuntimePluginId::parse_key(&selection.id) else {
            continue;
        };
        let Some(provider) = first_party_editor_registration_provider(&plugin_id) else {
            continue;
        };
        if !seen.insert(plugin_id) {
            continue;
        }
        registrations.push(provider());
    }
    registrations
}

pub fn first_party_registration_for_editor_plugin(
    plugin_id: RuntimePluginId,
) -> Option<EditorPluginRegistrationReport> {
    first_party_editor_registration_provider(&plugin_id).map(|provider| provider())
}

fn first_party_editor_registration_provider(
    _plugin_id: &RuntimePluginId,
) -> Option<EditorPluginRegistrationProvider> {
    // @cargo-zircon:editor-registration-begin
    #[cfg(feature = "navigation-editor-plugin")]
    if *_plugin_id == RuntimePluginId::Navigation {
        return Some(zircon_plugin_navigation_editor::plugin_registration);
    }
    #[cfg(feature = "neural-editor-plugin")]
    if _plugin_id.key() == "neural" {
        return Some(zircon_plugin_neural_editor::plugin_registration);
    }
    // @cargo-zircon:editor-registration-end
    None
}
