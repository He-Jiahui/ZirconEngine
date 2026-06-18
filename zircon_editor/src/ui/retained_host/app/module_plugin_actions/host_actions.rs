use super::super::RetainedEditorHost;
use super::action_ids::{parse_module_plugin_action, ModulePluginAction};
use super::live_host::{
    dispatch_live_plugin_backend_action, live_plugin_backend_success_message,
    ModulePluginLiveHostCommand,
};
use super::project_policy::{
    current_native_aware_project_selection, feature_dependency_enable_message, next_packaging,
    next_target_modes, packaging_status_label, target_modes_status_label,
};
use zircon_runtime::asset::project::ProjectManifest;

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
        let result = crate::ui::workbench::project::project_root_path(&project_path)
            .map_err(|error| error.to_string())
            .and_then(|project_root| {
                let manifest_path = project_root.join("zircon-project.toml");
                let mut manifest =
                    ProjectManifest::load(&manifest_path).map_err(|error| error.to_string())?;
                let outcome = match action {
                    ModulePluginAction::SetEnabled { plugin_id, enabled } => {
                        let report = self
                            .editor_manager
                            .set_native_aware_project_plugin_enabled(
                                &project_root,
                                &mut manifest,
                                plugin_id,
                                enabled,
                            )?;
                        let state = if report.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        format!("Plugin {} {state}", report.plugin_id)
                    }
                    ModulePluginAction::CyclePackaging { plugin_id } => {
                        let selection = current_native_aware_project_selection(
                            &self.editor_manager,
                            &project_root,
                            &manifest,
                            plugin_id,
                        )?;
                        let packaging = next_packaging(selection.packaging);
                        let report = self
                            .editor_manager
                            .set_native_aware_project_plugin_packaging(
                                &project_root,
                                &mut manifest,
                                plugin_id,
                                packaging,
                            )?;
                        format!(
                            "Plugin {} packaging set to {}",
                            report.plugin_id,
                            packaging_status_label(report.project_selection.packaging)
                        )
                    }
                    ModulePluginAction::CycleTargetModes { plugin_id } => {
                        let selection = current_native_aware_project_selection(
                            &self.editor_manager,
                            &project_root,
                            &manifest,
                            plugin_id,
                        )?;
                        let target_modes = next_target_modes(&selection.target_modes);
                        let report = self
                            .editor_manager
                            .set_native_aware_project_plugin_target_modes(
                                &project_root,
                                &mut manifest,
                                plugin_id,
                                target_modes,
                            )?;
                        format!(
                            "Plugin {} target modes set to {}",
                            report.plugin_id,
                            target_modes_status_label(&report.project_selection.target_modes)
                        )
                    }
                    ModulePluginAction::SetFeatureEnabled {
                        plugin_id,
                        feature_id,
                        enabled,
                    } => {
                        let report = self
                            .editor_manager
                            .set_native_aware_project_plugin_feature_enabled(
                                &project_root,
                                &mut manifest,
                                plugin_id,
                                feature_id,
                                enabled,
                            )?;
                        let state = if report.enabled {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        format!(
                            "Feature {} on plugin {} {state}",
                            report.feature_id, report.plugin_id
                        )
                    }
                    ModulePluginAction::EnableFeatureDependencies {
                        plugin_id,
                        feature_id,
                    } => {
                        let report = self
                            .editor_manager
                            .enable_native_aware_project_plugin_feature_dependencies(
                                &project_root,
                                &mut manifest,
                                plugin_id,
                                feature_id,
                            )?;
                        feature_dependency_enable_message(&report)
                    }
                    ModulePluginAction::Unload { plugin_id } => {
                        let outcome = dispatch_live_plugin_backend_action(
                            self.module_plugin_live_host_backend.as_ref(),
                            plugin_id,
                            ModulePluginLiveHostCommand::Unload,
                            &project_root,
                        )?;
                        live_plugin_backend_success_message(&outcome)
                    }
                    ModulePluginAction::HotReload { plugin_id } => {
                        let outcome = dispatch_live_plugin_backend_action(
                            self.module_plugin_live_host_backend.as_ref(),
                            plugin_id,
                            ModulePluginLiveHostCommand::HotReload,
                            &project_root,
                        )?;
                        live_plugin_backend_success_message(&outcome)
                    }
                };
                manifest
                    .save(&manifest_path)
                    .map_err(|error| error.to_string())?;
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
