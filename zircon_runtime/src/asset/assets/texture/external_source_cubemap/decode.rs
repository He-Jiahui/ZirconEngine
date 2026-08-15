use thiserror::Error;

use super::{
    external_source_cubemap_container_info, ExternalSourceCubemapContainerError,
    ExternalSourceCubemapContainerInfo, ExternalSourceCubemapContainerKind,
};
use crate::asset::assets::{TextureAsset, TexturePayload};
use crate::core::framework::render::{
    build_source_cubemap_from_source_mips, decode_rgba16f_texels, source_cubemap_face_mip_offset,
    source_cubemap_mip_size, source_cubemap_sample_count, CubemapFace, SourceCubemapMipChain,
};

const DDS_HEADER_SIZE: usize = 128;
const DDS_DX10_HEADER_SIZE: usize = 148;
const D3DFMT_A16B16G16R16F: u32 = 113;
const D3DFMT_A32B32G32R32F: u32 = 116;
const DXGI_FORMAT_R32G32B32A32_FLOAT: u32 = 2;
const DXGI_FORMAT_R16G16B16A16_FLOAT: u32 = 10;

const KTX1_HEADER_SIZE: usize = 64;
const KTX1_LITTLE_ENDIAN: u32 = 0x0403_0201;
const GL_RGBA32F: u32 = 0x8814;
const GL_RGBA16F: u32 = 0x881a;

const KTX2_HEADER_SIZE: usize = 80;
const KTX2_LEVEL_INDEX_ENTRY_SIZE: usize = 24;
const VK_FORMAT_R16G16B16A16_SFLOAT: u32 = 97;
const VK_FORMAT_R32G32B32A32_SFLOAT: u32 = 109;

pub fn decode_external_source_cubemap(
    texture: &TextureAsset,
) -> Result<Option<SourceCubemapMipChain>, ExternalSourceCubemapDecodeError> {
    let Some(info) = external_source_cubemap_container_info(texture)? else {
        return Ok(None);
    };
    let TexturePayload::Container { bytes, .. } = &texture.payload else {
        return Ok(None);
    };
    let source_texels = match info.kind {
        ExternalSourceCubemapContainerKind::Dds => decode_dds(bytes, &info)?,
        ExternalSourceCubemapContainerKind::Ktx1 => decode_ktx1(bytes, &info)?,
        ExternalSourceCubemapContainerKind::Ktx2 => decode_ktx2(bytes, &info)?,
    };
    Ok(Some(build_source_cubemap_from_source_mips(
        info.face_size,
        info.mip_count,
        source_texels,
    )))
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ExternalSourceCubemapDecodeError {
    #[error(transparent)]
    Container(#[from] ExternalSourceCubemapContainerError),
    #[error("{kind:?} source cubemap pixel format is not supported for linear decode: {format}")]
    UnsupportedPixelFormat {
        kind: ExternalSourceCubemapContainerKind,
        format: String,
    },
    #[error(
        "{kind:?} source cubemap payload is truncated at byte {offset}: need through byte {required}, file has {actual} bytes"
    )]
    TruncatedPayload {
        kind: ExternalSourceCubemapContainerKind,
        offset: usize,
        required: usize,
        actual: usize,
    },
    #[error("{kind:?} source cubemap payload is invalid: {reason}")]
    InvalidPayload {
        kind: ExternalSourceCubemapContainerKind,
        reason: String,
    },
}

#[derive(Clone, Copy)]
enum SourceTexelFormat {
    Rgba16Float,
    Rgba32Float,
}

impl SourceTexelFormat {
    const fn bytes_per_texel(self) -> usize {
        match self {
            Self::Rgba16Float => 8,
            Self::Rgba32Float => 16,
        }
    }
}

fn decode_dds(
    bytes: &[u8],
    info: &ExternalSourceCubemapContainerInfo,
) -> Result<Vec<[f32; 4]>, ExternalSourceCubemapDecodeError> {
    require_range(info.kind, bytes, 0, DDS_HEADER_SIZE)?;
    let (data_offset, format) = if bytes.get(84..88) == Some(b"DX10") {
        require_range(info.kind, bytes, 0, DDS_DX10_HEADER_SIZE)?;
        let value = read_u32(bytes, 128, info.kind)?;
        let format = match value {
            DXGI_FORMAT_R16G16B16A16_FLOAT => SourceTexelFormat::Rgba16Float,
            DXGI_FORMAT_R32G32B32A32_FLOAT => SourceTexelFormat::Rgba32Float,
            _ => return Err(unsupported(info, format!("DXGI format {value}"))),
        };
        (DDS_DX10_HEADER_SIZE, format)
    } else {
        let value = read_u32(bytes, 84, info.kind)?;
        let format = match value {
            D3DFMT_A16B16G16R16F => SourceTexelFormat::Rgba16Float,
            D3DFMT_A32B32G32R32F => SourceTexelFormat::Rgba32Float,
            _ => return Err(unsupported(info, format!("pre-DX10 D3D format {value}"))),
        };
        (DDS_HEADER_SIZE, format)
    };

    let mut output = empty_source_texels(info);
    let mut cursor = data_offset;
    // DDS/c mft stores every face with its complete mip chain before the next face.
    for face in CubemapFace::ALL {
        for mip in 0..info.mip_count {
            let mip_size = source_cubemap_mip_size(info.face_size, mip);
            let byte_len = mip_size as usize * mip_size as usize * format.bytes_per_texel();
            let payload = require_range(info.kind, bytes, cursor, byte_len)?;
            write_face_mip(&mut output, info, face, mip, payload, format)?;
            cursor += byte_len;
        }
    }
    Ok(output)
}

fn decode_ktx1(
    bytes: &[u8],
    info: &ExternalSourceCubemapContainerInfo,
) -> Result<Vec<[f32; 4]>, ExternalSourceCubemapDecodeError> {
    require_range(info.kind, bytes, 0, KTX1_HEADER_SIZE)?;
    if read_u32(bytes, 12, info.kind)? != KTX1_LITTLE_ENDIAN {
        return Err(invalid(
            info.kind,
            "big-endian KTX1 payloads are not supported",
        ));
    }
    let internal_format = read_u32(bytes, 28, info.kind)?;
    let format = match internal_format {
        GL_RGBA16F => SourceTexelFormat::Rgba16Float,
        GL_RGBA32F => SourceTexelFormat::Rgba32Float,
        _ => {
            return Err(unsupported(
                info,
                format!("GL internal format 0x{internal_format:08x}"),
            ));
        }
    };
    let key_value_len = read_u32(bytes, 60, info.kind)? as usize;
    let mut cursor = KTX1_HEADER_SIZE
        .checked_add(key_value_len)
        .ok_or_else(|| invalid(info.kind, "KTX1 key/value offset overflows usize"))?;
    require_range(info.kind, bytes, cursor, 0)?;

    let mut output = empty_source_texels(info);
    // KTX1 stores one mip at a time, then the six faces. Reorder into Zircon face-major mips.
    for mip in 0..info.mip_count {
        let face_byte_len = read_u32(bytes, cursor, info.kind)? as usize;
        cursor += 4;
        let mip_size = source_cubemap_mip_size(info.face_size, mip);
        let expected = mip_size as usize * mip_size as usize * format.bytes_per_texel();
        if face_byte_len != expected {
            return Err(invalid(
                info.kind,
                format!("KTX1 mip {mip} face size is {face_byte_len}, expected {expected}"),
            ));
        }
        for face in CubemapFace::ALL {
            let payload = require_range(info.kind, bytes, cursor, face_byte_len)?;
            write_face_mip(&mut output, info, face, mip, payload, format)?;
            cursor = align4(cursor + face_byte_len);
        }
        cursor = align4(cursor);
    }
    Ok(output)
}

fn decode_ktx2(
    bytes: &[u8],
    info: &ExternalSourceCubemapContainerInfo,
) -> Result<Vec<[f32; 4]>, ExternalSourceCubemapDecodeError> {
    require_range(info.kind, bytes, 0, KTX2_HEADER_SIZE)?;
    let vk_format = read_u32(bytes, 12, info.kind)?;
    let format = match vk_format {
        VK_FORMAT_R16G16B16A16_SFLOAT => SourceTexelFormat::Rgba16Float,
        VK_FORMAT_R32G32B32A32_SFLOAT => SourceTexelFormat::Rgba32Float,
        _ => return Err(unsupported(info, format!("Vulkan format {vk_format}"))),
    };
    let mut output = empty_source_texels(info);
    for mip in 0..info.mip_count {
        let index_offset = KTX2_HEADER_SIZE + mip as usize * KTX2_LEVEL_INDEX_ENTRY_SIZE;
        let level_offset = read_u64(bytes, index_offset, info.kind)? as usize;
        let level_len = read_u64(bytes, index_offset + 8, info.kind)? as usize;
        let mip_size = source_cubemap_mip_size(info.face_size, mip);
        let face_byte_len = mip_size as usize * mip_size as usize * format.bytes_per_texel();
        let expected_level_len = face_byte_len * CubemapFace::ALL.len();
        if level_len < expected_level_len {
            return Err(invalid(
                info.kind,
                format!(
                    "KTX2 mip {mip} level size is {level_len}, expected at least {expected_level_len}"
                ),
            ));
        }
        for (face_index, face) in CubemapFace::ALL.into_iter().enumerate() {
            let payload = require_range(
                info.kind,
                bytes,
                level_offset + face_index * face_byte_len,
                face_byte_len,
            )?;
            write_face_mip(&mut output, info, face, mip, payload, format)?;
        }
    }
    Ok(output)
}

fn empty_source_texels(info: &ExternalSourceCubemapContainerInfo) -> Vec<[f32; 4]> {
    vec![[0.0; 4]; source_cubemap_sample_count(info.face_size, info.mip_count)]
}

fn write_face_mip(
    output: &mut [[f32; 4]],
    info: &ExternalSourceCubemapContainerInfo,
    face: CubemapFace,
    mip: u32,
    bytes: &[u8],
    format: SourceTexelFormat,
) -> Result<(), ExternalSourceCubemapDecodeError> {
    let mip_size = source_cubemap_mip_size(info.face_size, mip);
    let texel_count = mip_size as usize * mip_size as usize;
    let decoded = decode_texels(bytes, format);
    if decoded.len() != texel_count {
        return Err(invalid(
            info.kind,
            format!(
                "decoded face {face:?} mip {mip} has {} texels, expected {texel_count}",
                decoded.len()
            ),
        ));
    }
    let offset = source_cubemap_face_mip_offset(info.face_size, info.mip_count, face, mip);
    output[offset..offset + texel_count].copy_from_slice(&decoded);
    Ok(())
}

fn decode_texels(bytes: &[u8], format: SourceTexelFormat) -> Vec<[f32; 4]> {
    match format {
        SourceTexelFormat::Rgba16Float => decode_rgba16f_texels(bytes)
            .into_iter()
            .map(sanitize_texel)
            .collect(),
        SourceTexelFormat::Rgba32Float => bytes
            .chunks_exact(16)
            .map(|chunk| {
                sanitize_texel([
                    read_f32(chunk, 0),
                    read_f32(chunk, 4),
                    read_f32(chunk, 8),
                    read_f32(chunk, 12),
                ])
            })
            .collect(),
    }
}

fn sanitize_texel(mut texel: [f32; 4]) -> [f32; 4] {
    for channel in &mut texel[..3] {
        *channel = if channel.is_finite() {
            channel.clamp(0.0, 65_504.0)
        } else {
            0.0
        };
    }
    texel[3] = 1.0;
    texel
}

fn require_range<'a>(
    kind: ExternalSourceCubemapContainerKind,
    bytes: &'a [u8],
    offset: usize,
    len: usize,
) -> Result<&'a [u8], ExternalSourceCubemapDecodeError> {
    let required = offset
        .checked_add(len)
        .ok_or_else(|| invalid(kind, "payload byte range overflows usize"))?;
    bytes
        .get(offset..required)
        .ok_or(ExternalSourceCubemapDecodeError::TruncatedPayload {
            kind,
            offset,
            required,
            actual: bytes.len(),
        })
}

fn read_u32(
    bytes: &[u8],
    offset: usize,
    kind: ExternalSourceCubemapContainerKind,
) -> Result<u32, ExternalSourceCubemapDecodeError> {
    let value = require_range(kind, bytes, offset, 4)?;
    Ok(u32::from_le_bytes(
        value.try_into().expect("four-byte range"),
    ))
}

fn read_u64(
    bytes: &[u8],
    offset: usize,
    kind: ExternalSourceCubemapContainerKind,
) -> Result<u64, ExternalSourceCubemapDecodeError> {
    let value = require_range(kind, bytes, offset, 8)?;
    Ok(u64::from_le_bytes(
        value.try_into().expect("eight-byte range"),
    ))
}

fn read_f32(bytes: &[u8], offset: usize) -> f32 {
    f32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("four-byte float"),
    )
}

fn align4(value: usize) -> usize {
    (value + 3) & !3
}

fn unsupported(
    info: &ExternalSourceCubemapContainerInfo,
    format: impl Into<String>,
) -> ExternalSourceCubemapDecodeError {
    ExternalSourceCubemapDecodeError::UnsupportedPixelFormat {
        kind: info.kind,
        format: format.into(),
    }
}

fn invalid(
    kind: ExternalSourceCubemapContainerKind,
    reason: impl Into<String>,
) -> ExternalSourceCubemapDecodeError {
    ExternalSourceCubemapDecodeError::InvalidPayload {
        kind,
        reason: reason.into(),
    }
}
