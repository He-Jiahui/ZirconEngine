use crate::RuntimeTargetMode;

pub(super) struct SoundFeatureRow {
    pub id_suffix: &'static str,
    pub display_name: &'static str,
    pub runtime_capability: &'static str,
    pub editor_capability: &'static str,
    pub runtime_crate: &'static str,
    pub editor_crate: &'static str,
    pub runtime_target_modes: &'static [RuntimeTargetMode],
    pub extra_dependencies: &'static [SoundFeatureDependencyRow],
}

pub(super) struct SoundFeatureDependencyRow {
    pub provider_plugin_id: &'static str,
    pub capability: &'static str,
}

const CLIENT_EDITOR_TARGETS: &[RuntimeTargetMode] = &[
    RuntimeTargetMode::ClientRuntime,
    RuntimeTargetMode::EditorHost,
];

const TIMELINE_ANIMATION_TRACK_DEPENDENCIES: &[SoundFeatureDependencyRow] =
    &[SoundFeatureDependencyRow {
        provider_plugin_id: "animation",
        capability: "runtime.feature.animation.timeline_event_track",
    }];

const RAY_TRACED_CONVOLUTION_REVERB_DEPENDENCIES: &[SoundFeatureDependencyRow] = &[
    SoundFeatureDependencyRow {
        provider_plugin_id: "physics",
        capability: "runtime.plugin.physics",
    },
    SoundFeatureDependencyRow {
        provider_plugin_id: "physics",
        capability: "runtime.capability.physics.raycast",
    },
];

pub(super) const SOUND_FEATURE_ROWS: &[SoundFeatureRow] = &[
    SoundFeatureRow {
        id_suffix: "timeline_animation_track",
        display_name: "Sound Timeline Animation Track",
        runtime_capability: "runtime.feature.sound.timeline_animation_track",
        editor_capability: "editor.feature.sound.timeline_animation_track",
        runtime_crate: "zircon_plugin_sound_timeline_animation_runtime",
        editor_crate: "zircon_plugin_sound_timeline_animation_editor",
        runtime_target_modes: CLIENT_EDITOR_TARGETS,
        extra_dependencies: TIMELINE_ANIMATION_TRACK_DEPENDENCIES,
    },
    SoundFeatureRow {
        id_suffix: "ray_traced_convolution_reverb",
        display_name: "Ray Traced Convolution Reverb",
        runtime_capability: "runtime.feature.sound.ray_traced_convolution_reverb",
        editor_capability: "editor.feature.sound.ray_traced_convolution_reverb",
        runtime_crate: "zircon_plugin_sound_ray_traced_convolution_runtime",
        editor_crate: "zircon_plugin_sound_ray_traced_convolution_editor",
        runtime_target_modes: CLIENT_EDITOR_TARGETS,
        extra_dependencies: RAY_TRACED_CONVOLUTION_REVERB_DEPENDENCIES,
    },
];
