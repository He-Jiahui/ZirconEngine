use zircon_editor::EditorPluginRegistrationReport;
use zircon_runtime::builtin::RuntimePluginId;
use zircon_runtime::core::framework::platform::RuntimeTargetMode;
use zircon_runtime::core::framework::project::{
    resolve_plugin_selections, PluginSelectionResolutionReport, ProjectPluginManifest,
};

type EditorPluginRegistrationProvider = fn() -> EditorPluginRegistrationReport;

pub fn first_party_editor_plugin_registrations_for_manifest(
    target_mode: RuntimeTargetMode,
    manifest: &ProjectPluginManifest,
) -> PluginSelectionResolutionReport<EditorPluginRegistrationReport> {
    if target_mode != RuntimeTargetMode::EditorHost {
        return resolve_plugin_selections(target_mode, manifest, |_| None);
    }
    resolve_plugin_selections(target_mode, manifest, |plugin_id| {
        first_party_editor_registration_provider(plugin_id).map(|provider| provider())
    })
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
