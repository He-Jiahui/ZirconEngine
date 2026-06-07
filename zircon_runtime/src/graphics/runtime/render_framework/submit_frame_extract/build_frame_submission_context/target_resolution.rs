use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{RGBA8_UNORM_FORMAT, RGBA8_UNORM_SRGB_FORMAT};
use crate::core::framework::render::{
    RenderCameraTarget, RenderFrameworkError, RenderImageDescriptor, RenderImageDimension,
    RenderImageUsage,
};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, TextureMarker};

const CAMERA_TEXTURE_TARGET_ASSET_CAPABILITY: &str = "camera texture render target asset";
const CAMERA_TEXTURE_TARGET_EXTENT_CAPABILITY: &str = "camera texture render target extent";
const CAMERA_TEXTURE_TARGET_FORMAT_CAPABILITY: &str = "camera texture render target format";
const CAMERA_TEXTURE_TARGET_SHAPE_CAPABILITY: &str = "camera texture render target 2d single-layer";
const CAMERA_TEXTURE_TARGET_USAGE_CAPABILITY: &str = "camera texture render target usage";
const CAMERA_TEXTURE_SURFACE_PRESENT_CAPABILITY: &str = "camera texture surface present";
const HEADLESS_CAMERA_SURFACE_PRESENT_CAPABILITY: &str = "headless camera surface present";

pub(super) fn resolve_camera_target_size(
    primary_size: UVec2,
    target: &RenderCameraTarget,
    asset_manager: &ProjectAssetManager,
) -> Result<UVec2, RenderFrameworkError> {
    match target {
        RenderCameraTarget::PrimarySurface => Ok(clamp_target_size(primary_size)),
        RenderCameraTarget::Headless { size } => Ok(clamp_target_size(*size)),
        RenderCameraTarget::Texture(texture) => {
            resolve_camera_texture_target_size(asset_manager, *texture)
        }
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
        RenderCameraTarget::Texture(_) => Err(RenderFrameworkError::UnsupportedCapability {
            capability: CAMERA_TEXTURE_SURFACE_PRESENT_CAPABILITY.to_string(),
        }),
    }
}

fn resolve_camera_texture_target_size(
    asset_manager: &ProjectAssetManager,
    texture: ResourceHandle<TextureMarker>,
) -> Result<UVec2, RenderFrameworkError> {
    let descriptor = asset_manager
        .load_texture_asset(texture.id())
        .map(|texture| texture.render_image_descriptor())
        .map_err(|_| unsupported_camera_texture_target_asset())?;
    validate_camera_texture_target_descriptor(&descriptor)?;
    Ok(UVec2::new(descriptor.width, descriptor.height))
}

fn validate_camera_texture_target_descriptor(
    descriptor: &RenderImageDescriptor,
) -> Result<(), RenderFrameworkError> {
    if descriptor.width == 0 || descriptor.height == 0 {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: CAMERA_TEXTURE_TARGET_EXTENT_CAPABILITY.to_string(),
        });
    }
    if descriptor.dimension != RenderImageDimension::D2
        || descriptor.depth_or_array_layers != 1
        || descriptor.array_layer_count != 1
        || descriptor.mip_count != 1
    {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: CAMERA_TEXTURE_TARGET_SHAPE_CAPABILITY.to_string(),
        });
    }
    if !is_camera_texture_render_target_format(&descriptor.format) {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: CAMERA_TEXTURE_TARGET_FORMAT_CAPABILITY.to_string(),
        });
    }
    if !descriptor.usage.contains(&RenderImageUsage::RenderTarget) {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: CAMERA_TEXTURE_TARGET_USAGE_CAPABILITY.to_string(),
        });
    }
    Ok(())
}

fn is_camera_texture_render_target_format(format: &str) -> bool {
    let format = format.trim();
    format.eq_ignore_ascii_case(RGBA8_UNORM_FORMAT)
        || format.eq_ignore_ascii_case(RGBA8_UNORM_SRGB_FORMAT)
}

fn unsupported_camera_texture_target_asset() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: CAMERA_TEXTURE_TARGET_ASSET_CAPABILITY.to_string(),
    }
}

fn clamp_target_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}
