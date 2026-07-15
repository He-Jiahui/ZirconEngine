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

    let mut seen = HashSet::new();
    manifest
        .enabled_for_target(target_mode)
        .filter_map(|selection| RuntimePluginId::parse_key(&selection.id))
        .filter(|plugin_id| seen.insert(*plugin_id))
        .filter_map(first_party_registration_for_editor_plugin)
        .collect()
}

pub fn first_party_registration_for_editor_plugin(
    plugin_id: RuntimePluginId,
) -> Option<EditorPluginRegistrationReport> {
    match plugin_id {
        #[cfg(feature = "navigation-editor-plugin")]
        RuntimePluginId::Navigation => Some(zircon_plugin_navigation_editor::plugin_registration()),
        _ => None,
    }
}
