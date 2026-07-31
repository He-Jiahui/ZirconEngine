use std::collections::HashSet;

use zircon_editor::EditorPluginRegistrationReport;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::ProjectPluginManifest;

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
        if !seen.insert(plugin_id.clone()) {
            continue;
        }
        let Some(registration) = first_party_registration_for_editor_plugin(plugin_id) else {
            continue;
        };
        registrations.push(registration);
    }
    registrations
}

pub fn first_party_registration_for_editor_plugin(
    _plugin_id: RuntimePluginId,
) -> Option<EditorPluginRegistrationReport> {
    #[cfg(feature = "navigation-editor-plugin")]
    if _plugin_id == RuntimePluginId::Navigation {
        return Some(zircon_plugin_navigation_editor::plugin_registration());
    }
    None
}
