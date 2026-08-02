use crate::capability::{RUNTIME_CRATE_NAME, SOUND_DECLARATION, SOUND_RUNTIME_CAPABILITY};

use super::feature_manifest::{
    sound_ray_traced_convolution_reverb_feature_manifest,
    sound_timeline_animation_track_feature_manifest,
};

pub fn runtime_plugin_descriptor() -> zircon_runtime::plugin::RuntimePluginDescriptor {
    SOUND_DECLARATION
        .runtime_declaration(RUNTIME_CRATE_NAME)
        .with_module_descriptor(crate::module::module_descriptor())
        .with_capability_status(
            zircon_runtime::plugin::CapabilityStatusManifest::new(
                SOUND_RUNTIME_CAPABILITY,
                zircon_runtime::plugin::CapabilityStatus::Partial,
            )
            .with_bevy_reference("dev/bevy/crates/bevy_audio/src/lib.rs"),
        )
        .with_optional_feature(sound_timeline_animation_track_feature_manifest())
        .with_optional_feature(sound_ray_traced_convolution_reverb_feature_manifest())
        .into_descriptor()
}
