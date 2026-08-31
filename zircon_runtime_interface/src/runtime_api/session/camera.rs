use serde::{Deserialize, Serialize};

use crate::math::Transform;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZrRuntimeViewportCameraV1 {
    pub abi_version: u32,
    pub transform: Transform,
    pub projection_kind: u32,
    pub fov_y_radians: f32,
    pub ortho_size: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl ZrRuntimeViewportCameraV1 {
    pub const fn new(
        abi_version: u32,
        transform: Transform,
        projection_kind: u32,
        fov_y_radians: f32,
        ortho_size: f32,
        z_near: f32,
        z_far: f32,
    ) -> Self {
        Self {
            abi_version,
            transform,
            projection_kind,
            fov_y_radians,
            ortho_size,
            z_near,
            z_far,
        }
    }
}
