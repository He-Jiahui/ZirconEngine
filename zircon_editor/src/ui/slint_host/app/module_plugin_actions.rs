use super::*;
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{ExportPackagingStrategy, RuntimeTargetMode};

impl SlintEditorHost {
    pub(super) fn dispatch_module_plugin_action(&mut self, action_id: &str) {
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
                };
                manifest
                    .save(&manifest_path)
                    .map_err(|error| error.to_string())?;
                Ok(outcome)
            });
        match result {
            Ok(message) => {
                self.set_status_line(message);
                self.layout_dirty = true;
                self.presentation_dirty = true;
            }
            Err(error) => self.set_status_line(format!("Plugin action failed: {error}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ModulePluginAction<'a> {
    SetEnabled { plugin_id: &'a str, enabled: bool },
    CyclePackaging { plugin_id: &'a str },
    CycleTargetModes { plugin_id: &'a str },
}

fn parse_module_plugin_action(action_id: &str) -> Option<ModulePluginAction<'_>> {
    action_id
        .strip_prefix("Plugin.Enable.")
        .map(|plugin_id| ModulePluginAction::SetEnabled {
            plugin_id,
            enabled: true,
        })
        .or_else(|| {
            action_id.strip_prefix("Plugin.Disable.").map(|plugin_id| {
                ModulePluginAction::SetEnabled {
                    plugin_id,
                    enabled: false,
                }
            })
        })
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.Packaging.Next.")
                .map(|plugin_id| ModulePluginAction::CyclePackaging { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.TargetModes.Next.")
                .map(|plugin_id| ModulePluginAction::CycleTargetModes { plugin_id })
        })
}

fn current_native_aware_project_selection(
    editor_manager: &crate::ui::host::EditorManager,
    project_root: &std::path::Path,
    manifest: &ProjectManifest,
    plugin_id: &str,
) -> Result<zircon_runtime::ProjectPluginSelection, String> {
    editor_manager
        .complete_native_aware_project_plugin_manifest(project_root, manifest)
        .plugins
        .selections
        .into_iter()
        .find(|selection| selection.id == plugin_id)
        .ok_or_else(|| format!("plugin {plugin_id} is not registered in builtin or native catalog"))
}

fn next_packaging(packaging: ExportPackagingStrategy) -> ExportPackagingStrategy {
    match packaging {
        ExportPackagingStrategy::LibraryEmbed => ExportPackagingStrategy::NativeDynamic,
        ExportPackagingStrategy::NativeDynamic => ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::SourceTemplate => ExportPackagingStrategy::LibraryEmbed,
    }
}

fn next_target_modes(target_modes: &[RuntimeTargetMode]) -> Vec<RuntimeTargetMode> {
    match target_modes {
        [] => vec![RuntimeTargetMode::ClientRuntime],
        [RuntimeTargetMode::ClientRuntime] => vec![RuntimeTargetMode::ServerRuntime],
        [RuntimeTargetMode::ServerRuntime] => vec![RuntimeTargetMode::EditorHost],
        [RuntimeTargetMode::EditorHost] => {
            vec![
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ]
        }
        [RuntimeTargetMode::ClientRuntime, RuntimeTargetMode::EditorHost] => Vec::new(),
        _ => Vec::new(),
    }
}

fn packaging_status_label(packaging: ExportPackagingStrategy) -> &'static str {
    match packaging {
        ExportPackagingStrategy::SourceTemplate => "source-template",
        ExportPackagingStrategy::LibraryEmbed => "library-embed",
        ExportPackagingStrategy::NativeDynamic => "native-dynamic",
    }
}

fn target_modes_status_label(target_modes: &[RuntimeTargetMode]) -> String {
    if target_modes.is_empty() {
        return "all".to_string();
    }
    target_modes
        .iter()
        .map(|mode| match mode {
            RuntimeTargetMode::ClientRuntime => "client",
            RuntimeTargetMode::ServerRuntime => "server",
            RuntimeTargetMode::EditorHost => "editor",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_plugin_actions_parse_enable_policy_and_target_mode_updates() {
        assert_eq!(
            parse_module_plugin_action("Plugin.Enable.physics"),
            Some(ModulePluginAction::SetEnabled {
                plugin_id: "physics",
                enabled: true,
            })
        );
        assert_eq!(
            parse_module_plugin_action("Plugin.Disable.physics"),
            Some(ModulePluginAction::SetEnabled {
                plugin_id: "physics",
                enabled: false,
            })
        );
        assert_eq!(
            parse_module_plugin_action("Plugin.Packaging.Next.physics"),
            Some(ModulePluginAction::CyclePackaging {
                plugin_id: "physics"
            })
        );
        assert_eq!(
            parse_module_plugin_action("Plugin.TargetModes.Next.physics"),
            Some(ModulePluginAction::CycleTargetModes {
                plugin_id: "physics"
            })
        );
        assert_eq!(parse_module_plugin_action("Plugin.Unknown.physics"), None);
    }

    #[test]
    fn module_plugin_policy_cycles_are_deterministic() {
        assert_eq!(
            next_packaging(ExportPackagingStrategy::LibraryEmbed),
            ExportPackagingStrategy::NativeDynamic
        );
        assert_eq!(
            next_packaging(ExportPackagingStrategy::NativeDynamic),
            ExportPackagingStrategy::SourceTemplate
        );
        assert_eq!(
            next_packaging(ExportPackagingStrategy::SourceTemplate),
            ExportPackagingStrategy::LibraryEmbed
        );

        assert_eq!(
            next_target_modes(&[]),
            vec![RuntimeTargetMode::ClientRuntime]
        );
        assert_eq!(
            next_target_modes(&[RuntimeTargetMode::ClientRuntime]),
            vec![RuntimeTargetMode::ServerRuntime]
        );
        assert_eq!(
            next_target_modes(&[RuntimeTargetMode::ServerRuntime]),
            vec![RuntimeTargetMode::EditorHost]
        );
        assert_eq!(
            next_target_modes(&[RuntimeTargetMode::EditorHost]),
            vec![
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ]
        );
        assert_eq!(
            next_target_modes(&[
                RuntimeTargetMode::ClientRuntime,
                RuntimeTargetMode::EditorHost,
            ]),
            Vec::<RuntimeTargetMode>::new()
        );
    }
}
