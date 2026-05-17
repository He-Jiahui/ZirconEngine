use crate::core::framework::render::{RenderCameraTarget, RenderFrameworkError};
use crate::core::math::UVec2;

const CAMERA_TEXTURE_TARGET_CAPABILITY: &str = "camera texture render target";
const HEADLESS_CAMERA_SURFACE_PRESENT_CAPABILITY: &str = "headless camera surface present";

pub(super) fn resolve_camera_target_size(
    primary_size: UVec2,
    target: &RenderCameraTarget,
) -> Result<UVec2, RenderFrameworkError> {
    match target {
        RenderCameraTarget::PrimarySurface => Ok(clamp_target_size(primary_size)),
        RenderCameraTarget::Headless { size } => Ok(clamp_target_size(*size)),
        RenderCameraTarget::Texture(_) => Err(unsupported_camera_texture_target()),
    }
}

pub(in crate::graphics::runtime::render_framework::submit_frame_extract) fn validate_camera_surface_present_target(
    target: &RenderCameraTarget,
) -> Result<(), RenderFrameworkError> {
    match target {
        RenderCameraTarget::PrimarySurface => Ok(()),
        RenderCameraTarget::Headless { .. } => Err(RenderFrameworkError::UnsupportedCapability {
            capability: HEADLESS_CAMERA_SURFACE_PRESENT_CAPABILITY.to_string(),
        }),
        RenderCameraTarget::Texture(_) => Err(unsupported_camera_texture_target()),
    }
}

fn unsupported_camera_texture_target() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: CAMERA_TEXTURE_TARGET_CAPABILITY.to_string(),
    }
}

fn clamp_target_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}
