use zircon_runtime::asset::project::ProjectManifest;

pub(in crate::ui::retained_host::app::module_plugin_actions) fn current_native_aware_project_selection(
    editor_manager: &crate::ui::host::EditorManager,
    project_root: &std::path::Path,
    manifest: &ProjectManifest,
    plugin_id: &str,
) -> Result<zircon_runtime::core::framework::project::ProjectPluginSelection, String> {
    editor_manager
        .complete_native_aware_project_plugin_manifest(project_root, manifest)
        .plugins
        .selections
        .into_iter()
        .find(|selection| selection.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} is not registered in builtin or native catalog"))
}
