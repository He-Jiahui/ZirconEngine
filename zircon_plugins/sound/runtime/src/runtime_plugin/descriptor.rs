use crate::capability::{PLUGIN_ID, SOUND_RUNTIME_CAPABILITY};

use super::feature_manifest::{
    sound_ray_traced_convolution_reverb_feature_manifest,
    sound_timeline_animation_track_feature_manifest,
};

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    zircon_runtime::plugin::RuntimePluginDescriptor::builder(
        PLUGIN_ID,
        "Sound",
        zircon_runtime::builtin::RuntimePluginId::Sound,
        "zircon_plugin_sound_runtime",
    )
    .with_module_descriptor(crate::module::module_descriptor())
    .with_target_modes([
        zircon_runtime::core::framework::platform::RuntimeTargetMode::ClientRuntime,
        zircon_runtime::core::framework::platform::RuntimeTargetMode::EditorHost,
    ])
    .with_maturity(zircon_runtime::plugin::PluginMaturity::Beta)
    .with_capability(SOUND_RUNTIME_CAPABILITY)
    .with_capability_status(
        zircon_runtime::plugin::CapabilityStatusManifest::new(
            SOUND_RUNTIME_CAPABILITY,
            zircon_runtime::plugin::CapabilityStatus::Partial,
        )
        .with_bevy_reference("dev/bevy/crates/bevy_audio/src/lib.rs"),
    )
    .with_optional_feature(sound_timeline_animation_track_feature_manifest())
    .with_optional_feature(sound_ray_traced_convolution_reverb_feature_manifest())
    .build()
}
