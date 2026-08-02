use super::super::RetainedEditorHost;
use super::action_ids::{parse_module_plugin_action, ModulePluginAction};
use super::live_host::ModulePluginLiveHostCommand;

mod live_actions;
mod manifest;
mod project_actions;

use manifest::load_module_plugin_project_manifest;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_module_plugin_action(
        &mut self,
        action_id: &str,
    ) {
        let Some(action) = parse_module_plugin_action(action_id) else {
            self.set_status_line(format!("Unknown module plugin action {action_id}"));
            return;
        };
        let project_path = self.runtime.editor_snapshot().project_path;
        let result = load_module_plugin_project_manifest(&project_path).and_then(|mut context| {
            let outcome = match action {
                ModulePluginAction::Unload { plugin_id } => self
                    .dispatch_module_plugin_live_host_action(
                        plugin_id,
                        ModulePluginLiveHostCommand::Unload,
                        &context.project_root,
                    )?,
                ModulePluginAction::HotReload { plugin_id } => self
                    .dispatch_module_plugin_live_host_action(
                        plugin_id,
                        ModulePluginLiveHostCommand::HotReload,
                        &context.project_root,
                    )?,
                action => self.dispatch_module_plugin_project_manifest_action(
                    &context.project_root,
                    &mut context.manifest,
                    action,
                )?,
            };
            context.save()?;
            Ok(outcome)
        });
        match result {
            Ok(message) => {
                self.set_status_line(message);
                self.mark_layout_dirty();
            }
            Err(error) => self.set_status_line(format!("Plugin action failed: {error}")),
        }
    }
}
