use crate::PLUGIN_ID;

pub fn sound_timeline_animation_track_feature_manifest(
) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        "sound.timeline_animation_track",
        "Sound Timeline Animation Track",
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        "runtime.plugin.sound",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "animation",
        "runtime.feature.animation.timeline_event_track",
    ))
    .with_capability("runtime.feature.sound.timeline_animation_track")
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "sound.timeline_animation_track.runtime",
            "zircon_plugin_sound_timeline_animation_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.timeline_animation_track".to_string()]),
    )
    .with_editor_module(
        zircon_runtime::plugin::PluginModuleManifest::editor(
            "sound.timeline_animation_track.editor",
            "zircon_plugin_sound_timeline_animation_editor",
        )
        .with_capabilities(["editor.feature.sound.timeline_animation_track".to_string()]),
    )
}

pub fn sound_ray_traced_convolution_reverb_feature_manifest(
) -> zircon_runtime::plugin::PluginFeatureBundleManifest {
    zircon_runtime::plugin::PluginFeatureBundleManifest::new(
        "sound.ray_traced_convolution_reverb",
        "Ray Traced Convolution Reverb",
        PLUGIN_ID,
    )
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::primary(
        PLUGIN_ID,
        "runtime.plugin.sound",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.plugin.physics",
    ))
    .with_dependency(zircon_runtime::plugin::PluginFeatureDependency::required(
        "physics",
        "runtime.capability.physics.raycast",
    ))
    .with_capability("runtime.feature.sound.ray_traced_convolution_reverb")
    .with_runtime_module(
        zircon_runtime::plugin::PluginModuleManifest::runtime(
            "sound.ray_traced_convolution_reverb.runtime",
            "zircon_plugin_sound_ray_traced_convolution_runtime",
        )
        .with_target_modes([
            zircon_runtime::RuntimeTargetMode::ClientRuntime,
            zircon_runtime::RuntimeTargetMode::EditorHost,
        ])
        .with_capabilities(["runtime.feature.sound.ray_traced_convolution_reverb".to_string()]),
    )
    .with_editor_module(
        zircon_runtime::plugin::PluginModuleManifest::editor(
            "sound.ray_traced_convolution_reverb.editor",
            "zircon_plugin_sound_ray_traced_convolution_editor",
        )
        .with_capabilities(["editor.feature.sound.ray_traced_convolution_reverb".to_string()]),
    )
}
