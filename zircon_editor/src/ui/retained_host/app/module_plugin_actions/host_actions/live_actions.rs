use std::path::Path;

use super::super::super::RetainedEditorHost;
use super::super::live_host::{
    dispatch_live_plugin_backend_action, live_plugin_backend_success_message,
    ModulePluginLiveHostCommand, ModulePluginLiveHostCompletion, ModulePluginLiveHostProject,
};

impl RetainedEditorHost {
    pub(super) fn dispatch_module_plugin_live_host_action(
        &mut self,
        plugin_id: &str,
        command: ModulePluginLiveHostCommand,
        project_root: &Path,
    ) -> Result<String, String> {
        let project = self
            .active_module_plugin_live_project()
            .ok_or_else(|| "live plugin action requires an active project session".to_string())?;
        if project.root() != project_root {
            return Err("live plugin action targets a stale project".to_string());
        }
        self.dispatch_module_plugin_live_host_action_with_operation(
            plugin_id,
            command,
            &project,
            command.label(),
        )
    }

    fn dispatch_module_plugin_live_host_action_with_operation(
        &mut self,
        plugin_id: &str,
        command: ModulePluginLiveHostCommand,
        project: &ModulePluginLiveHostProject,
        operation: &str,
    ) -> Result<String, String> {
        let outcome = self
            .editor_manager
            .execute_native_live_plugin_for_session(&project.session_target(), || {
                self.runtime
                    .execute_native_live_action_without_active_contribution(plugin_id, || {
                        dispatch_live_plugin_backend_action(
                            self.module_plugin_live_host_backend.as_ref(),
                            plugin_id,
                            command,
                            project,
                        )
                    })
            })
            .map(|outcome| live_plugin_backend_success_message(&outcome));
        if self.active_module_plugin_live_project().as_ref() != Some(project) {
            return Err("project session changed during the live plugin action".to_string());
        }
        let live_loaded = self
            .module_plugin_live_host_backend
            .loaded_editor_plugin_ids()
            .map(|ids| ids.iter().any(|id| id == plugin_id));
        let outcome = reconcile_live_plugin_result(command, plugin_id, outcome, &live_loaded);
        if !self.editor_manager.publish_native_live_plugin_status(
            &project.session_target(),
            plugin_id,
            operation,
            &outcome,
            &live_loaded,
        )? {
            return Err("active project plugin status is unavailable".to_string());
        }
        self.mark_layout_dirty();
        outcome
    }

    pub(in crate::ui::retained_host::app) fn poll_module_plugin_development_watches(
        &mut self,
    ) -> Option<std::time::Instant> {
        let project = self.active_module_plugin_live_project();
        let (diagnostics, completions, deadline) = self
            .module_plugin_live_host_backend
            .poll_development_watches(project.as_ref())
            .into_parts();
        for diagnostic in diagnostics {
            self.set_status_line(diagnostic);
        }
        for completion in completions {
            let Some(execution) = execute_matching_watch_completion(
                &completion,
                project.as_ref(),
                |plugin_id, expected_project| {
                    self.dispatch_module_plugin_live_host_action_with_operation(
                        plugin_id,
                        ModulePluginLiveHostCommand::HotReload,
                        expected_project,
                        "watch_hot_reload",
                    )
                },
            ) else {
                continue;
            };
            if completion.result.is_ok() {
                if self.active_module_plugin_live_project().as_ref() == Some(&completion.project) {
                    match execution {
                        Ok(message) => self.set_status_line(message),
                        Err(error) => self.set_status_line(error),
                    }
                }
                continue;
            }
            let live_loaded = self
                .module_plugin_live_host_backend
                .loaded_editor_plugin_ids()
                .map(|ids| ids.iter().any(|id| id == &completion.plugin_id));
            let outcome = reconcile_live_plugin_result(
                ModulePluginLiveHostCommand::HotReload,
                &completion.plugin_id,
                execution,
                &live_loaded,
            );
            let publication = self.editor_manager.publish_native_live_plugin_status(
                &completion.project.session_target(),
                &completion.plugin_id,
                "watch_hot_reload",
                &outcome,
                &live_loaded,
            );
            match publication {
                Ok(true) => {}
                Ok(false) => continue,
                Err(error) => {
                    self.set_status_line(error);
                    continue;
                }
            }
            match outcome {
                Ok(message) => self.set_status_line(message),
                Err(error) => self.set_status_line(error),
            }
            self.mark_layout_dirty();
        }
        deadline
    }

    fn active_module_plugin_live_project(&self) -> Option<ModulePluginLiveHostProject> {
        self.editor_manager
            .active_project_session_focus_target()
            .map(|(root, instance_id, generation)| {
                ModulePluginLiveHostProject::new(root, instance_id, generation)
            })
    }
}

fn execute_matching_watch_completion(
    completion: &ModulePluginLiveHostCompletion,
    project: Option<&ModulePluginLiveHostProject>,
    execute: impl FnOnce(&str, &ModulePluginLiveHostProject) -> Result<String, String>,
) -> Option<Result<String, String>> {
    if !completion.matches_project(project) {
        return None;
    }
    Some(match &completion.result {
        Ok(_) => execute(&completion.plugin_id, &completion.project),
        Err(error) => Err(error.clone()),
    })
}

fn reconcile_live_plugin_result(
    command: ModulePluginLiveHostCommand,
    plugin_id: &str,
    outcome: Result<String, String>,
    live_loaded: &Result<bool, String>,
) -> Result<String, String> {
    if outcome.is_err() {
        return outcome;
    }
    match (command, live_loaded) {
        (ModulePluginLiveHostCommand::Unload, Ok(true)) => Err(format!(
            "plugin `{plugin_id}` unload returned success, but the editor live host still has it loaded"
        )),
        (ModulePluginLiveHostCommand::HotReload, Ok(false)) => Err(format!(
            "plugin `{plugin_id}` hot reload returned success, but the editor live host has no loaded plugin"
        )),
        (_, Err(error)) => Err(format!(
            "plugin `{plugin_id}` {} could not verify the editor live state: {error}",
            command.label()
        )),
        _ => outcome,
    }
}

#[cfg(test)]
#[path = "tests/live_actions.rs"]
mod tests;
