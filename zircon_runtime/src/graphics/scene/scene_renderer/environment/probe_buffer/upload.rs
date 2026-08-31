use std::sync::Arc;

use thiserror::Error;
use zr_rhi::TextureCopyRegion;
use zr_rhi_wgpu::{WgpuTextureUpload, WgpuTextureUploadBatch};

use crate::asset::{
    decode_ibl_pmrem_rgba16f_texture, is_zcube_source_cubemap_texture, IblPmremTextureError,
    TextureAsset,
};
use crate::core::framework::render::RenderImageDimension;
use crate::core::resource::ResourceId;

use super::resources::{
    REFLECTION_PROBE_FACE_COUNT, REFLECTION_PROBE_FACE_SIZE, REFLECTION_PROBE_MIP_COUNT,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ReflectionProbeAssetRejectionReason {
    MissingResource,
    LoadFailed,
    SourceCubemapRequiresPrefiltering,
    Dimension,
    Extent,
    FaceCount,
    MipCount,
    Payload,
    PayloadLength,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ReflectionProbeAssetRejection {
    pub(super) cubemap: ResourceId,
    pub(super) reason: ReflectionProbeAssetRejectionReason,
}

#[derive(Debug, Error)]
pub(super) enum ReflectionProbeAssetError {
    #[error("reflection probe cubemap resource {cubemap} is not registered")]
    MissingResource { cubemap: ResourceId },
    #[error("failed to load reflection probe cubemap resource {cubemap}")]
    Load {
        cubemap: ResourceId,
        #[source]
        source: crate::core::CoreError,
    },
    #[error(
        "reflection probe cubemap {cubemap} contains source mips and must be prefiltered before sampling"
    )]
    SourceCubemapRequiresPrefiltering { cubemap: ResourceId },
    #[error("reflection probe cubemap {cubemap} must use the Cube dimension, found {actual:?}")]
    Dimension {
        cubemap: ResourceId,
        actual: RenderImageDimension,
    },
    #[error(
        "reflection probe cubemap {cubemap} must be {expected}x{expected}, found {width}x{height}"
    )]
    Extent {
        cubemap: ResourceId,
        expected: u32,
        width: u32,
        height: u32,
    },
    #[error("reflection probe cubemap {cubemap} must contain six faces, found {actual}")]
    FaceCount { cubemap: ResourceId, actual: u32 },
    #[error(
        "reflection probe cubemap {cubemap} must contain {expected} PMREM mips, found {actual}"
    )]
    MipCount {
        cubemap: ResourceId,
        expected: u32,
        actual: u32,
    },
    #[error("reflection probe cubemap {cubemap} must use the current RGBA16F PMREM payload")]
    Payload { cubemap: ResourceId },
    #[error(
        "reflection probe cubemap {cubemap} rgba8 payload length mismatch: expected {expected}, found {actual}"
    )]
    PayloadLength {
        cubemap: ResourceId,
        expected: usize,
        actual: usize,
    },
}

impl ReflectionProbeAssetError {
    pub(super) fn rejection(&self) -> ReflectionProbeAssetRejection {
        let (cubemap, reason) = match self {
            Self::MissingResource { cubemap } => (
                *cubemap,
                ReflectionProbeAssetRejectionReason::MissingResource,
            ),
            Self::Load { cubemap, .. } => {
                (*cubemap, ReflectionProbeAssetRejectionReason::LoadFailed)
            }
            Self::SourceCubemapRequiresPrefiltering { cubemap } => (
                *cubemap,
                ReflectionProbeAssetRejectionReason::SourceCubemapRequiresPrefiltering,
            ),
            Self::Dimension { cubemap, .. } => {
                (*cubemap, ReflectionProbeAssetRejectionReason::Dimension)
            }
            Self::Extent { cubemap, .. } => (*cubemap, ReflectionProbeAssetRejectionReason::Extent),
            Self::FaceCount { cubemap, .. } => {
                (*cubemap, ReflectionProbeAssetRejectionReason::FaceCount)
            }
            Self::MipCount { cubemap, .. } => {
                (*cubemap, ReflectionProbeAssetRejectionReason::MipCount)
            }
            Self::Payload { cubemap } => (*cubemap, ReflectionProbeAssetRejectionReason::Payload),
            Self::PayloadLength { cubemap, .. } => {
                (*cubemap, ReflectionProbeAssetRejectionReason::PayloadLength)
            }
        };
        ReflectionProbeAssetRejection { cubemap, reason }
    }
}

pub(super) fn validate_probe_pmrem_texture<'a>(
    cubemap: ResourceId,
    texture: &'a TextureAsset,
) -> Result<&'a [u8], ReflectionProbeAssetError> {
    if is_zcube_source_cubemap_texture(texture) {
        return Err(ReflectionProbeAssetError::SourceCubemapRequiresPrefiltering { cubemap });
    }
    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension != RenderImageDimension::Cube {
        return Err(ReflectionProbeAssetError::Dimension {
            cubemap,
            actual: descriptor.dimension,
        });
    }
    if texture.width != REFLECTION_PROBE_FACE_SIZE || texture.height != REFLECTION_PROBE_FACE_SIZE {
        return Err(ReflectionProbeAssetError::Extent {
            cubemap,
            expected: REFLECTION_PROBE_FACE_SIZE,
            width: texture.width,
            height: texture.height,
        });
    }
    if descriptor.array_layer_count != REFLECTION_PROBE_FACE_COUNT
        || descriptor.depth_or_array_layers != REFLECTION_PROBE_FACE_COUNT
    {
        return Err(ReflectionProbeAssetError::FaceCount {
            cubemap,
            actual: descriptor.array_layer_count,
        });
    }
    if descriptor.mip_count != REFLECTION_PROBE_MIP_COUNT {
        return Err(ReflectionProbeAssetError::MipCount {
            cubemap,
            expected: REFLECTION_PROBE_MIP_COUNT,
            actual: descriptor.mip_count,
        });
    }
    let bytes = decode_ibl_pmrem_rgba16f_texture(texture).map_err(|error| match error {
        IblPmremTextureError::PayloadLength { expected, actual } => {
            ReflectionProbeAssetError::PayloadLength {
                cubemap,
                expected,
                actual,
            }
        }
        _ => ReflectionProbeAssetError::Payload { cubemap },
    })?;
    let expected =
        rgba16f_cube_mip_chain_len(REFLECTION_PROBE_FACE_SIZE, REFLECTION_PROBE_MIP_COUNT);
    if bytes.len() != expected {
        return Err(ReflectionProbeAssetError::PayloadLength {
            cubemap,
            expected,
            actual: bytes.len(),
        });
    }
    Ok(bytes)
}

pub(super) fn append_probe_pmrem_texture_uploads(
    batch: &mut WgpuTextureUploadBatch,
    destination: &wgpu::Texture,
    array_slice: u32,
    bytes: &[u8],
) {
    let payload: Arc<[u8]> = Arc::from(bytes);
    let mut mip_base = 0_usize;
    for mip_level in 0..REFLECTION_PROBE_MIP_COUNT {
        let mip_size = (REFLECTION_PROBE_FACE_SIZE >> mip_level).max(1);
        let face_byte_len = mip_size as usize * mip_size as usize * 8;
        let mip_byte_len = face_byte_len * REFLECTION_PROBE_FACE_COUNT as usize;
        let source_range = mip_base..mip_base + mip_byte_len;
        batch.push(
            WgpuTextureUpload::new(
                destination.clone(),
                TextureCopyRegion::new(mip_size, mip_size)
                    .with_mip_level(mip_level)
                    .with_origin(0, 0, array_slice * REFLECTION_PROBE_FACE_COUNT)
                    .with_depth_or_array_layers(REFLECTION_PROBE_FACE_COUNT),
                mip_size * 8,
                mip_size,
                Arc::clone(&payload),
                source_range,
            )
            .expect("validated probe PMREM mip range must reference its shared payload"),
        );
        mip_base += mip_byte_len;
    }
    debug_assert_eq!(mip_base, payload.len());
}

fn rgba16f_cube_mip_chain_len(face_size: u32, mip_count: u32) -> usize {
    (0..mip_count)
        .map(|mip| {
            let mip_size = (face_size >> mip).max(1) as usize;
            mip_size * mip_size * 8 * REFLECTION_PROBE_FACE_COUNT as usize
        })
        .sum()
}
