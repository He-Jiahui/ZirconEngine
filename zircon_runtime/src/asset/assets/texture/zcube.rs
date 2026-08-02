use thiserror::Error;

use crate::asset::AssetUri;
use crate::core::framework::render::{
    RGBA16F_TEXEL_SIZE_BYTES, RenderImageColorSpace, RenderImageDimension,
    SOURCE_CUBEMAP_FACE_COUNT, SourceCubemapMipChain, decode_rgba16f_texels, encode_rgba16f_texels,
    source_cubemap_mip_count,
};
use crate::core::math::Real;

use super::{TextureAsset, TextureAssetDescriptor, TexturePayload};

const ZCUBE_SOURCE_CUBEMAP_MAGIC: [u8; 8] = *b"ZRZCUBE1";
const ZCUBE_SOURCE_CUBEMAP_FORMAT_VERSION: u32 = 1;
const ZCUBE_TEXEL_FORMAT_RGBA16F: u32 = 1;
const ZCUBE_CONTENTS_SOURCE_MIPS: u32 = 1;

pub const ZCUBE_SOURCE_CUBEMAP_FORMAT: &str = "zircon/zcube-source-cubemap-rgba16f-v1";
pub const ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT: &str = "rgba16float";
pub const ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE: usize = 32;

#[derive(Clone, Debug, PartialEq)]
pub struct ZcubeSourceCubemap {
    face_size: u32,
    mip_count: u32,
    texels: Vec<[Real; 4]>,
}

impl ZcubeSourceCubemap {
    pub fn face_size(&self) -> u32 {
        self.face_size
    }

    pub fn mip_count(&self) -> u32 {
        self.mip_count
    }

    pub fn texels(&self) -> &[[Real; 4]] {
        &self.texels
    }
}

pub fn texture_asset_from_source_cubemap_zcube(
    uri: AssetUri,
    cubemap: &SourceCubemapMipChain,
) -> TextureAsset {
    let mut bytes = ZcubeSourceCubemapHeader {
        face_size: cubemap.source_face_size(),
        mip_count: cubemap.source_mip_count(),
    }
    .encode()
    .to_vec();
    bytes.extend_from_slice(&encode_rgba16f_texels(cubemap.source_texels()));

    TextureAsset::new_container(
        uri,
        cubemap.source_face_size(),
        cubemap.source_face_size(),
        ZCUBE_SOURCE_CUBEMAP_FORMAT,
        bytes,
        cubemap.source_mip_count(),
        SOURCE_CUBEMAP_FACE_COUNT as u32,
    )
    .with_descriptor(zcube_source_cubemap_descriptor(cubemap.source_mip_count()))
}

pub fn decode_zcube_source_cubemap_texture(
    texture: &TextureAsset,
) -> Result<ZcubeSourceCubemap, ZcubeSourceCubemapError> {
    let TexturePayload::Container {
        format,
        bytes,
        mip_count,
        array_layers,
    } = &texture.payload
    else {
        return Err(ZcubeSourceCubemapError::NotContainer);
    };

    if format != ZCUBE_SOURCE_CUBEMAP_FORMAT {
        return Err(ZcubeSourceCubemapError::UnsupportedContainerFormat {
            format: format.clone(),
        });
    }

    let cubemap = decode_zcube_source_cubemap_bytes(bytes)?;
    if texture.width != cubemap.face_size || texture.height != cubemap.face_size {
        return Err(ZcubeSourceCubemapError::TextureExtentMismatch {
            expected_face_size: cubemap.face_size,
            actual_width: texture.width,
            actual_height: texture.height,
        });
    }
    if *mip_count != cubemap.mip_count || *array_layers != SOURCE_CUBEMAP_FACE_COUNT as u32 {
        return Err(ZcubeSourceCubemapError::TextureContainerMetadataMismatch {
            expected_mip_count: cubemap.mip_count,
            actual_mip_count: *mip_count,
            expected_array_layers: SOURCE_CUBEMAP_FACE_COUNT as u32,
            actual_array_layers: *array_layers,
        });
    }

    Ok(cubemap)
}

/// Decodes a staged `.zcube` directly from its self-describing container.
/// This path deliberately does not borrow PMREM artifact dimensions.
pub fn decode_zcube_source_cubemap_bytes(
    bytes: &[u8],
) -> Result<ZcubeSourceCubemap, ZcubeSourceCubemapError> {
    let header = ZcubeSourceCubemapHeader::decode(bytes)?;

    let expected_payload_len = zcube_source_payload_len(header.face_size, header.mip_count).ok_or(
        ZcubeSourceCubemapError::ExtentTooLarge {
            face_size: header.face_size,
            mip_count: header.mip_count,
        },
    )?;
    let payload = &bytes[ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE..];
    if payload.len() != expected_payload_len {
        return Err(ZcubeSourceCubemapError::InvalidPayloadLength {
            expected: expected_payload_len,
            actual: payload.len(),
        });
    }

    Ok(ZcubeSourceCubemap {
        face_size: header.face_size,
        mip_count: header.mip_count,
        texels: decode_rgba16f_texels(payload),
    })
}

pub fn is_zcube_source_cubemap_texture(texture: &TextureAsset) -> bool {
    matches!(
        &texture.payload,
        TexturePayload::Container { format, .. } if format == ZCUBE_SOURCE_CUBEMAP_FORMAT
    )
}

fn zcube_source_cubemap_descriptor(mip_count: u32) -> TextureAssetDescriptor {
    let mut descriptor = TextureAssetDescriptor::container(
        ZCUBE_SOURCE_CUBEMAP_GPU_FORMAT,
        mip_count,
        SOURCE_CUBEMAP_FACE_COUNT as u32,
    );
    descriptor.color_space = RenderImageColorSpace::Linear;
    descriptor.metadata.color_space = RenderImageColorSpace::Linear;
    descriptor.dimension = RenderImageDimension::Cube;
    descriptor.depth_or_array_layers = SOURCE_CUBEMAP_FACE_COUNT as u32;
    descriptor.array_layer_count = SOURCE_CUBEMAP_FACE_COUNT as u32;
    descriptor.normalized()
}

fn zcube_source_payload_len(face_size: u32, mip_count: u32) -> Option<usize> {
    let mut per_face = 0_usize;
    let mut mip_size = face_size.max(1);
    for _ in 0..mip_count {
        let size = usize::try_from(mip_size).ok()?;
        per_face = per_face.checked_add(size.checked_mul(size)?)?;
        mip_size = (mip_size / 2).max(1);
    }
    per_face
        .checked_mul(SOURCE_CUBEMAP_FACE_COUNT)?
        .checked_mul(RGBA16F_TEXEL_SIZE_BYTES)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ZcubeSourceCubemapHeader {
    face_size: u32,
    mip_count: u32,
}

impl ZcubeSourceCubemapHeader {
    fn encode(self) -> [u8; ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE] {
        let mut bytes = [0; ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE];
        let mut cursor = 0;
        write_bytes(&mut bytes, &mut cursor, &ZCUBE_SOURCE_CUBEMAP_MAGIC);
        write_u32(&mut bytes, &mut cursor, ZCUBE_SOURCE_CUBEMAP_FORMAT_VERSION);
        write_u32(&mut bytes, &mut cursor, self.face_size);
        write_u32(&mut bytes, &mut cursor, self.mip_count);
        write_u32(&mut bytes, &mut cursor, SOURCE_CUBEMAP_FACE_COUNT as u32);
        write_u32(&mut bytes, &mut cursor, ZCUBE_TEXEL_FORMAT_RGBA16F);
        write_u32(&mut bytes, &mut cursor, ZCUBE_CONTENTS_SOURCE_MIPS);
        bytes
    }

    fn decode(bytes: &[u8]) -> Result<Self, ZcubeSourceCubemapError> {
        if bytes.len() < ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE {
            return Err(ZcubeSourceCubemapError::TruncatedHeader {
                expected: ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE,
                actual: bytes.len(),
            });
        }

        let mut cursor = 0;
        if read_bytes::<8>(bytes, &mut cursor) != ZCUBE_SOURCE_CUBEMAP_MAGIC {
            return Err(ZcubeSourceCubemapError::InvalidMagic);
        }
        let format_version = read_u32(bytes, &mut cursor);
        if format_version != ZCUBE_SOURCE_CUBEMAP_FORMAT_VERSION {
            return Err(ZcubeSourceCubemapError::UnsupportedFormatVersion(
                format_version,
            ));
        }
        let face_size = read_u32(bytes, &mut cursor);
        let mip_count = read_u32(bytes, &mut cursor);
        let face_count = read_u32(bytes, &mut cursor);
        let texel_format = read_u32(bytes, &mut cursor);
        let contents = read_u32(bytes, &mut cursor);

        if face_size == 0 || mip_count == 0 {
            return Err(ZcubeSourceCubemapError::InvalidLayout {
                face_size,
                mip_count,
            });
        }
        let expected_mip_count = source_cubemap_mip_count(face_size);
        if mip_count != expected_mip_count {
            return Err(ZcubeSourceCubemapError::InvalidMipCount {
                face_size,
                expected: expected_mip_count,
                actual: mip_count,
            });
        }
        if face_count != SOURCE_CUBEMAP_FACE_COUNT as u32 {
            return Err(ZcubeSourceCubemapError::UnsupportedFaceCount(face_count));
        }
        if texel_format != ZCUBE_TEXEL_FORMAT_RGBA16F {
            return Err(ZcubeSourceCubemapError::UnsupportedTexelFormat(
                texel_format,
            ));
        }
        if contents != ZCUBE_CONTENTS_SOURCE_MIPS {
            return Err(ZcubeSourceCubemapError::UnsupportedContents(contents));
        }

        Ok(Self {
            face_size,
            mip_count,
        })
    }
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZcubeSourceCubemapError {
    #[error("texture asset is not a container payload")]
    NotContainer,
    #[error("texture container format `{format}` is not Zircon source-cubemap .zcube")]
    UnsupportedContainerFormat { format: String },
    #[error("truncated .zcube header: expected at least {expected} bytes, found {actual}")]
    TruncatedHeader { expected: usize, actual: usize },
    #[error("invalid .zcube magic")]
    InvalidMagic,
    #[error("unsupported .zcube format version {0}")]
    UnsupportedFormatVersion(u32),
    #[error("invalid .zcube source layout face_size={face_size}, mip_count={mip_count}")]
    InvalidLayout { face_size: u32, mip_count: u32 },
    #[error(
        ".zcube mip count mismatch for face_size={face_size}: expected {expected}, found {actual}"
    )]
    InvalidMipCount {
        face_size: u32,
        expected: u32,
        actual: u32,
    },
    #[error("unsupported .zcube face count {0}")]
    UnsupportedFaceCount(u32),
    #[error("unsupported .zcube texel format {0}")]
    UnsupportedTexelFormat(u32),
    #[error("unsupported .zcube contents bitmask {0}")]
    UnsupportedContents(u32),
    #[error(".zcube extent is too large to validate: face_size={face_size}, mip_count={mip_count}")]
    ExtentTooLarge { face_size: u32, mip_count: u32 },
    #[error(".zcube payload length mismatch: expected {expected} bytes, found {actual}")]
    InvalidPayloadLength { expected: usize, actual: usize },
    #[error(
        ".zcube texture extent mismatch: expected square face {expected_face_size}, found {actual_width}x{actual_height}"
    )]
    TextureExtentMismatch {
        expected_face_size: u32,
        actual_width: u32,
        actual_height: u32,
    },
    #[error(
        ".zcube texture metadata mismatch: expected mip_count={expected_mip_count}, array_layers={expected_array_layers}; found mip_count={actual_mip_count}, array_layers={actual_array_layers}"
    )]
    TextureContainerMetadataMismatch {
        expected_mip_count: u32,
        actual_mip_count: u32,
        expected_array_layers: u32,
        actual_array_layers: u32,
    },
}

fn write_bytes(
    bytes: &mut [u8; ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE],
    cursor: &mut usize,
    value: &[u8],
) {
    let next = *cursor + value.len();
    bytes[*cursor..next].copy_from_slice(value);
    *cursor = next;
}

fn write_u32(bytes: &mut [u8; ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE], cursor: &mut usize, value: u32) {
    write_bytes(bytes, cursor, &value.to_le_bytes());
}

fn read_bytes<const N: usize>(bytes: &[u8], cursor: &mut usize) -> [u8; N] {
    let mut value = [0; N];
    let next = *cursor + N;
    value.copy_from_slice(&bytes[*cursor..next]);
    *cursor = next;
    value
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> u32 {
    u32::from_le_bytes(read_bytes(bytes, cursor))
}
