use crate::core::math::UVec2;

use super::super::{CameraRenderDescriptor, RenderCameraTarget};

/// 只从相机自身声明的区域或离屏目标推导尺寸；主窗口与纹理目标的尺寸由提交端提供。
/// 调用方在没有显式尺寸时应保留 `None`，而非把占位尺寸当作真实输出尺寸。
pub(in crate::core::framework::render) fn camera_target_size_from_descriptor(
    camera: Option<&CameraRenderDescriptor>,
) -> Option<UVec2> {
    let camera = camera?;
    if let Some(viewport) = camera.viewport_rect {
        return Some(viewport.physical_size);
    }
    match &camera.target {
        RenderCameraTarget::Headless { size } => Some(*size),
        RenderCameraTarget::PrimarySurface | RenderCameraTarget::Texture(_) => None,
    }
}
