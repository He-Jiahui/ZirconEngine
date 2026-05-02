use zircon_runtime::core::framework::sound::{
    AUDIO_LISTENER_COMPONENT_TYPE, AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};
use zircon_runtime::plugin::ComponentTypeDescriptor;

use crate::PLUGIN_ID;

pub fn sound_component_descriptors() -> Vec<ComponentTypeDescriptor> {
    vec![
        ComponentTypeDescriptor::new(AUDIO_SOURCE_COMPONENT_TYPE, PLUGIN_ID, "Audio Source")
            .with_property("input", "sound_source_input", true)
            .with_property("output_track", "sound_track_id", true)
            .with_property("position", "vec3", true)
            .with_property("forward", "vec3", true)
            .with_property("velocity", "vec3", true)
            .with_property("gain", "scalar", true)
            .with_property("looped", "bool", true)
            .with_property("playing", "bool", true)
            .with_property("spatial_blend", "scalar", true)
            .with_property("attenuation", "sound_attenuation", true)
            .with_property("doppler_factor", "scalar", true)
            .with_property("occlusion_enabled", "bool", true)
            .with_property("convolution_send", "sound_impulse_response_id", true),
        ComponentTypeDescriptor::new(AUDIO_LISTENER_COMPONENT_TYPE, PLUGIN_ID, "Audio Listener")
            .with_property("active", "bool", true)
            .with_property("position", "vec3", true)
            .with_property("forward", "vec3", true)
            .with_property("up", "vec3", true)
            .with_property("velocity", "vec3", true)
            .with_property("hrtf_profile", "string", true)
            .with_property("doppler_tracking", "bool", true)
            .with_property("mixer_target", "sound_track_id", true),
        ComponentTypeDescriptor::new(AUDIO_VOLUME_COMPONENT_TYPE, PLUGIN_ID, "Audio Volume")
            .with_property("shape", "sound_volume_shape", true)
            .with_property("priority", "integer", true)
            .with_property("interior_gain", "scalar", true)
            .with_property("exterior_gain", "scalar", true)
            .with_property("low_pass_cutoff_hz", "scalar", true)
            .with_property("reverb_send", "scalar", true)
            .with_property("convolution_send", "sound_impulse_response_id", true)
            .with_property("crossfade_distance", "scalar", true),
    ]
}
