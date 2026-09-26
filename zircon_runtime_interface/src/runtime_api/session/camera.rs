use serde::{Deserialize, Serialize};

use crate::math::Transform;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
/// 编辑器模拟视口相机的 JSON 事件载体。
/// 构造器只封装字段；运行时收到事件后再验证版本、变换与投影参数。
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
