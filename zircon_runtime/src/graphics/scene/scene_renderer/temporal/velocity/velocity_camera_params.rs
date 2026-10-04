use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    ProjectionMode, ViewProjectionMatrixPair, ViewportCameraSnapshot,
};
use crate::core::math::{Mat4, UVec2};

const VELOCITY_CAMERA_ENABLED: u32 = 1;
const VELOCITY_CAMERA_PERSPECTIVE: u32 = 1;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
/// 相机运动向量 shader 的非抖动矩阵契约；相机 cut 或无效逆矩阵时禁用重投影。
pub(in crate::graphics::scene::scene_renderer) struct VelocityCameraParams {
    pub(in crate::graphics::scene::scene_renderer) viewport_and_flags: [u32; 4],
    pub(in crate::graphics::scene::scene_renderer) current_clip_from_world: [[f32; 4]; 4],
    pub(in crate::graphics::scene::scene_renderer) current_world_from_clip: [[f32; 4]; 4],
    pub(in crate::graphics::scene::scene_renderer) previous_clip_from_world: [[f32; 4]; 4],
}

impl VelocityCameraParams {
    pub(in crate::graphics::scene::scene_renderer) fn from_cameras(
        viewport_size: UVec2,
        current: &ViewportCameraSnapshot,
        previous: &ViewportCameraSnapshot,
        enabled: bool,
    ) -> Self {
        let mut enabled = enabled && current.supports_temporal_reprojection_from(previous);
        let (current_clip_from_world, current_world_from_clip, previous_clip_from_world) =
            if enabled {
                let current_clip_from_world = camera_clip_from_world(current, viewport_size);
                let current_world_from_clip = current_clip_from_world.inverse();
                let previous_clip_from_world = camera_clip_from_world(previous, viewport_size);
                if velocity_camera_matrix_finite(current_clip_from_world)
                    && velocity_camera_matrix_finite(current_world_from_clip)
                    && velocity_camera_matrix_finite(previous_clip_from_world)
                {
                    (
                        current_clip_from_world,
                        current_world_from_clip,
                        previous_clip_from_world,
                    )
                } else {
                    enabled = false;
                    (Mat4::IDENTITY, Mat4::IDENTITY, Mat4::IDENTITY)
                }
            } else {
                (Mat4::IDENTITY, Mat4::IDENTITY, Mat4::IDENTITY)
            };
        let projection_flag = if matches!(current.projection_mode, ProjectionMode::Perspective) {
            VELOCITY_CAMERA_PERSPECTIVE
        } else {
            0
        };

        Self {
            viewport_and_flags: [
                viewport_size.x.max(1),
                viewport_size.y.max(1),
                if enabled { VELOCITY_CAMERA_ENABLED } else { 0 },
                projection_flag,
            ],
            current_clip_from_world: current_clip_from_world.to_cols_array_2d(),
            current_world_from_clip: current_world_from_clip.to_cols_array_2d(),
            previous_clip_from_world: previous_clip_from_world.to_cols_array_2d(),
        }
    }

    pub(in crate::graphics::scene::scene_renderer) fn is_enabled(self) -> bool {
        self.viewport_and_flags[2] == VELOCITY_CAMERA_ENABLED
    }

    pub(in crate::graphics::scene::scene_renderer) fn previous_clip_from_world(
        self,
    ) -> [[f32; 4]; 4] {
        self.previous_clip_from_world
    }
}

fn velocity_camera_matrix_finite(matrix: Mat4) -> bool {
    matrix
        .to_cols_array()
        .into_iter()
        .all(|value| value.is_finite())
}

fn camera_clip_from_world(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Mat4 {
    ViewProjectionMatrixPair::from_camera(camera, viewport_size).clip_from_world_unjittered
}

#[cfg(test)]
#[path = "tests/velocity_camera_params.rs"]
mod tests;
