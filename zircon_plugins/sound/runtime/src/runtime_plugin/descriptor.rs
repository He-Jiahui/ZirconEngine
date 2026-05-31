use crate::PLUGIN_ID;

use super::feature_manifest::{
    sound_ray_traced_convolution_reverb_feature_manifest,
    sound_timeline_animation_track_feature_manifest,
};

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::new(
        PLUGIN_ID,
        "Sound",
        zircon_runtime::RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_target_modes([
        zircon_runtime::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::RuntimeTargetMode::EditorHost,
    ])
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_capability("runtime.plugin.sound")
    .with_capability_status(
        zircon_runtime::plugin::CapabilityStatusManifest::new(
            "runtime.plugin.sound",
            zircon_runtime::plugin::CapabilityStatus::Partial,
        )
        .with_bevy_reference("dev/bevy/crates/bevy_audio/src/lib.rs"),
    )
    .with_optional_feature(sound_timeline_animation_track_feature_manifest())
    .with_optional_feature(sound_ray_traced_convolution_reverb_feature_manifest())
}
