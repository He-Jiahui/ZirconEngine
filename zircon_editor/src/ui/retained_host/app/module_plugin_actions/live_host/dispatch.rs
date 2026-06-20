use super::types::{
    ModulePluginLiveHostBackend, ModulePluginLiveHostCommand, ModulePluginLiveHostOutcome,
    ModulePluginLiveHostRequest,
};

pub(in crate::ui::retained_host::app::module_plugin_actions) fn dispatch_live_plugin_backend_action(
    backend: &dyn ModulePluginLiveHostBackend,
    plugin_id: &str,
    command: ModulePluginLiveHostCommand,
    project_root: &std::path::Path,
) -> Result<ModulePluginLiveHostOutcome, String> {
    if plugin_id.trim().is_empty() {
        return Err("plugin id is empty".to_string());
    }
    backend.execute(ModulePluginLiveHostRequest {
        plugin_id,
        command,
        project_root,
    })
}

pub(in crate::ui::retained_host::app::module_plugin_actions) fn live_plugin_backend_success_message(
    outcome: &ModulePluginLiveHostOutcome,
) -> String {
    if outcome.diagnostics.is_empty() {
        return format!(
            "Plugin {} {}",
            outcome.plugin_id,
            outcome.command.past_tense()
        );
    }
    format!(
        "Plugin {} {}: {}",
        outcome.plugin_id,
        outcome.command.past_tense(),
        outcome.diagnostics.join("; ")
    )
}
