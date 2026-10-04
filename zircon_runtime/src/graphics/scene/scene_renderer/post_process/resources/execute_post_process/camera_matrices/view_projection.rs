use crate::core::framework::render::{ViewProjectionMatrixPair, ViewportCameraSnapshot};
use crate::core::math::{Mat4, UVec2};

/// 为屏幕空间探针与追踪区域生成稳定的未抖动投影，viewport_size 须匹配本次编码的局部视图。
/// 时间抖动由重建链管理，不应改变这些 CPU 生成的空间权重。
pub(in super::super) fn view_projection(
    camera: &ViewportCameraSnapshot,
    viewport_size: UVec2,
) -> Mat4 {
    // Post-process screen-space inputs must not inherit temporal jitter.
    ViewProjectionMatrixPair::from_camera(camera, viewport_size).clip_from_world_unjittered
}
