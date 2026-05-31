use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{SoundParameterId, SoundSourceDescriptor};

pub(in crate::engine::render) fn source_descriptor_with_parameter_bindings(
    descriptor: &SoundSourceDescriptor,
    parameters: &HashMap<SoundParameterId, f32>,
) -> SoundSourceDescriptor {
    let mut resolved = descriptor.clone();
    for binding in &descriptor.parameter_bindings {
        let Some(value) = parameters.get(&binding.synth_parameter).copied() else {
            continue;
        };
        apply_source_bound_parameter(&mut resolved, binding.source_parameter.as_str(), value);
    }
    resolved
}

fn apply_source_bound_parameter(source: &mut SoundSourceDescriptor, parameter: &str, value: f32) {
    match parameter {
        "gain" => source.gain = value,
        "speed" => source.speed = value,
        "playing" => source.playing = bool_from_parameter(value),
        "looped" => source.looped = bool_from_parameter(value),
        "muted" => source.muted = bool_from_parameter(value),
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
        "occlusion_enabled" => source.spatial.occlusion_enabled = bool_from_parameter(value),
        _ => {}
    }
}

fn bool_from_parameter(value: f32) -> bool {
    value >= 0.5
}
