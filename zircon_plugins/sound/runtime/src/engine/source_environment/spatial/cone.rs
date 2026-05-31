use zircon_runtime::core::framework::sound::SoundSourceDescriptor;

use super::super::super::math::{dot3, normalize3, sub3};

pub(super) fn cone_gain(
    source_forward: [f32; 3],
    source_position: [f32; 3],
    listener_position: [f32; 3],
    source: &SoundSourceDescriptor,
) -> f32 {
    let outer = source.spatial.cone_outer_degrees.clamp(0.0, 360.0);
    if outer >= 360.0 {
        return 1.0;
    }
    let inner = source.spatial.cone_inner_degrees.clamp(0.0, outer);
    let forward = normalize3(source_forward);
    let to_listener = normalize3(sub3(listener_position, source_position));
    let angle = dot3(forward, to_listener)
        .clamp(-1.0, 1.0)
        .acos()
        .to_degrees();
    let inner_half = inner * 0.5;
    let outer_half = outer * 0.5;
    if angle <= inner_half {
        1.0
    } else if angle >= outer_half {
        0.0
    } else {
        1.0 - ((angle - inner_half) / (outer_half - inner_half).max(0.0001))
    }
}
