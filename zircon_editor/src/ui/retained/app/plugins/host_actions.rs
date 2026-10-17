use super::super::RetainedEditorHost;
use super::action_ids::{parse_module_plugin_action, ModulePluginAction};
use super::live_host::ModulePluginLiveHostCommand;

#[path = "host_actions/live_actions.rs"]
mod live_actions;
#[path = "host_actions/manifest.rs"]
mod manifest;
#[path = "host_actions/project_actions.rs"]
mod project_actions;

use manifest::load_module_plugin_project_manifest;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_module_plugin_action(
        &mut self,
        action_id: &str,
    ) {
        if let Some(action) = action_id.strip_prefix("workbench.plugin.enablement.") {
            match self.dispatch_module_plugin_enablement_recovery(action) {
                Ok(message) => {
                    self.set_status_line(message);
                    self.mark_layout_dirty();
                }
                Err(error) => self.set_status_line(format!("Plugin recovery failed: {error}")),
            }
            return;
        }
        let Some(action) = parse_module_plugin_action(action_id) else {
            self.set_status_line(format!("Unknown module plugin action {action_id}"));
            return;
        };
        let project_path = (match self.runtime.editor_snapshot() {
            Ok(value) => value,
            Err(error) => {
                self.set_status_line(error.to_string());
                return;
            }
        })
        .project_path;
        let result = match action {
            ModulePluginAction::SetEnabled { plugin_id, enabled } => {
                crate::ui::workbench::project::project_root_path(&project_path)
                    .map_err(|e| e.to_string())
                    .and_then(|root| self.set_project_plugin_enabled(&root, plugin_id, enabled))
            }
            ModulePluginAction::Unload { plugin_id } => {
                let project_root = crate::ui::workbench::project::project_root_path(&project_path)
                    .map_err(|error| error.to_string());
                project_root.and_then(|root| {
                    self.dispatch_module_plugin_live_host_action(
                        plugin_id,
                        ModulePluginLiveHostCommand::Unload,
                        &root,
                    )
                })
            }
            ModulePluginAction::HotReload { plugin_id } => {
                let project_root = crate::ui::workbench::project::project_root_path(&project_path)
                    .map_err(|error| error.to_string());
                project_root.and_then(|root| {
                    self.dispatch_module_plugin_live_host_action(
                        plugin_id,
                        ModulePluginLiveHostCommand::HotReload,
                        &root,
                    )
                })
            }
            action => load_module_plugin_project_manifest(&project_path).and_then(|mut context| {
                let outcome = self.dispatch_module_plugin_project_manifest_action(
                    &context.project_root,
                    &mut context.manifest,
                    action,
                )?;
                context.save()?;
                Ok(outcome)
            }),
        };
        match result {
            Ok(message) => {
                self.set_status_line(message);
                self.mark_layout_dirty();
            }
            Err(error) => self.set_status_line(format!("Plugin action failed: {error}")),
        }
    }
}

impl RetainedEditorHost {
    fn dispatch_module_plugin_enablement_recovery(
        &mut self,
        action: &str,
    ) -> Result<String, String> {
        use crate::ui::host::PluginEnablementRecoveryCommand as Command;
        let (verb, operation_id) = action
            .split_once(':')
            .ok_or("invalid plugin recovery action")?;
        let manager = self.editor_manager.enter().map_err(|e| e.to_string())?;
        let status = manager.published_plugin_status_report();
        let operation = status
            .enablement
            .as_ref()
            .filter(|p| p.operation_id == operation_id)
            .ok_or("plugin recovery action is stale; refresh the panel")?;
        if verb == "open" {
            manager
                .open_project_and_remember(&operation.project_root)
                .map_err(|e| e.to_string())?;
            return Ok("Project opened after explicit plugin change resolution".into());
        }
        let command = match verb {
            "recover" => Command::RecoverFiles {
                admission: operation
                    .recovery_admission
                    .clone()
                    .ok_or("writer admission is unavailable")?,
            },
            "retry" => Command::Retry {
                operation_id: operation_id.into(),
                expected_revision: operation
                    .committed_revision
                    .clone()
                    .ok_or("committed revision is unavailable")?,
            },
            "revert" => Command::Revert {
                operation_id: operation_id.into(),
                expected_revision: operation
                    .committed_revision
                    .clone()
                    .ok_or("committed revision is unavailable")?,
                admission: operation.recovery_admission.clone(),
            },
            _ => return Err("unknown plugin recovery command".into()),
        };
        match manager.execute_plugin_enablement_recovery(&operation.project_root, command) {
            Ok(report) | Err(report) => Ok(report.status_message()),
        }
    }
}
