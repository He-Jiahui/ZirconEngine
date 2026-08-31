use crate::core::math::UVec2;

use super::super::{CameraRenderDescriptor, RenderCameraTarget};

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
