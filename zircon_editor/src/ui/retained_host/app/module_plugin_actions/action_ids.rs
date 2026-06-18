#[derive(Debug, PartialEq, Eq)]
pub(super) enum ModulePluginAction<'a> {
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

pub(super) fn parse_module_plugin_action(action_id: &str) -> Option<ModulePluginAction<'_>> {
    action_id
        .strip_prefix("workbench.plugin.enable.")
        .map(|plugin_id| ModulePluginAction::SetEnabled {
            plugin_id,
            enabled: true,
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.disable.")
                .map(|plugin_id| ModulePluginAction::SetEnabled {
                    plugin_id,
                    enabled: false,
                })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.packaging.next.")
                .map(|plugin_id| ModulePluginAction::CyclePackaging { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.target_modes.next.")
                .map(|plugin_id| ModulePluginAction::CycleTargetModes { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.feature.enable_dependencies.")
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
                .strip_prefix("workbench.plugin.feature.enable.")
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
                .strip_prefix("workbench.plugin.feature.disable.")
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
                .strip_prefix("workbench.plugin.unload.")
                .map(|plugin_id| ModulePluginAction::Unload { plugin_id })
        })
        .or_else(|| {
            action_id
                .strip_prefix("workbench.plugin.hot_reload.")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_plugin_actions_parse_enable_policy_and_target_mode_updates() {
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.enable.physics"),
            Some(ModulePluginAction::SetEnabled {
                plugin_id: "physics",
                enabled: true,
            })
        );
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.disable.physics"),
            Some(ModulePluginAction::SetEnabled {
                plugin_id: "physics",
                enabled: false,
            })
        );
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.packaging.next.physics"),
            Some(ModulePluginAction::CyclePackaging {
                plugin_id: "physics"
            })
        );
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.target_modes.next.physics"),
            Some(ModulePluginAction::CycleTargetModes {
                plugin_id: "physics"
            })
        );
        assert_eq!(
            parse_module_plugin_action(
                "workbench.plugin.feature.enable.sound.sound.timeline_animation_track"
            ),
            Some(ModulePluginAction::SetFeatureEnabled {
                plugin_id: "sound",
                feature_id: "sound.timeline_animation_track",
                enabled: true,
            })
        );
        assert_eq!(
            parse_module_plugin_action(
                "workbench.plugin.feature.enable_dependencies.sound.sound.timeline_animation_track"
            ),
            Some(ModulePluginAction::EnableFeatureDependencies {
                plugin_id: "sound",
                feature_id: "sound.timeline_animation_track",
            })
        );
        assert_eq!(
            parse_module_plugin_action(
                "workbench.plugin.feature.disable.sound.sound.timeline_animation_track"
            ),
            Some(ModulePluginAction::SetFeatureEnabled {
                plugin_id: "sound",
                feature_id: "sound.timeline_animation_track",
                enabled: false,
            })
        );
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.unload.physics"),
            Some(ModulePluginAction::Unload {
                plugin_id: "physics"
            })
        );
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.hot_reload.physics"),
            Some(ModulePluginAction::HotReload {
                plugin_id: "physics"
            })
        );
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.unknown.physics"),
            None
        );
        assert_eq!(
            parse_module_plugin_action("workbench.plugin.feature.enable.sound"),
            None
        );
    }
}
