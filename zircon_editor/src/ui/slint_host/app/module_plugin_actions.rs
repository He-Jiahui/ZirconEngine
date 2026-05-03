use super::*;
use zircon_runtime::asset::project::ProjectManifest;
use zircon_runtime::{
    plugin::ExportPackagingStrategy, plugin::NativePluginLiveHost, RuntimeTargetMode,
};

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
                self.layout_dirty = true;
                self.presentation_dirty = true;
            }
            Err(error) => self.set_status_line(format!("Plugin action failed: {error}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum ModulePluginAction<'a> {
    SetEnabled {
        plugin_id: &'a str,
        enabled: bool,
    },
    CyclePackaging {
        plugin_id: &'a str,
    },
    CycleTargetModes {
        plugin_id: &'a str,
    },
    SetFeatureEnabled {
        plugin_id: &'a str,
        feature_id: &'a str,
        enabled: bool,
    },
    EnableFeatureDependencies {
        plugin_id: &'a str,
        feature_id: &'a str,
    },
    Unload {
        plugin_id: &'a str,
    },
    HotReload {
        plugin_id: &'a str,
    },
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
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.Feature.EnableDependencies.")
                .and_then(parse_module_plugin_feature_action)
                .map(
                    |(plugin_id, feature_id)| ModulePluginAction::EnableFeatureDependencies {
                        plugin_id,
                        feature_id,
                    },
                )
        })
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.Feature.Enable.")
                .and_then(parse_module_plugin_feature_action)
                .map(
                    |(plugin_id, feature_id)| ModulePluginAction::SetFeatureEnabled {
                        plugin_id,
                        feature_id,
                        enabled: true,
                    },
                )
        })
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.Feature.Disable.")
                .and_then(parse_module_plugin_feature_action)
                .map(
                    |(plugin_id, feature_id)| ModulePluginAction::SetFeatureEnabled {
                        plugin_id,
                        feature_id,
                        enabled: false,
                    },
                )
        })
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.Unload.")
                .map(|plugin_id| ModulePluginAction::Unload { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("Plugin.HotReload.")
                .map(|plugin_id| ModulePluginAction::HotReload { plugin_id })
        })
}

fn parse_module_plugin_feature_action(action: &str) -> Option<(&str, &str)> {
    let (plugin_id, feature_id) = action.split_once('.')?;
    if plugin_id.is_empty() || feature_id.is_empty() {
        return None;
    }
    Some((plugin_id, feature_id))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ModulePluginLiveHostCommand {
    Unload,
    HotReload,
}

impl ModulePluginLiveHostCommand {
    fn label(self) -> &'static str {
        match self {
            Self::Unload => "unload",
            Self::HotReload => "hot reload",
        }
    }

    fn past_tense(self) -> &'static str {
        match self {
            Self::Unload => "unloaded",
            Self::HotReload => "hot reloaded",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ModulePluginLiveHostOutcome {
    pub(super) plugin_id: String,
    pub(super) command: ModulePluginLiveHostCommand,
    pub(super) diagnostics: Vec<String>,
}

pub(super) struct ModulePluginLiveHostRequest<'a> {
    pub(super) plugin_id: &'a str,
    pub(super) command: ModulePluginLiveHostCommand,
    pub(super) project_root: &'a std::path::Path,
}

// This editor-side seam keeps Plugin Manager action parsing stable while the
// runtime/native plugin host grows the real package-handle ownership backend.
pub(super) trait ModulePluginLiveHostBackend {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String>;
}

impl ModulePluginLiveHostBackend for NativePluginLiveHost {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String> {
        let outcome = match request.command {
            ModulePluginLiveHostCommand::Unload => self.unload_editor_plugin(request.plugin_id),
            ModulePluginLiveHostCommand::HotReload => {
                self.hot_reload_editor_plugin(request.project_root, request.plugin_id)
            }
        }?;
        Ok(ModulePluginLiveHostOutcome {
            plugin_id: outcome.plugin_id,
            command: request.command,
            diagnostics: outcome.diagnostics,
        })
    }
}

impl ModulePluginLiveHostBackend for Arc<NativePluginLiveHost> {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String> {
        self.as_ref().execute(request)
    }
}

pub(super) struct UnavailableModulePluginLiveHostBackend;

impl ModulePluginLiveHostBackend for UnavailableModulePluginLiveHostBackend {
    fn execute(
        &self,
        request: ModulePluginLiveHostRequest<'_>,
    ) -> Result<ModulePluginLiveHostOutcome, String> {
        Err(unavailable_live_plugin_backend_message(
            request.plugin_id,
            request.command.label(),
        ))
    }
}

fn dispatch_live_plugin_backend_action(
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

fn live_plugin_backend_success_message(outcome: &ModulePluginLiveHostOutcome) -> String {
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

fn feature_dependency_enable_message(
    report: &crate::ui::host::EditorPluginFeatureSelectionUpdateReport,
) -> String {
    let mut details = Vec::new();
    if !report.enabled_dependency_plugins.is_empty() {
        details.push(format!(
            "plugins {}",
            report.enabled_dependency_plugins.join(", ")
        ));
    }
    if !report.enabled_dependency_features.is_empty() {
        details.push(format!(
            "features {}",
            report.enabled_dependency_features.join(", ")
        ));
    }
    if details.is_empty() {
        let mut message = format!("Feature {} dependencies already enabled", report.feature_id);
        if !report.diagnostics.is_empty() {
            message.push_str(": ");
            message.push_str(&report.diagnostics.join("; "));
        }
        return message;
    }
    let mut message = format!(
        "Feature {} dependencies enabled: {}",
        report.feature_id,
        details.join("; ")
    );
    if !report.diagnostics.is_empty() {
        message.push_str("; ");
        message.push_str(&report.diagnostics.join("; "));
    }
    message
}

fn unavailable_live_plugin_backend_message(plugin_id: &str, action: &str) -> String {
    format!(
        "plugin {plugin_id} {action} is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
    )
}

fn current_native_aware_project_selection(
    editor_manager: &crate::ui::host::EditorManager,
    project_root: &std::path::Path,
    manifest: &ProjectManifest,
    plugin_id: &str,
) -> Result<zircon_runtime::plugin::ProjectPluginSelection, String> {
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
        assert_eq!(
            parse_module_plugin_action(
                "Plugin.Feature.Enable.sound.sound.timeline_animation_track"
            ),
            Some(ModulePluginAction::SetFeatureEnabled {
                plugin_id: "sound",
                feature_id: "sound.timeline_animation_track",
                enabled: true,
            })
        );
        assert_eq!(
            parse_module_plugin_action(
                "Plugin.Feature.EnableDependencies.sound.sound.timeline_animation_track"
            ),
            Some(ModulePluginAction::EnableFeatureDependencies {
                plugin_id: "sound",
                feature_id: "sound.timeline_animation_track",
            })
        );
        assert_eq!(
            parse_module_plugin_action(
                "Plugin.Feature.Disable.sound.sound.timeline_animation_track"
            ),
            Some(ModulePluginAction::SetFeatureEnabled {
                plugin_id: "sound",
                feature_id: "sound.timeline_animation_track",
                enabled: false,
            })
        );
        assert_eq!(
            parse_module_plugin_action("Plugin.Unload.physics"),
            Some(ModulePluginAction::Unload {
                plugin_id: "physics"
            })
        );
        assert_eq!(
            parse_module_plugin_action("Plugin.HotReload.physics"),
            Some(ModulePluginAction::HotReload {
                plugin_id: "physics"
            })
        );
        assert_eq!(parse_module_plugin_action("Plugin.Unknown.physics"), None);
        assert_eq!(
            parse_module_plugin_action("Plugin.Feature.Enable.sound"),
            None
        );
    }

    #[test]
    fn live_backend_actions_report_unavailable_backend_until_runtime_host_is_connected() {
        let project_root = std::path::Path::new("project");
        assert_eq!(
            dispatch_live_plugin_backend_action(
                &UnavailableModulePluginLiveHostBackend,
                "physics",
                ModulePluginLiveHostCommand::Unload,
                project_root,
            )
            .unwrap_err(),
            "plugin physics unload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
        );
        assert_eq!(
            dispatch_live_plugin_backend_action(
                &UnavailableModulePluginLiveHostBackend,
                "physics",
                ModulePluginLiveHostCommand::HotReload,
                project_root,
            )
            .unwrap_err(),
            "plugin physics hot reload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
        );
        assert_eq!(
            dispatch_live_plugin_backend_action(
                &UnavailableModulePluginLiveHostBackend,
                "   ",
                ModulePluginLiveHostCommand::Unload,
                project_root,
            )
            .unwrap_err(),
            "plugin id is empty"
        );
        assert_eq!(
            unavailable_live_plugin_backend_message("physics", "unload"),
            "plugin physics unload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
        );
        assert_eq!(
            unavailable_live_plugin_backend_message("physics", "hot reload"),
            "plugin physics hot reload is reserved in the Plugin Manager UI but the live plugin host backend is not connected yet"
        );
    }

    #[test]
    fn live_backend_dispatch_routes_unload_and_hot_reload_commands() {
        #[derive(Clone, Copy)]
        struct RecordingLiveBackend;

        impl ModulePluginLiveHostBackend for RecordingLiveBackend {
            fn execute(
                &self,
                request: ModulePluginLiveHostRequest<'_>,
            ) -> Result<ModulePluginLiveHostOutcome, String> {
                Ok(ModulePluginLiveHostOutcome {
                    plugin_id: request.plugin_id.to_string(),
                    command: request.command,
                    diagnostics: Vec::new(),
                })
            }
        }

        let project_root = std::path::Path::new("project");
        let unload = dispatch_live_plugin_backend_action(
            &RecordingLiveBackend,
            "physics",
            ModulePluginLiveHostCommand::Unload,
            project_root,
        )
        .expect("unload command should route into live backend");
        assert_eq!(
            unload,
            ModulePluginLiveHostOutcome {
                plugin_id: "physics".to_string(),
                command: ModulePluginLiveHostCommand::Unload,
                diagnostics: Vec::new(),
            }
        );
        assert_eq!(
            live_plugin_backend_success_message(&unload),
            "Plugin physics unloaded"
        );

        let hot_reload = dispatch_live_plugin_backend_action(
            &RecordingLiveBackend,
            "physics",
            ModulePluginLiveHostCommand::HotReload,
            project_root,
        )
        .expect("hot reload command should route into live backend");
        assert_eq!(
            live_plugin_backend_success_message(&ModulePluginLiveHostOutcome {
                diagnostics: vec!["library handle was replaced".to_string()],
                ..hot_reload
            }),
            "Plugin physics hot reloaded: library handle was replaced"
        );
    }

    #[test]
    fn runtime_native_live_backend_reports_missing_editor_package_on_hot_reload() {
        let project_root = std::env::temp_dir().join(format!(
            "zircon-missing-native-live-backend-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be after unix epoch")
                .as_nanos()
        ));
        let error = dispatch_live_plugin_backend_action(
            &NativePluginLiveHost::default(),
            "physics",
            ModulePluginLiveHostCommand::HotReload,
            &project_root,
        )
        .unwrap_err();
        assert!(error.contains("plugin physics hot reload did not load an editor native package"));
        assert!(error.contains("native plugin root does not exist"));
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

    #[test]
    fn feature_dependency_enable_message_includes_enabled_items_and_diagnostics() {
        let report = crate::ui::host::EditorPluginFeatureSelectionUpdateReport {
            plugin_id: "rendering".to_string(),
            feature_id: "rendering.vfx_graph".to_string(),
            enabled: false,
            project_selection: zircon_runtime::plugin::ProjectPluginSelection::runtime_plugin(
                zircon_runtime::RuntimePluginId::Rendering,
                true,
                false,
            ),
            enabled_dependency_plugins: vec!["rendering".to_string(), "particles".to_string()],
            enabled_dependency_features: vec!["rendering.shader_graph".to_string()],
            diagnostics: vec![
                "enabled dependencies for feature rendering.vfx_graph on plugin rendering"
                    .to_string(),
            ],
        };

        assert_eq!(
            feature_dependency_enable_message(&report),
            "Feature rendering.vfx_graph dependencies enabled: plugins rendering, particles; features rendering.shader_graph; enabled dependencies for feature rendering.vfx_graph on plugin rendering"
        );

        let already_ready = crate::ui::host::EditorPluginFeatureSelectionUpdateReport {
            enabled_dependency_plugins: Vec::new(),
            enabled_dependency_features: Vec::new(),
            diagnostics: vec!["dependencies were already enabled".to_string()],
            ..report
        };
        assert_eq!(
            feature_dependency_enable_message(&already_ready),
            "Feature rendering.vfx_graph dependencies already enabled: dependencies were already enabled"
        );
    }
}
