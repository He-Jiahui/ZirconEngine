use crate::ui::host::EditorPluginFeatureStatus;
use zircon_runtime::asset::{project::ProjectManifest, AssetUri};
use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::ExportPackagingStrategy;

pub(super) fn module_plugin_optional_feature_summary(
    features: &[EditorPluginFeatureStatus],
) -> String {
    features
        .iter()
        .map(|feature| {
            let state = if feature.enabled {
                if feature.available {
                    "enabled"
                } else {
                    "blocked"
                }
            } else if feature.available {
                "ready"
            } else {
                "blocked"
            };
            let dependencies = feature
                .dependencies
                .iter()
                .map(|dependency| {
                    let dependency_state =
                        match (dependency.plugin_enabled, dependency.capability_available) {
                            (true, true) => "ok",
                            (false, _) => "missing plugin",
                            (true, false) => "missing capability",
                        };
                    let role = if dependency.primary { "primary " } else { "" };
                    format!(
                        "{role}{}:{} ({dependency_state})",
                        dependency.plugin_id, dependency.capability
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            if dependencies.is_empty() {
                format!("{} [{state}]", feature.display_name)
            } else {
                format!("{} [{state}] deps: {dependencies}", feature.display_name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub(super) fn module_plugin_feature_action(
    features: &[EditorPluginFeatureStatus],
) -> (String, String) {
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && !feature.available)
    {
        return (
            "Enable Deps".to_string(),
            module_plugin_feature_action_id(
                "workbench.plugin.feature.enable_dependencies",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| !feature.enabled && feature.available)
    {
        return (
            "Enable Feature".to_string(),
            module_plugin_feature_action_id(
                "workbench.plugin.feature.enable",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    if let Some(feature) = features
        .iter()
        .find(|feature| feature.enabled && !feature.required)
    {
        return (
            "Disable Feature".to_string(),
            module_plugin_feature_action_id(
                "workbench.plugin.feature.disable",
                &feature.owner_plugin_id,
                &feature.id,
            ),
        );
    }
    (String::new(), String::new())
}

fn module_plugin_feature_action_id(prefix: &str, plugin_id: &str, feature_id: &str) -> String {
    format!("{prefix}.{plugin_id}.{feature_id}")
}

pub(super) fn module_plugin_primary_action(
    plugin_id: &str,
    enabled: bool,
    required: bool,
) -> (String, String) {
    if required {
        return ("Required".to_string(), String::new());
    }

    if enabled {
        (
            "Disable".to_string(),
            module_plugin_action_id("workbench.plugin.disable", plugin_id),
        )
    } else {
        (
            "Enable".to_string(),
            module_plugin_action_id("workbench.plugin.enable", plugin_id),
        )
    }
}

pub(super) fn module_plugin_action_id(prefix: &str, plugin_id: &str) -> String {
    format!("{prefix}.{plugin_id}")
}

pub(super) fn target_mode_label(mode: &RuntimeTargetMode) -> &'static str {
    match mode {
        RuntimeTargetMode::ClientRuntime => "client",
        RuntimeTargetMode::ServerRuntime => "server",
        RuntimeTargetMode::EditorHost => "editor",
    }
}

pub(super) fn packaging_label(strategy: ExportPackagingStrategy) -> &'static str {
    match strategy {
        ExportPackagingStrategy::SourceTemplate => "source-template",
        ExportPackagingStrategy::LibraryEmbed => "library-embed",
        ExportPackagingStrategy::NativeDynamic => "native-dynamic",
    }
}

pub(super) fn fallback_project_manifest() -> ProjectManifest {
    ProjectManifest::new(
        "Unsaved",
        AssetUri::parse("res://scenes/main.scene.toml")
            .expect("fallback project asset URI is valid"),
        1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::host::{EditorPluginFeatureDependencyStatus, EditorPluginFeatureStatus};

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
            provided_capabilities: vec![
                "runtime.feature.sound.timeline_animation_track".to_string()
            ],
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
}
