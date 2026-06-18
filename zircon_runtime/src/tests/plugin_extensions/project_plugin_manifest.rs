use crate::builtin::{RuntimePluginId, RuntimeTargetMode};
use crate::plugin::{ProjectPluginFeatureSelection, ProjectPluginManifest, ProjectPluginSelection};

#[test]
fn project_plugin_manifest_preserves_nested_feature_selections() {
    let selection = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime")
                .with_editor_crate("zircon_plugin_sound_timeline_animation_editor")
                .with_target_modes([
                    RuntimeTargetMode::ClientRuntime,
                    RuntimeTargetMode::EditorHost,
                ]),
        );
    let manifest = ProjectPluginManifest {
        selections: vec![selection],
    };

    let encoded = toml::to_string(&manifest).expect("project manifest toml");
    let decoded: ProjectPluginManifest =
        toml::from_str(&encoded).expect("project manifest roundtrip");

    let sound = decoded
        .selections
        .iter()
        .find(|selection| selection.id == "sound")
        .expect("sound selection");
    assert_eq!(sound.features.len(), 1);
    assert_eq!(sound.features[0].id, "sound.timeline_animation_track");
    assert!(sound.features[0].enabled);
    assert_eq!(
        sound.features[0].runtime_crate.as_deref(),
        Some("zircon_plugin_sound_timeline_animation_runtime")
    );
}

#[test]
fn project_plugin_manifest_preserves_external_feature_provider_selection() {
    let selection = ProjectPluginSelection::runtime_plugin(RuntimePluginId::Sound, true, false)
        .with_feature(
            ProjectPluginFeatureSelection::new("sound.timeline_animation_track")
                .with_provider_package_id("sound_timeline_animation_track")
                .with_runtime_crate("zircon_plugin_sound_timeline_animation_runtime"),
        );
    let manifest = ProjectPluginManifest {
        selections: vec![selection],
    };

    let encoded = toml::to_string(&manifest).expect("project manifest toml");
    let decoded: ProjectPluginManifest =
        toml::from_str(&encoded).expect("project manifest roundtrip");
    let sound = decoded
        .selections
        .iter()
        .find(|selection| selection.id == "sound")
        .expect("sound selection");

    assert_eq!(
        sound.features[0].provider_package_id.as_deref(),
        Some("sound_timeline_animation_track")
    );
    assert_eq!(
        sound.features[0].runtime_crate_path("sound"),
        "sound_timeline_animation_track/runtime"
    );
}
