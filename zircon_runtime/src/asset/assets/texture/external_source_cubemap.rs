use thiserror::Error;

use crate::core::framework::render::{
    RenderImageDimension, SOURCE_CUBEMAP_FACE_COUNT, source_cubemap_mip_count,
};

use super::{TextureAsset, TexturePayload};

mod decode;

pub use decode::{ExternalSourceCubemapDecodeError, decode_external_source_cubemap};

const DDS_HEADER_SIZE: usize = 128;
const DDS_DX10_HEADER_SIZE: usize = 148;
const DDSD_MIPMAPCOUNT: u32 = 0x0002_0000;
const DDSCAPS_COMPLEX: u32 = 0x0000_0008;
const DDSCAPS2_CUBEMAP: u32 = 0x0000_0200;
const DDSCAPS2_CUBEMAP_POSITIVEX: u32 = 0x0000_0400;
const DDSCAPS2_CUBEMAP_NEGATIVEX: u32 = 0x0000_0800;
const DDSCAPS2_CUBEMAP_POSITIVEY: u32 = 0x0000_1000;
const DDSCAPS2_CUBEMAP_NEGATIVEY: u32 = 0x0000_2000;
const DDSCAPS2_CUBEMAP_POSITIVEZ: u32 = 0x0000_4000;
const DDSCAPS2_CUBEMAP_NEGATIVEZ: u32 = 0x0000_8000;
const DDSCAPS2_CUBEMAP_ALL_FACES: u32 = DDSCAPS2_CUBEMAP
    | DDSCAPS2_CUBEMAP_POSITIVEX
    | DDSCAPS2_CUBEMAP_NEGATIVEX
    | DDSCAPS2_CUBEMAP_POSITIVEY
    | DDSCAPS2_CUBEMAP_NEGATIVEY
    | DDSCAPS2_CUBEMAP_POSITIVEZ
    | DDSCAPS2_CUBEMAP_NEGATIVEZ;
const DDS_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;

const KTX1_IDENTIFIER: &[u8] = b"\xABKTX 11\xBB\r\n\x1A\n";
const KTX2_IDENTIFIER: &[u8] = b"\xABKTX 20\xBB\r\n\x1A\n";

/// External cubemap containers carry source environment mips for the IBL baker.
///
/// They are intentionally classified before compressed upload planning so a
/// valid DDS/KTX cubemap cannot be consumed as a material texture or PMREM blob.
pub const EXTERNAL_SOURCE_CUBEMAP_UPLOAD_UNSUPPORTED_REASON: &str = "external DDS/KTX cubemap containers are source cubemap inputs for IBL baking, not PMREM or direct texture upload payloads";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExternalSourceCubemapContainerKind {
    Dds,
    Ktx1,
    Ktx2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalSourceCubemapContainerInfo {
    pub kind: ExternalSourceCubemapContainerKind,
    pub format: String,
    pub face_size: u32,
    pub mip_count: u32,
}

/// Classifies external DDS/KTX cubemap containers that are valid source inputs.
///
/// The check validates header shape and texture metadata only. It does not
/// decode pixels or convert the container; the follow-up importer/bake stage is
/// responsible for creating the `.zcube` source-cubemap representation.
pub fn external_source_cubemap_container_info(
    texture: &TextureAsset,
) -> Result<Option<ExternalSourceCubemapContainerInfo>, ExternalSourceCubemapContainerError> {
    let TexturePayload::Container {
        format,
        bytes,
        mip_count,
        array_layers,
    } = &texture.payload
    else {
        return Ok(None);
    };

    let normalized = format.trim().to_ascii_lowercase();
    let Some(header) = parse_external_cubemap_header(&normalized, bytes)? else {
        return Ok(None);
    };
    validate_texture_metadata(texture, *mip_count, *array_layers, header)?;
    Ok(Some(ExternalSourceCubemapContainerInfo {
        kind: header.kind,
        format: normalized,
        face_size: header.face_size,
        mip_count: header.mip_count,
    }))
}

pub fn is_external_source_cubemap_container(texture: &TextureAsset) -> bool {
    matches!(external_source_cubemap_container_info(texture), Ok(Some(_)))
}

fn parse_external_cubemap_header(
    format: &str,
    bytes: &[u8],
) -> Result<Option<ExternalSourceCubemapHeader>, ExternalSourceCubemapContainerError> {
    if format.starts_with("dds/") {
        return parse_dds_source_cubemap_header(bytes);
    }
    if format.starts_with("ktx2/") {
        return parse_ktx2_source_cubemap_header(bytes);
    }
    if format.starts_with("ktx/") {
        return parse_ktx1_source_cubemap_header(bytes);
    }
    Ok(None)
}

fn parse_dds_source_cubemap_header(
    bytes: &[u8],
) -> Result<Option<ExternalSourceCubemapHeader>, ExternalSourceCubemapContainerError> {
    require_len(
        ExternalSourceCubemapContainerKind::Dds,
        bytes,
        DDS_HEADER_SIZE,
    )?;
    if bytes.get(..4) != Some(b"DDS ") {
        return Err(invalid_header(
            ExternalSourceCubemapContainerKind::Dds,
            "missing DDS magic",
        ));
    }

    let height = read_u32_le(bytes, 12, ExternalSourceCubemapContainerKind::Dds)?;
    let width = read_u32_le(bytes, 16, ExternalSourceCubemapContainerKind::Dds)?;
    let mip_count = dds_mip_count(bytes)?;
    let caps = read_u32_le(bytes, 108, ExternalSourceCubemapContainerKind::Dds)?;
    let caps2 = read_u32_le(bytes, 112, ExternalSourceCubemapContainerKind::Dds)?;
    let caps2_cubemap = caps2 & DDSCAPS2_CUBEMAP != 0;
    let dx10 = bytes.get(84..88) == Some(b"DX10");

    let (is_cubemap, array_layers) = if dx10 {
        require_len(
            ExternalSourceCubemapContainerKind::Dds,
            bytes,
            DDS_DX10_HEADER_SIZE,
        )?;
        let misc_flag = read_u32_le(bytes, 136, ExternalSourceCubemapContainerKind::Dds)?;
        let dx10_cubemap = misc_flag & DDS_RESOURCE_MISC_TEXTURECUBE != 0;
        if caps2_cubemap && dx10_cubemap {
            return Err(invalid_header(
                ExternalSourceCubemapContainerKind::Dds,
                "cubemap declared by both DDS caps2 and DX10 misc flag",
            ));
        }
        let array_size = read_u32_le(bytes, 140, ExternalSourceCubemapContainerKind::Dds)?;
        (
            caps2_cubemap || dx10_cubemap,
            array_size
                .max(1)
                .saturating_mul(SOURCE_CUBEMAP_FACE_COUNT as u32),
        )
    } else {
        (
            caps2_cubemap,
            if caps2_cubemap {
                SOURCE_CUBEMAP_FACE_COUNT as u32
            } else {
                1
            },
        )
    };

    if !is_cubemap {
        return Ok(None);
    }
    if caps & DDSCAPS_COMPLEX == 0 {
        return Err(invalid_header(
            ExternalSourceCubemapContainerKind::Dds,
            "cubemap must set DDSCAPS_COMPLEX",
        ));
    }
    if caps2_cubemap && caps2 & DDSCAPS2_CUBEMAP_ALL_FACES != DDSCAPS2_CUBEMAP_ALL_FACES {
        return Err(invalid_header(
            ExternalSourceCubemapContainerKind::Dds,
            "DDS caps2 cubemap must declare all six face flags",
        ));
    }

    validate_dds_source_pixel_format(bytes, dx10)?;
    Ok(Some(ExternalSourceCubemapHeader {
        kind: ExternalSourceCubemapContainerKind::Dds,
        face_size: square_face_size(ExternalSourceCubemapContainerKind::Dds, width, height)?,
        mip_count,
        array_layers,
    }))
}

fn parse_ktx1_source_cubemap_header(
    bytes: &[u8],
) -> Result<Option<ExternalSourceCubemapHeader>, ExternalSourceCubemapContainerError> {
    require_len(ExternalSourceCubemapContainerKind::Ktx1, bytes, 64)?;
    if bytes.get(..KTX1_IDENTIFIER.len()) != Some(KTX1_IDENTIFIER) {
        return Err(invalid_header(
            ExternalSourceCubemapContainerKind::Ktx1,
            "missing KTX 1 identifier",
        ));
    }

    let width = read_u32_le(bytes, 36, ExternalSourceCubemapContainerKind::Ktx1)?;
    let height = read_u32_le(bytes, 40, ExternalSourceCubemapContainerKind::Ktx1)?;
    let depth = read_u32_le(bytes, 44, ExternalSourceCubemapContainerKind::Ktx1)?;
    let array_elements = read_u32_le(bytes, 48, ExternalSourceCubemapContainerKind::Ktx1)?.max(1);
    let face_count = read_u32_le(bytes, 52, ExternalSourceCubemapContainerKind::Ktx1)?;
    let mip_count = read_u32_le(bytes, 56, ExternalSourceCubemapContainerKind::Ktx1)?.max(1);

    if face_count == 1 {
        return Ok(None);
    }
    validate_ktx_cubemap_header(ExternalSourceCubemapContainerKind::Ktx1, face_count, depth)?;
    validate_ktx1_source_pixel_format(bytes)?;

    Ok(Some(ExternalSourceCubemapHeader {
        kind: ExternalSourceCubemapContainerKind::Ktx1,
        face_size: square_face_size(ExternalSourceCubemapContainerKind::Ktx1, width, height)?,
        mip_count,
        array_layers: array_elements.saturating_mul(face_count),
    }))
}

fn parse_ktx2_source_cubemap_header(
    bytes: &[u8],
) -> Result<Option<ExternalSourceCubemapHeader>, ExternalSourceCubemapContainerError> {
    require_len(ExternalSourceCubemapContainerKind::Ktx2, bytes, 80)?;
    if bytes.get(..KTX2_IDENTIFIER.len()) != Some(KTX2_IDENTIFIER) {
        return Err(invalid_header(
            ExternalSourceCubemapContainerKind::Ktx2,
            "missing KTX 2 identifier",
        ));
    }

    let width = read_u32_le(bytes, 20, ExternalSourceCubemapContainerKind::Ktx2)?;
    let height = read_u32_le(bytes, 24, ExternalSourceCubemapContainerKind::Ktx2)?;
    let depth = read_u32_le(bytes, 28, ExternalSourceCubemapContainerKind::Ktx2)?;
    let layer_count = read_u32_le(bytes, 32, ExternalSourceCubemapContainerKind::Ktx2)?.max(1);
    let face_count = read_u32_le(bytes, 36, ExternalSourceCubemapContainerKind::Ktx2)?;
    let mip_count = read_u32_le(bytes, 40, ExternalSourceCubemapContainerKind::Ktx2)?.max(1);

    if face_count == 1 {
        return Ok(None);
    }
    validate_ktx_cubemap_header(ExternalSourceCubemapContainerKind::Ktx2, face_count, depth)?;
    validate_ktx2_source_pixel_format(bytes)?;

    Ok(Some(ExternalSourceCubemapHeader {
        kind: ExternalSourceCubemapContainerKind::Ktx2,
        face_size: square_face_size(ExternalSourceCubemapContainerKind::Ktx2, width, height)?,
        mip_count,
        array_layers: layer_count.saturating_mul(face_count),
    }))
}

fn validate_dds_source_pixel_format(
    bytes: &[u8],
    dx10: bool,
) -> Result<(), ExternalSourceCubemapContainerError> {
    let supported = if dx10 {
        matches!(
            read_u32_le(bytes, 128, ExternalSourceCubemapContainerKind::Dds)?,
            2 | 10
        )
    } else {
        matches!(
            read_u32_le(bytes, 84, ExternalSourceCubemapContainerKind::Dds)?,
            113 | 116
        )
    };
    if supported {
        return Ok(());
    }
    Err(
        ExternalSourceCubemapContainerError::UnsupportedSourcePixelFormat {
            kind: ExternalSourceCubemapContainerKind::Dds,
            format: "expected RGBA16F or RGBA32F DDS source cubemap".to_string(),
        },
    )
}

fn validate_ktx1_source_pixel_format(
    bytes: &[u8],
) -> Result<(), ExternalSourceCubemapContainerError> {
    let internal_format = read_u32_le(bytes, 28, ExternalSourceCubemapContainerKind::Ktx1)?;
    if matches!(internal_format, 0x881a | 0x8814) {
        return Ok(());
    }
    Err(
        ExternalSourceCubemapContainerError::UnsupportedSourcePixelFormat {
            kind: ExternalSourceCubemapContainerKind::Ktx1,
            format: format!("GL internal format 0x{internal_format:08x}; expected RGBA16F/RGBA32F"),
        },
    )
}

fn validate_ktx2_source_pixel_format(
    bytes: &[u8],
) -> Result<(), ExternalSourceCubemapContainerError> {
    let vk_format = read_u32_le(bytes, 12, ExternalSourceCubemapContainerKind::Ktx2)?;
    let supercompression = read_u32_le(bytes, 44, ExternalSourceCubemapContainerKind::Ktx2)?;
    if matches!(vk_format, 97 | 109) && supercompression == 0 {
        return Ok(());
    }
    Err(
        ExternalSourceCubemapContainerError::UnsupportedSourcePixelFormat {
            kind: ExternalSourceCubemapContainerKind::Ktx2,
            format: format!(
                "Vulkan format {vk_format}, supercompression {supercompression}; expected uncompressed RGBA16F/RGBA32F"
            ),
        },
    )
}

fn validate_texture_metadata(
    texture: &TextureAsset,
    payload_mip_count: u32,
    payload_array_layers: u32,
    header: ExternalSourceCubemapHeader,
) -> Result<(), ExternalSourceCubemapContainerError> {
    if header.array_layers != SOURCE_CUBEMAP_FACE_COUNT as u32 {
        return Err(
            ExternalSourceCubemapContainerError::UnsupportedCubemapArray {
                kind: header.kind,
                array_layers: header.array_layers,
            },
        );
    }
    if !header.face_size.is_power_of_two() {
        return Err(ExternalSourceCubemapContainerError::NonPowerOfTwoFaceSize {
            kind: header.kind,
            face_size: header.face_size,
        });
    }
    let expected_mip_count = source_cubemap_mip_count(header.face_size);
    if header.mip_count != expected_mip_count {
        return Err(ExternalSourceCubemapContainerError::IncompleteMipChain {
            kind: header.kind,
            face_size: header.face_size,
            expected: expected_mip_count,
            actual: header.mip_count,
        });
    }
    if texture.width != header.face_size || texture.height != header.face_size {
        return Err(
            ExternalSourceCubemapContainerError::TextureMetadataMismatch {
                kind: header.kind,
                field: "extent",
                expected: format!("{}x{}", header.face_size, header.face_size),
                actual: format!("{}x{}", texture.width, texture.height),
            },
        );
    }
    if payload_mip_count != header.mip_count {
        return Err(
            ExternalSourceCubemapContainerError::TextureMetadataMismatch {
                kind: header.kind,
                field: "payload_mip_count",
                expected: header.mip_count.to_string(),
                actual: payload_mip_count.to_string(),
            },
        );
    }
    if payload_array_layers != header.array_layers {
        return Err(
            ExternalSourceCubemapContainerError::TextureMetadataMismatch {
                kind: header.kind,
                field: "payload_array_layers",
                expected: header.array_layers.to_string(),
                actual: payload_array_layers.to_string(),
            },
        );
    }

    let descriptor = texture.render_image_descriptor();
    if !matches!(
        descriptor.dimension,
        RenderImageDimension::D2 | RenderImageDimension::Cube
    ) {
        return Err(
            ExternalSourceCubemapContainerError::TextureMetadataMismatch {
                kind: header.kind,
                field: "dimension",
                expected: "2d or cube".to_string(),
                actual: format!("{:?}", descriptor.dimension),
            },
        );
    }
    if descriptor.array_layer_count != header.array_layers
        || descriptor.depth_or_array_layers != header.array_layers
    {
        return Err(
            ExternalSourceCubemapContainerError::TextureMetadataMismatch {
                kind: header.kind,
                field: "descriptor_layers",
                expected: header.array_layers.to_string(),
                actual: format!(
                    "{}/{}",
                    descriptor.array_layer_count, descriptor.depth_or_array_layers
                ),
            },
        );
    }
    if descriptor.mip_count != header.mip_count {
        return Err(
            ExternalSourceCubemapContainerError::TextureMetadataMismatch {
                kind: header.kind,
                field: "descriptor_mip_count",
                expected: header.mip_count.to_string(),
                actual: descriptor.mip_count.to_string(),
            },
        );
    }

    Ok(())
}

fn dds_mip_count(bytes: &[u8]) -> Result<u32, ExternalSourceCubemapContainerError> {
    let flags = read_u32_le(bytes, 8, ExternalSourceCubemapContainerKind::Dds)?;
    if flags & DDSD_MIPMAPCOUNT == 0 {
        return Ok(1);
    }
    Ok(read_u32_le(bytes, 28, ExternalSourceCubemapContainerKind::Dds)?.max(1))
}

fn validate_ktx_cubemap_header(
    kind: ExternalSourceCubemapContainerKind,
    face_count: u32,
    depth: u32,
) -> Result<(), ExternalSourceCubemapContainerError> {
    if face_count != SOURCE_CUBEMAP_FACE_COUNT as u32 {
        return Err(invalid_header(
            kind,
            format!("cubemap face count must be 6, got {face_count}"),
        ));
    }
    if depth != 0 {
        return Err(invalid_header(kind, "cubemap must not declare 3d depth"));
    }
    Ok(())
}

fn square_face_size(
    kind: ExternalSourceCubemapContainerKind,
    width: u32,
    height: u32,
) -> Result<u32, ExternalSourceCubemapContainerError> {
    if width == 0 || height == 0 || width != height {
        return Err(invalid_header(
            kind,
            format!("cubemap faces must be square 2d images, got {width}x{height}"),
        ));
    }
    Ok(width)
}

fn require_len(
    kind: ExternalSourceCubemapContainerKind,
    bytes: &[u8],
    expected: usize,
) -> Result<(), ExternalSourceCubemapContainerError> {
    if bytes.len() < expected {
        return Err(ExternalSourceCubemapContainerError::TruncatedHeader {
            kind,
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}

fn read_u32_le(
    bytes: &[u8],
    offset: usize,
    kind: ExternalSourceCubemapContainerKind,
) -> Result<u32, ExternalSourceCubemapContainerError> {
    let value = bytes.get(offset..offset + 4).ok_or(
        ExternalSourceCubemapContainerError::TruncatedHeader {
            kind,
            expected: offset + 4,
            actual: bytes.len(),
        },
    )?;
    Ok(u32::from_le_bytes(
        value.try_into().expect("slice length checked"),
    ))
}

fn invalid_header(
    kind: ExternalSourceCubemapContainerKind,
    reason: impl Into<String>,
) -> ExternalSourceCubemapContainerError {
    ExternalSourceCubemapContainerError::InvalidHeader {
        kind,
        reason: reason.into(),
    }
}

#[derive(Clone, Copy, Debug)]
struct ExternalSourceCubemapHeader {
    kind: ExternalSourceCubemapContainerKind,
    face_size: u32,
    mip_count: u32,
    array_layers: u32,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ExternalSourceCubemapContainerError {
    #[error(
        "{kind:?} external source cubemap header is too short: expected at least {expected} bytes, found {actual}"
    )]
    TruncatedHeader {
        kind: ExternalSourceCubemapContainerKind,
        expected: usize,
        actual: usize,
    },
    #[error("{kind:?} external source cubemap header is invalid: {reason}")]
    InvalidHeader {
        kind: ExternalSourceCubemapContainerKind,
        reason: String,
    },
    #[error("{kind:?} external source cubemap pixel format is unsupported: {format}")]
    UnsupportedSourcePixelFormat {
        kind: ExternalSourceCubemapContainerKind,
        format: String,
    },
    #[error(
        "{kind:?} external source cubemap arrays are not supported by .zcube: array_layers={array_layers}"
    )]
    UnsupportedCubemapArray {
        kind: ExternalSourceCubemapContainerKind,
        array_layers: u32,
    },
    #[error("{kind:?} external source cubemap face size must be power-of-two, found {face_size}")]
    NonPowerOfTwoFaceSize {
        kind: ExternalSourceCubemapContainerKind,
        face_size: u32,
    },
    #[error(
        "{kind:?} external source cubemap mip chain is incomplete for face_size={face_size}: expected {expected}, found {actual}"
    )]
    IncompleteMipChain {
        kind: ExternalSourceCubemapContainerKind,
        face_size: u32,
        expected: u32,
        actual: u32,
    },
    #[error(
        "{kind:?} external source cubemap texture metadata mismatch for {field}: expected {expected}, found {actual}"
    )]
    TextureMetadataMismatch {
        kind: ExternalSourceCubemapContainerKind,
        field: &'static str,
        expected: String,
        actual: String,
    },
}
