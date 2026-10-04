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

#[test]
fn qualified_package_feature_actions_keep_owner_and_feature_for_every_kind() {
    let plugin_id = "third_party.weather_sim";
    let feature_id = "weather.lightning";
    let route = format!("#{}:{plugin_id}.{feature_id}", plugin_id.len());

    assert_eq!(
        parse_module_plugin_action(&format!("workbench.plugin.feature.enable.{route}")),
        Some(ModulePluginAction::SetFeatureEnabled {
            plugin_id,
            feature_id,
            enabled: true,
        })
    );
    assert_eq!(
        parse_module_plugin_action(&format!("workbench.plugin.feature.disable.{route}")),
        Some(ModulePluginAction::SetFeatureEnabled {
            plugin_id,
            feature_id,
            enabled: false,
        })
    );
    assert_eq!(
        parse_module_plugin_action(&format!(
            "workbench.plugin.feature.enable_dependencies.{route}"
        )),
        Some(ModulePluginAction::EnableFeatureDependencies {
            plugin_id,
            feature_id,
        })
    );
}

#[test]
fn malformed_qualified_package_feature_actions_never_retarget() {
    for route in [
        "#:third_party.weather_sim.weather.lightning",
        "#0:third_party.weather_sim.weather.lightning",
        "#11:third_party.weather_sim.weather.lightning",
        "#999:third_party.weather_sim.weather.lightning",
        "#23:third_party.weather_sim",
        "#23:third_party.weather_sim.",
        "#1:é.weather.lightning",
    ] {
        assert_eq!(
            parse_module_plugin_action(&format!("workbench.plugin.feature.enable.{route}")),
            None,
            "{route}"
        );
    }
    assert_eq!(
        parse_module_plugin_action(
            "workbench.plugin.feature.unknown.#23:third_party.weather_sim.weather.lightning"
        ),
        None
    );
}
