use super::ModulePluginAction;

pub(in crate::ui::retained_host::app::module_plugin_actions) fn parse_module_plugin_action(
    action_id: &str,
) -> Option<ModulePluginAction<'_>> {
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
