use bytemuck::{Pod, Zeroable};

use crate::core::framework::render::{
    FroxelGridParams, ProjectionMode, ViewProjectionMatrixPair, ViewportCameraSnapshot,
};
use crate::core::math::{Mat4, UVec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FroxelViewReconstruction {
    world_from_clip: Mat4,
    camera_position: Vec3,
    camera_forward: Vec3,
    orthographic: bool,
}

impl FroxelViewReconstruction {
    pub(crate) fn perspective(
        world_from_clip: Mat4,
        camera_position: Vec3,
        camera_forward: Vec3,
    ) -> Self {
        Self {
            world_from_clip,
            camera_position,
            camera_forward,
            orthographic: false,
        }
    }

    pub(crate) fn from_camera(camera: &ViewportCameraSnapshot, viewport_size: UVec2) -> Self {
        let clip_from_world =
            ViewProjectionMatrixPair::from_camera(camera, viewport_size).clip_from_world_unjittered;
        Self {
            world_from_clip: clip_from_world.inverse(),
            camera_position: camera.transform.translation,
            camera_forward: camera.transform.rotation * Vec3::NEG_Z,
            orthographic: camera.projection_mode == ProjectionMode::Orthographic,
        }
    }

    fn validate(self) -> Result<Self, String> {
        if !self.world_from_clip.is_finite()
            || !vec3_is_finite(self.camera_position)
            || !vec3_is_finite(self.camera_forward)
        {
            return Err("froxel view reconstruction inputs must be finite".to_string());
        }
        if self.camera_forward.length_squared() <= 0.000001 {
            return Err("froxel view reconstruction camera forward must be nonzero".to_string());
        }
        Ok(Self {
            camera_forward: self.camera_forward.normalize(),
            ..self
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(crate) struct GpuFroxelViewParams {
    world_from_clip: [[f32; 4]; 4],
    camera_position_projection: [f32; 4],
    camera_forward: [f32; 4],
    depth: [f32; 4],
}

impl GpuFroxelViewParams {
    pub(crate) fn new(
        view: FroxelViewReconstruction,
        grid: FroxelGridParams,
    ) -> Result<Self, String> {
        let view = view.validate()?;
        let grid = grid.sanitized();
        Ok(Self {
            world_from_clip: view.world_from_clip.to_cols_array_2d(),
            camera_position_projection: [
                view.camera_position.x,
                view.camera_position.y,
                view.camera_position.z,
                if view.orthographic { 1.0 } else { 0.0 },
            ],
            camera_forward: [
                view.camera_forward.x,
                view.camera_forward.y,
                view.camera_forward.z,
                0.0,
            ],
            depth: [
                grid.near_depth,
                grid.far_depth,
                grid.depth_distribution_exp,
                0.0,
            ],
        })
    }
}

fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_froxel_view_reconstruction_uses_unjittered_camera_projection() {
        let camera = ViewportCameraSnapshot::default();
        let viewport = UVec2::new(1600, 900);

        let view = FroxelViewReconstruction::from_camera(&camera, viewport);
        let expected = ViewProjectionMatrixPair::from_camera(&camera, viewport)
            .clip_from_world_unjittered
            .inverse();

        assert_eq!(view.world_from_clip, expected);
        assert!(!view.orthographic);
    }
}
