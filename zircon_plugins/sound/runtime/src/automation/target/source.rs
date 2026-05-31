use zircon_runtime::core::framework::sound::{SoundError, SoundParameterId, SoundSourceDescriptor};

use super::helpers::{bool_from_value, unsupported_automation_parameter};

pub(super) fn apply_source_parameter(
    source: &mut SoundSourceDescriptor,
    parameter: &SoundParameterId,
    value: f32,
) -> Result<(), SoundError> {
    match parameter.as_str() {
        "gain" => source.gain = value,
        "speed" => source.speed = value,
        "playing" => source.playing = bool_from_value(value),
        "looped" => source.looped = bool_from_value(value),
        "muted" => source.muted = bool_from_value(value),
        "position_x" => source.position[0] = value,
        "position_y" => source.position[1] = value,
        "position_z" => source.position[2] = value,
        "forward_x" => source.forward[0] = value,
        "forward_y" => source.forward[1] = value,
        "forward_z" => source.forward[2] = value,
        "velocity_x" => source.velocity[0] = value,
        "velocity_y" => source.velocity[1] = value,
        "velocity_z" => source.velocity[2] = value,
        "spatial_blend" => source.spatial.spatial_blend = value,
        "spatial_scale" => source.spatial.spatial_scale = Some(value),
        "min_distance" => source.spatial.min_distance = value,
        "max_distance" => source.spatial.max_distance = value,
        "cone_inner_degrees" => source.spatial.cone_inner_degrees = value,
        "cone_outer_degrees" => source.spatial.cone_outer_degrees = value,
        "doppler_factor" => source.spatial.doppler_factor = value,
        "occlusion_enabled" => source.spatial.occlusion_enabled = bool_from_value(value),
        _ => return Err(unsupported_automation_parameter("source", parameter)),
    }
    Ok(())
}
