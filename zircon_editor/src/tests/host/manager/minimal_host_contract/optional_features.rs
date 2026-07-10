use super::*;

#[test]
fn editor_manager_plugin_status_lists_owner_optional_feature_dependencies() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_optional_feature_status");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Optional Feature Status",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let blocked = manager
        .set_project_plugin_feature_enabled(
            &mut manifest,
            "sound",
            "sound.timeline_animation_track",
            true,
        )
        .unwrap_err();
    assert!(
        blocked.contains("missing plugins"),
        "feature enablement should report disabled owner dependencies: {blocked}"
    );

    let dependency_report = manager
        .enable_project_plugin_feature_dependencies(
            &mut manifest,
            "sound",
            "sound.timeline_animation_track",
        )
        .expect("explicit dependency enable should update plugin selections");
    assert_eq!(
        dependency_report.enabled_dependency_plugins,
        vec![
            "sound".to_string(),
            "animation".to_string(),
            "sound_timeline_animation_track".to_string(),
        ]
    );
    assert!(manifest.plugins.selections.iter().any(|selection| {
        selection.id == "sound_timeline_animation_track" && selection.enabled
    }));

    let status = manager.plugin_status_report(&manifest);
    let sound = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "sound")
        .expect("sound plugin should be in the builtin catalog");
    let timeline = sound
        .optional_features
        .iter()
        .find(|feature| feature.id == "sound.timeline_animation_track")
        .expect("sound timeline animation optional feature should be projected");

    assert!(!timeline.enabled);
    assert!(timeline.available);
    assert_eq!(timeline.owner_plugin_id, "sound");
    assert!(timeline
        .provided_capabilities
        .contains(&"runtime.feature.sound.timeline_animation_track".to_string()));
    assert!(timeline.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "sound"
            && dependency.capability == "runtime.plugin.sound"
            && dependency.primary
            && dependency.plugin_enabled
            && dependency.capability_available
    }));
    assert!(timeline.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "animation"
            && dependency.capability == "runtime.feature.animation.timeline_event_track"
            && !dependency.primary
            && dependency.plugin_enabled
            && dependency.capability_available
    }));

    let feature_report = manager
        .set_project_plugin_feature_enabled(
            &mut manifest,
            "sound",
            "sound.timeline_animation_track",
            true,
        )
        .expect("feature should enable after dependency plugins are enabled");
    assert!(feature_report.enabled);
    assert!(feature_report
        .project_selection
        .features
        .iter()
        .any(|feature| feature.id == "sound.timeline_animation_track" && feature.enabled));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_feature_dependency_enablement_turns_on_unique_provider_features() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_optional_feature_provider_dependencies");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Optional Feature Provider Dependencies",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    let report = manager
        .enable_project_plugin_feature_dependencies(
            &mut manifest,
            "rendering",
            "rendering.vfx_graph",
        )
        .expect("explicit dependency enable should include provider features");

    assert!(report
        .enabled_dependency_plugins
        .contains(&"rendering".to_string()));
    assert!(report
        .enabled_dependency_plugins
        .contains(&"particles".to_string()));
    assert_eq!(
        report.enabled_dependency_features,
        vec!["rendering.shader_graph".to_string()]
    );

    let rendering_selection = manifest
        .plugins
        .selections
        .iter()
        .find(|selection| selection.id == "rendering")
        .expect("rendering selection should be written back");
    assert!(rendering_selection
        .features
        .iter()
        .any(|feature| feature.id == "rendering.shader_graph" && feature.enabled));
    assert!(rendering_selection
        .features
        .iter()
        .any(|feature| feature.id == "rendering.vfx_graph" && !feature.enabled));

    let status = manager.plugin_status_report(&manifest);
    let vfx_graph = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "rendering")
        .and_then(|plugin| {
            plugin
                .optional_features
                .iter()
                .find(|feature| feature.id == "rendering.vfx_graph")
        })
        .expect("vfx graph optional feature should be projected");
    assert!(!vfx_graph.enabled);
    assert!(vfx_graph.available);

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}

#[test]
fn editor_manager_plugin_status_lists_rendering_owner_features_and_defaults() {
    let _guard = env_lock().lock().unwrap();
    let path = unique_temp_path("zircon_editor_rendering_optional_feature_status");
    let runtime = editor_runtime_with_disabled_subsystems_config_path(&path);
    let manager = runtime
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap();
    let mut manifest = zircon_runtime::asset::project::ProjectManifest::new(
        "Rendering Optional Feature Status",
        zircon_runtime::asset::AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    );

    manager
        .set_project_plugin_enabled(&mut manifest, "rendering", true)
        .expect("rendering plugin should be selectable from builtin catalogs");

    let status = manager.plugin_status_report(&manifest);
    let rendering = status
        .plugins
        .iter()
        .find(|plugin| plugin.plugin_id == "rendering")
        .expect("rendering plugin should be projected into the plugin manager status");

    assert!(rendering.enabled);
    assert_eq!(rendering.optional_features.len(), 9);
    assert!(rendering
        .editor_capabilities
        .contains(&"editor.extension.rendering_authoring".to_string()));
    assert_eq!(
        rendering
            .optional_features
            .iter()
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "rendering.post_process",
            "rendering.ssao",
            "rendering.contact_shadow",
            "rendering.decals",
            "rendering.reflection_probes",
            "rendering.baked_lighting",
            "rendering.ray_tracing_policy",
            "rendering.shader_graph",
            "rendering.vfx_graph",
        ]
    );
    assert_eq!(
        rendering
            .optional_features
            .iter()
            .filter(|feature| feature.enabled)
            .map(|feature| feature.id.as_str())
            .collect::<Vec<_>>(),
        vec![
            "rendering.post_process",
            "rendering.ssao",
            "rendering.reflection_probes",
            "rendering.baked_lighting",
        ]
    );

    let contact_shadow = rendering
        .optional_features
        .iter()
        .find(|feature| feature.id == "rendering.contact_shadow")
        .expect("contact shadow feature status");
    assert!(!contact_shadow.enabled);
    assert!(contact_shadow.available);

    let shader_graph = rendering
        .optional_features
        .iter()
        .find(|feature| feature.id == "rendering.shader_graph")
        .expect("shader graph feature status");
    assert!(!shader_graph.enabled);
    assert!(shader_graph.available);
    assert_eq!(
        shader_graph.runtime_crate.as_deref(),
        Some("zircon_plugin_rendering_shader_graph_runtime")
    );
    assert_eq!(
        shader_graph.editor_crate.as_deref(),
        Some("zircon_plugin_rendering_shader_graph_editor")
    );

    let vfx_graph = rendering
        .optional_features
        .iter()
        .find(|feature| feature.id == "rendering.vfx_graph")
        .expect("vfx graph feature status");
    assert!(!vfx_graph.enabled);
    assert!(!vfx_graph.available);
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "particles"
            && dependency.capability == "runtime.plugin.particles"
            && !dependency.primary
            && !dependency.plugin_enabled
            && !dependency.capability_available
    }));
    assert!(vfx_graph.dependencies.iter().any(|dependency| {
        dependency.plugin_id == "rendering"
            && dependency.capability == "runtime.feature.rendering.shader_graph"
            && !dependency.primary
            && dependency.plugin_enabled
            && !dependency.capability_available
    }));

    std::env::remove_var("ZIRCON_CONFIG_PATH");
    let _ = std::fs::remove_file(path);
}
