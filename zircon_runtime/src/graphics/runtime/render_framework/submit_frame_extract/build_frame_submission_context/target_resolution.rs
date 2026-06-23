use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderCameraTarget, RenderFrameworkError, RenderImageDescriptor, RenderImageDimension,
    RenderImageUsage,
};
use crate::core::math::UVec2;
use crate::core::resource::{ResourceHandle, TextureMarker};
use crate::graphics::pipeline::{
    RenderGraphCompileCameraTargetFingerprint, RenderGraphCompileTextureTargetFormat,
};

const CAMERA_TEXTURE_TARGET_ASSET_CAPABILITY: &str = "camera texture render target asset";
const CAMERA_TEXTURE_TARGET_EXTENT_CAPABILITY: &str = "camera texture render target extent";
const CAMERA_TEXTURE_TARGET_FORMAT_CAPABILITY: &str = "camera texture render target format";
const CAMERA_TEXTURE_TARGET_SHAPE_CAPABILITY: &str = "camera texture render target 2d single-layer";
const CAMERA_TEXTURE_TARGET_USAGE_CAPABILITY: &str = "camera texture render target usage";
const CAMERA_TEXTURE_SURFACE_PRESENT_CAPABILITY: &str = "camera texture surface present";
const HEADLESS_CAMERA_SURFACE_PRESENT_CAPABILITY: &str = "headless camera surface present";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ResolvedCameraTargetDescriptor {
    size: UVec2,
    compile_fingerprint: RenderGraphCompileCameraTargetFingerprint,
    texture_format: Option<&'static str>,
}

impl ResolvedCameraTargetDescriptor {
    pub(super) const fn size(self) -> UVec2 {
        self.size
    }

    pub(super) const fn compile_fingerprint(self) -> RenderGraphCompileCameraTargetFingerprint {
        self.compile_fingerprint
    }

    pub(super) const fn texture_format(self) -> Option<&'static str> {
        self.texture_format
    }
}

pub(super) fn resolve_camera_target_descriptor(
    primary_size: UVec2,
    target: &RenderCameraTarget,
    asset_manager: &ProjectAssetManager,
) -> Result<ResolvedCameraTargetDescriptor, RenderFrameworkError> {
    match target {
        RenderCameraTarget::PrimarySurface => Ok(ResolvedCameraTargetDescriptor {
            size: clamp_target_size(primary_size),
            compile_fingerprint: RenderGraphCompileCameraTargetFingerprint::PrimarySurface,
            texture_format: None,
        }),
        RenderCameraTarget::Headless { size } => {
            let size = clamp_target_size(*size);
            Ok(ResolvedCameraTargetDescriptor {
                size,
                compile_fingerprint: RenderGraphCompileCameraTargetFingerprint::Headless {
                    width: size.x,
                    height: size.y,
                },
                texture_format: None,
            })
        }
        RenderCameraTarget::Texture(texture) => {
            resolve_camera_texture_target_descriptor(asset_manager, *texture)
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

fn resolve_camera_texture_target_descriptor(
    asset_manager: &ProjectAssetManager,
    texture: ResourceHandle<TextureMarker>,
) -> Result<ResolvedCameraTargetDescriptor, RenderFrameworkError> {
    let descriptor = asset_manager
        .load_texture_asset(texture.id())
        .map(|texture| texture.render_image_descriptor())
        .map_err(|_| unsupported_camera_texture_target_asset())?;
    let format = validate_camera_texture_target_descriptor(&descriptor)?;
    let size = UVec2::new(descriptor.width, descriptor.height);
    Ok(ResolvedCameraTargetDescriptor {
        size,
        compile_fingerprint: RenderGraphCompileCameraTargetFingerprint::Texture {
            id: texture.id(),
            width: size.x,
            height: size.y,
            format,
        },
        texture_format: Some(format.as_format_label()),
    })
}

fn validate_camera_texture_target_descriptor(
    descriptor: &RenderImageDescriptor,
) -> Result<RenderGraphCompileTextureTargetFormat, RenderFrameworkError> {
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
    let format = resolve_camera_texture_target_format(&descriptor.format)?;
    if !descriptor.usage.contains(&RenderImageUsage::RenderTarget) {
        return Err(RenderFrameworkError::UnsupportedCapability {
            capability: CAMERA_TEXTURE_TARGET_USAGE_CAPABILITY.to_string(),
        });
    }
    Ok(format)
}

fn resolve_camera_texture_target_format(
    format: &str,
) -> Result<RenderGraphCompileTextureTargetFormat, RenderFrameworkError> {
    RenderGraphCompileTextureTargetFormat::from_format_label(format).ok_or_else(|| {
        RenderFrameworkError::UnsupportedCapability {
            capability: CAMERA_TEXTURE_TARGET_FORMAT_CAPABILITY.to_string(),
        }
    })
}

fn unsupported_camera_texture_target_asset() -> RenderFrameworkError {
    RenderFrameworkError::UnsupportedCapability {
        capability: CAMERA_TEXTURE_TARGET_ASSET_CAPABILITY.to_string(),
    }
}

fn clamp_target_size(size: UVec2) -> UVec2 {
    UVec2::new(size.x.max(1), size.y.max(1))
}
