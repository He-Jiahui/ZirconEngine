use zircon_runtime::core::framework::sound::{SoundVolumeDescriptor, SoundVolumeShape};

use crate::engine::math::{length3, sub3};

pub(super) fn volume_weight(source_position: [f32; 3], volume: &SoundVolumeDescriptor) -> f32 {
    let distance_outside = distance_outside_volume(source_position, volume);
    if distance_outside <= 0.0 {
        return 1.0;
    }
    let crossfade = volume.crossfade_distance.max(0.0);
    if crossfade <= 0.0 {
        0.0
    } else {
        (1.0 - distance_outside / crossfade).clamp(0.0, 1.0)
    }
}

fn distance_outside_volume(source_position: [f32; 3], volume: &SoundVolumeDescriptor) -> f32 {
    match &volume.shape {
        SoundVolumeShape::Sphere { center, radius } => {
            sphere_distance_outside(source_position, *center, *radius)
        }
        SoundVolumeShape::Box { center, extents } => {
            box_distance_outside(source_position, *center, *extents)
        }
    }
}

fn sphere_distance_outside(source_position: [f32; 3], center: [f32; 3], radius: f32) -> f32 {
    (length3(sub3(source_position, center)) - radius.max(0.0)).max(0.0)
}

fn box_distance_outside(source_position: [f32; 3], center: [f32; 3], extents: [f32; 3]) -> f32 {
    let delta = [
        (source_position[0] - center[0]).abs() - extents[0].max(0.0),
        (source_position[1] - center[1]).abs() - extents[1].max(0.0),
        (source_position[2] - center[2]).abs() - extents[2].max(0.0),
    ];
    length3([delta[0].max(0.0), delta[1].max(0.0), delta[2].max(0.0)])
}
