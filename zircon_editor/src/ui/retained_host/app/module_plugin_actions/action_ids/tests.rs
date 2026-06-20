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
