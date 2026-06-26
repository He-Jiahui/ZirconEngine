use zircon_runtime::builtin::RuntimeTargetMode;
use zircon_runtime::plugin::{
    ExportPackagingStrategy, PluginFeatureBundleManifest, PluginModuleManifest,
};

use crate::capability::{
    PLUGIN_ID, SOUND_RAY_TRACED_CONVOLUTION_REVERB_CAPABILITY, SOUND_RUNTIME_CAPABILITY,
    SOUND_TIMELINE_ANIMATION_TRACK_CAPABILITY,
};

const SOUND_TIMELINE_ANIMATION_DIST_CRATE: &str = "zircon_plugin_sound_timeline_animation_dist";
const SOUND_RAY_TRACED_CONVOLUTION_DIST_CRATE: &str =
    "zircon_plugin_sound_ray_traced_convolution_dist";

pub fn sound_timeline_animation_track_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
        PLUGIN_ID,
    )
    .with_default_packaging([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ])
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        SOUND_RUNTIME_CAPABILITY,
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "animation",
        "runtime.feature.animation.timeline_event_track",
    ))
    .with_capability(SOUND_TIMELINE_ANIMATION_TRACK_CAPABILITY)
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([SOUND_TIMELINE_ANIMATION_TRACK_CAPABILITY.to_string()]),
    )
    .with_editor_module(
        PluginModuleManifest::editor(
            "sound.timeline_animation_track.editor",
            "zircon_plugin_sound_timeline_animation_editor",
        )
        .with_capabilities(["editor.feature.sound.timeline_animation_track".to_string()]),
    )
    .with_native_module(
        PluginModuleManifest::native(
            "sound.timeline_animation_track.dist",
            SOUND_TIMELINE_ANIMATION_DIST_CRATE,
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([SOUND_TIMELINE_ANIMATION_TRACK_CAPABILITY.to_string()]),
    )
}

pub fn sound_ray_traced_convolution_reverb_feature_manifest() -> PluginFeatureBundleManifest {
    PluginFeatureBundleManifest::new(
        "sound.ray_traced_convolution_reverb",
        "Ray Traced Convolution Reverb",
        PLUGIN_ID,
    )
    .with_default_packaging([
        ExportPackagingStrategy::SourceTemplate,
        ExportPackagingStrategy::LibraryEmbed,
        ExportPackagingStrategy::NativeDynamic,
    ])
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        SOUND_RUNTIME_CAPABILITY,
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.plugin.physics",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.capability.physics.raycast",
    ))
    .with_capability(SOUND_RAY_TRACED_CONVOLUTION_REVERB_CAPABILITY)
    .with_runtime_module(
        PluginModuleManifest::runtime(
            "sound.ray_traced_convolution_reverb.runtime",
            "zircon_plugin_sound_ray_traced_convolution_runtime",
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([SOUND_RAY_TRACED_CONVOLUTION_REVERB_CAPABILITY.to_string()]),
    )
    .with_editor_module(
        PluginModuleManifest::editor(
            "sound.ray_traced_convolution_reverb.editor",
            "zircon_plugin_sound_ray_traced_convolution_editor",
        )
        .with_capabilities(["editor.feature.sound.ray_traced_convolution_reverb".to_string()]),
    )
    .with_native_module(
        PluginModuleManifest::native(
            "sound.ray_traced_convolution_reverb.dist",
            SOUND_RAY_TRACED_CONVOLUTION_DIST_CRATE,
        )
        .with_target_modes([
            RuntimeTargetMode::ClientRuntime,
            RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities([SOUND_RAY_TRACED_CONVOLUTION_REVERB_CAPABILITY.to_string()]),
    )
}
