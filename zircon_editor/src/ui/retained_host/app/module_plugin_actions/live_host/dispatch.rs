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
    const PREFIX: &str = "Plugin ";
    const DIAGNOSTIC_PREFIX: &str = ": ";
    const DIAGNOSTIC_SEPARATOR: &str = "; ";

    let action = outcome.command.past_tense();
    let diagnostic_bytes = outcome
        .diagnostics
        .iter()
        .map(String::len)
        .fold(0usize, usize::saturating_add);
    let separator_bytes = outcome
        .diagnostics
        .len()
        .saturating_sub(1)
        .saturating_mul(DIAGNOSTIC_SEPARATOR.len());
    let capacity = PREFIX
        .len()
        .saturating_add(outcome.plugin_id.len())
        .saturating_add(1)
        .saturating_add(action.len())
        .saturating_add(
            (!outcome.diagnostics.is_empty())
                .then_some(DIAGNOSTIC_PREFIX.len() + diagnostic_bytes + separator_bytes)
                .unwrap_or_default(),
        );
    let mut message = String::with_capacity(capacity);
    message.push_str(PREFIX);
    message.push_str(&outcome.plugin_id);
    message.push(' ');
    message.push_str(action);
    if outcome.diagnostics.is_empty() {
        return message;
    }
    message.push_str(DIAGNOSTIC_PREFIX);
    for (index, diagnostic) in outcome.diagnostics.iter().enumerate() {
        if index > 0 {
            message.push_str(DIAGNOSTIC_SEPARATOR);
        }
        message.push_str(diagnostic);
    }
    message
}

#[cfg(test)]
#[path = "dispatch/single_allocation_success_message_tests.rs"]
mod single_allocation_success_message_tests;
