use crate::core::framework::render::{ViewProjectionMatrixPair, ViewportCameraSnapshot};
use crate::core::math::{Mat4, UVec2};

pub(in super::super) fn view_projection(
    camera: &ViewportCameraSnapshot,
    viewport_size: UVec2,
) -> Mat4 {
    // Post-process screen-space inputs must not inherit temporal jitter.
    ViewProjectionMatrixPair::from_camera(camera, viewport_size).clip_from_world_unjittered
}
