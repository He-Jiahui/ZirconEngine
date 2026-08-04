use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    FroxelGridParams, ViewProjectionMatrixPair, ViewportCameraSnapshot, halton,
};
use crate::core::math::UVec2;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(crate) struct GpuFroxelTemporalReprojection {
    previous_clip_from_world: [[f32; 4]; 4],
    previous_camera_position: [f32; 4],
    previous_camera_forward: [f32; 4],
    previous_depth: [f32; 4],
    jitter_and_history: [f32; 4],
}

impl GpuFroxelTemporalReprojection {
    pub(crate) fn new(
        current: &ViewportCameraSnapshot,
        previous: Option<&ViewportCameraSnapshot>,
        viewport_size: UVec2,
        grid: FroxelGridParams,
        jitter_enabled: bool,
        history_available: bool,
    ) -> Self {
        let grid = grid.sanitized();
        let previous = previous.unwrap_or(current);
        let previous_clip_from_world =
            ViewProjectionMatrixPair::from_camera(previous, viewport_size)
                .clip_from_world_unjittered;
        let previous_forward = previous.transform.rotation * crate::core::math::Vec3::NEG_Z;
        let sequence_index = if jitter_enabled {
            current.temporal_jitter.sequence_index
        } else {
            0
        };
        let jitter_z = if sequence_index == 0 {
            0.0
        } else {
            halton(sequence_index, 5) - 0.5
        };
        Self {
            previous_clip_from_world: previous_clip_from_world.to_cols_array_2d(),
            previous_camera_position: previous.transform.translation.extend(0.0).to_array(),
            previous_camera_forward: previous_forward.normalize_or_zero().extend(0.0).to_array(),
            previous_depth: [
                previous.z_near.max(0.0001),
                previous.z_far.max(previous.z_near + 0.0001),
                grid.depth_distribution_exp,
                0.0,
            ],
            jitter_and_history: [
                if jitter_enabled {
                    current.temporal_jitter.offset_pixels.x
                } else {
                    0.0
                },
                if jitter_enabled {
                    current.temporal_jitter.offset_pixels.y
                } else {
                    0.0
                },
                jitter_z,
                if history_available { 0.9 } else { 0.0 },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::{FroxelGridQuality, TemporalJitterSample};
    use crate::core::math::Vec2;

    #[test]
    fn temporal_reprojection_consumes_camera_jitter_only_when_history_is_available() {
        let current = ViewportCameraSnapshot {
            temporal_jitter: TemporalJitterSample {
                offset_pixels: Vec2::new(0.25, -0.125),
                sequence_index: 3,
            },
            ..ViewportCameraSnapshot::default()
        };
        let grid = FroxelGridParams::for_quality(FroxelGridQuality::High, 0.1, 1000.0, 2.0);

        let unavailable = GpuFroxelTemporalReprojection::new(
            &current,
            None,
            UVec2::new(1600, 900),
            grid,
            true,
            false,
        );
        let available = GpuFroxelTemporalReprojection::new(
            &current,
            Some(&current),
            UVec2::new(1600, 900),
            grid,
            true,
            true,
        );

        assert_eq!(unavailable.jitter_and_history[0], 0.25);
        assert_eq!(unavailable.jitter_and_history[1], -0.125);
        assert_eq!(unavailable.jitter_and_history[3], 0.0);
        assert_eq!(available.jitter_and_history[3], 0.9);
        assert!((available.jitter_and_history[2] - 0.1).abs() <= 0.000001);
    }
}
