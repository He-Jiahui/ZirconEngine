use serde::Deserialize;
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::framework::sound::{
    SoundClipId, SoundError, SoundSourceDescriptor, SoundSourceInput, AUDIO_SOURCE_COMPONENT_TYPE,
};
use zircon_runtime::scene::World;

pub(super) fn enabled(world: &World, entity: EntityId) -> bool {
    world
        .dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
        .is_some_and(|value| {
            value.get("enabled").and_then(|value| value.as_bool()) != Some(false)
                && value.get("playing").and_then(|value| value.as_bool()) != Some(false)
                && world.active_in_hierarchy(entity) == Some(true)
        })
}

pub(super) fn read(world: &World, entity: EntityId) -> Result<SoundSourceDescriptor, SoundError> {
    let invalid = |error: String| SoundError::InvalidParameter(error);
    let value = world
        .dynamic_component(entity, AUDIO_SOURCE_COMPONENT_TYPE)
        .expect("enabled audio source must have a component");
    let input = value
        .get("input")
        .ok_or_else(|| invalid("AudioSource requires a sound_source_input".to_string()))?;
    let input = SoundSourceInput::deserialize(input).map_err(|error| invalid(error.to_string()))?;
    // Reuse the service's defaults. Input uses the existing SoundSourceInput
    // serde contract (e.g. {"Clip": 1}); clip loading remains the asset service's job.
    let mut source = SoundSourceDescriptor::clip(SoundClipId::new(0));
    source.input = input;
    macro_rules! read_fields {
        ($target:expr; $($field:ident),+ $(,)?) => {
            $(if let Some(value) = value.get(stringify!($field)) {
                $target.$field = Deserialize::deserialize(value)
                    .map_err(|error| invalid(format!("{}: {error}", stringify!($field))))?;
            })+
        };
    }
    read_fields!(source;
        output_track, sends, position, forward, velocity, gain, speed, looped,
        playing, muted, completion_action, start_seconds, duration_seconds, parameter_bindings,
        spatial,
    );
    // Descriptor metadata exposes spatial properties flat; also accept the
    // framework's nested spatial DTO without selecting a scene listener.
    read_fields!(source.spatial;
        spatial_blend, spatial_scale, attenuation, doppler_factor, occlusion_enabled,
        convolution_send,
    );
    // Service IDs and gameplay-emitter ownership are runtime state, never
    // adopted from a serialized component.
    Ok(source)
}
