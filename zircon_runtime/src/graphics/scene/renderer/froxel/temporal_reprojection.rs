use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    halton, FroxelGridParams, ViewProjectionMatrixPair, ViewportCameraSnapshot,
};
use crate::core::math::UVec2;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
/// 上传给散射 shader 的跨帧 ABI：使用上一帧未抖动矩阵、相机位置/前向和深度范围重投影；历史不可用时把最后的混合权重设为零，避免伪造有效历史。
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
#[path = "tests/temporal_reprojection.rs"]
mod tests;
