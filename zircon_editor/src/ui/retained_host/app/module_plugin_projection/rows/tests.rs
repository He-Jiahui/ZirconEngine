use super::*;
use crate::ui::host::{EditorPluginFeatureDependencyStatus, EditorPluginFeatureStatus};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::ExportPackagingStrategy;

#[test]
fn module_plugin_primary_action_respects_required_and_enabled_state() {
    assert_eq!(
        module_plugin_primary_action("physics", true, false),
        (
            "Disable".to_string(),
            "workbench.plugin.disable.physics".to_string()
        )
    );
    assert_eq!(
        module_plugin_primary_action("physics", false, false),
        (
            "Enable".to_string(),
            "workbench.plugin.enable.physics".to_string()
        )
    );
    assert_eq!(
        module_plugin_primary_action("core", true, true),
        ("Required".to_string(), String::new())
    );
}

#[test]
fn module_plugin_optional_feature_summary_lists_dependency_state() {
    let summary = module_plugin_optional_feature_summary(&[EditorPluginFeatureStatus {
        id: "sound.timeline_animation_track".to_string(),
        display_name: "Sound Timeline Animation Track".to_string(),
        owner_plugin_id: "sound".to_string(),
        enabled: false,
        required: false,
        available: false,
        target_modes: vec![RuntimeTargetMode::EditorHost],
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: Some("zircon_plugin_sound_timeline_animation_runtime".to_string()),
        editor_crate: Some("zircon_plugin_sound_timeline_animation_editor".to_string()),
        provided_capabilities: vec!["runtime.feature.sound.timeline_animation_track".to_string()],
        dependencies: vec![
            EditorPluginFeatureDependencyStatus {
                plugin_id: "sound".to_string(),
                capability: "runtime.plugin.sound".to_string(),
                primary: true,
                plugin_enabled: true,
                capability_available: true,
            },
            EditorPluginFeatureDependencyStatus {
                plugin_id: "animation".to_string(),
                capability: "runtime.feature.animation.timeline_event_track".to_string(),
                primary: false,
                plugin_enabled: false,
                capability_available: false,
            },
        ],
        diagnostics: Vec::new(),
    }]);

    assert!(summary.contains("Sound Timeline Animation Track [blocked]"));
    assert!(summary.contains("primary sound:runtime.plugin.sound (ok)"));
    assert!(summary
        .contains("animation:runtime.feature.animation.timeline_event_track (missing plugin)"));
}

#[test]
fn module_plugin_feature_action_prefers_dependency_gate_then_enable() {
    let blocked = EditorPluginFeatureStatus {
        id: "sound.timeline_animation_track".to_string(),
        display_name: "Sound Timeline Animation Track".to_string(),
        owner_plugin_id: "sound".to_string(),
        enabled: false,
        required: false,
        available: false,
        target_modes: vec![RuntimeTargetMode::EditorHost],
        packaging: ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: None,
        editor_crate: None,
        provided_capabilities: Vec::new(),
        dependencies: Vec::new(),
        diagnostics: Vec::new(),
    };
    assert_eq!(
        module_plugin_feature_action(&[blocked.clone()]),
        (
            "Enable Deps".to_string(),
            "workbench.plugin.feature.enable_dependencies.sound.sound.timeline_animation_track"
                .to_string()
        )
    );

    let ready = EditorPluginFeatureStatus {
        available: true,
        ..blocked
    };
    assert_eq!(
        module_plugin_feature_action(&[ready]),
        (
            "Enable Feature".to_string(),
            "workbench.plugin.feature.enable.sound.sound.timeline_animation_track".to_string()
        )
    );
}
