use thiserror::Error;

use crate::asset::AssetUri;
use crate::core::framework::render::{
    IBL_BAKE_ALGORITHM_VERSION, IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES, IblBakeArtifactBlob,
    RenderImageColorSpace, RenderImageDimension, SOURCE_CUBEMAP_FACE_COUNT,
};

use super::{TextureAsset, TextureAssetDescriptor, TexturePayload};

pub const IBL_PMREM_RGBA16F_FORMAT: &str = "zircon/ibl-pmrem-rgba16f-v1";
pub const IBL_PMREM_RGBA16F_GPU_FORMAT: &str = "rgba16float";

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum IblPmremTextureError {
    #[error("IBL bake artifact does not contain a PMREM section")]
    MissingPmrem,
    #[error("IBL bake artifact algorithm version {actual} is stale; current version is {expected}")]
    StaleAlgorithmVersion { expected: u64, actual: u64 },
    #[error("texture payload is not the Zircon RGBA16F PMREM container")]
    NotPmremContainer,
    #[error("PMREM texture must use the Cube dimension, found {actual:?}")]
    Dimension { actual: RenderImageDimension },
    #[error("PMREM texture must contain six faces, found {actual}")]
    FaceCount { actual: u32 },
    #[error(
        "PMREM texture must use linear rgba16float, found format {format} and color space {color_space:?}"
    )]
    Descriptor {
        format: String,
        color_space: RenderImageColorSpace,
    },
    #[error("PMREM rgba16f payload length mismatch: expected {expected}, found {actual}")]
    PayloadLength { expected: usize, actual: usize },
    #[error("PMREM texture extent is too large")]
    ExtentOverflow,
}

pub fn texture_asset_from_ibl_bake_artifact_pmrem(
    uri: AssetUri,
    blob: &IblBakeArtifactBlob,
) -> Result<TextureAsset, IblPmremTextureError> {
    let descriptor = blob.descriptor();
    if descriptor.algorithm_version() != IBL_BAKE_ALGORITHM_VERSION {
        return Err(IblPmremTextureError::StaleAlgorithmVersion {
            expected: IBL_BAKE_ALGORITHM_VERSION,
            actual: descriptor.algorithm_version(),
        });
    }
    let range = blob
        .payload()
        .pmrem_rgba16f_byte_range()
        .ok_or(IblPmremTextureError::MissingPmrem)?;
    let bytes = blob.payload().bytes()[range].to_vec();
    let mut texture_descriptor = TextureAssetDescriptor::container(
        IBL_PMREM_RGBA16F_GPU_FORMAT,
        descriptor.mip_count(),
        SOURCE_CUBEMAP_FACE_COUNT as u32,
    );
    texture_descriptor.color_space = RenderImageColorSpace::Linear;
    texture_descriptor.dimension = RenderImageDimension::Cube;
    texture_descriptor.depth_or_array_layers = SOURCE_CUBEMAP_FACE_COUNT as u32;
    texture_descriptor.array_layer_count = SOURCE_CUBEMAP_FACE_COUNT as u32;

    let texture = TextureAsset::new_container(
        uri,
        descriptor.face_size(),
        descriptor.face_size(),
        IBL_PMREM_RGBA16F_FORMAT,
        bytes,
        descriptor.mip_count(),
        SOURCE_CUBEMAP_FACE_COUNT as u32,
    )
    .with_descriptor(texture_descriptor);
    decode_ibl_pmrem_rgba16f_texture(&texture)?;
    Ok(texture)
}

pub fn is_ibl_pmrem_rgba16f_texture(texture: &TextureAsset) -> bool {
    matches!(
        &texture.payload,
        TexturePayload::Container { format, .. } if format == IBL_PMREM_RGBA16F_FORMAT
    )
}

pub fn decode_ibl_pmrem_rgba16f_texture(
    texture: &TextureAsset,
) -> Result<&[u8], IblPmremTextureError> {
    let TexturePayload::Container {
        format,
        bytes,
        mip_count,
        array_layers,
    } = &texture.payload
    else {
        return Err(IblPmremTextureError::NotPmremContainer);
    };
    if format != IBL_PMREM_RGBA16F_FORMAT {
        return Err(IblPmremTextureError::NotPmremContainer);
    }

    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension != RenderImageDimension::Cube {
        return Err(IblPmremTextureError::Dimension {
            actual: descriptor.dimension,
        });
    }
    if *array_layers != SOURCE_CUBEMAP_FACE_COUNT as u32
        || descriptor.array_layer_count != SOURCE_CUBEMAP_FACE_COUNT as u32
        || descriptor.depth_or_array_layers != SOURCE_CUBEMAP_FACE_COUNT as u32
    {
        return Err(IblPmremTextureError::FaceCount {
            actual: descriptor.array_layer_count,
        });
    }
    if descriptor.format != IBL_PMREM_RGBA16F_GPU_FORMAT
        || descriptor.color_space != RenderImageColorSpace::Linear
    {
        return Err(IblPmremTextureError::Descriptor {
            format: descriptor.format,
            color_space: descriptor.color_space,
        });
    }
    let expected = rgba16f_cube_mip_chain_len(texture.width, *mip_count)
        .ok_or(IblPmremTextureError::ExtentOverflow)?;
    if bytes.len() != expected {
        return Err(IblPmremTextureError::PayloadLength {
            expected,
            actual: bytes.len(),
        });
    }
    Ok(bytes)
}

fn rgba16f_cube_mip_chain_len(face_size: u32, mip_count: u32) -> Option<usize> {
    let mut texel_count = 0_usize;
    for mip in 0..mip_count {
        let mip_size = usize::try_from((face_size >> mip).max(1)).ok()?;
        texel_count = texel_count.checked_add(
            mip_size
                .checked_mul(mip_size)?
                .checked_mul(SOURCE_CUBEMAP_FACE_COUNT)?,
        )?;
    }
    texel_count.checked_mul(IBL_BAKE_ARTIFACT_RGBA16F_TEXEL_SIZE_BYTES)
}
