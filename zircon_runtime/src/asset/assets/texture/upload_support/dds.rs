use super::super::TextureAsset;
use super::bytes::read_u32_le;
use super::{
    TextureUploadCompressionFamily, TextureUploadPlan, TextureUploadSubresource,
    texture_descriptor_layer_count, texture_descriptor_mip_count,
};

const DDPF_FOURCC: u32 = 0x0000_0004;
const DDSCAPS_COMPLEX: u32 = 0x0000_0008;
const DDSCAPS_MIPMAP: u32 = 0x0040_0000;
const DDSCAPS_TEXTURE: u32 = 0x0000_1000;
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
const DDSCAPS2_VOLUME: u32 = 0x0020_0000;
const DDSD_CAPS: u32 = 0x0000_0001;
const DDSD_HEIGHT: u32 = 0x0000_0002;
const DDSD_WIDTH: u32 = 0x0000_0004;
const DDSD_PITCH: u32 = 0x0000_0008;
const DDSD_PIXELFORMAT: u32 = 0x0000_1000;
const DDSD_DEPTH: u32 = 0x0080_0000;
const DDSD_LINEARSIZE: u32 = 0x0008_0000;
const DDSD_MIPMAPCOUNT: u32 = 0x0002_0000;
const DDSD_REQUIRED_FLAGS: u32 = DDSD_CAPS | DDSD_HEIGHT | DDSD_WIDTH | DDSD_PIXELFORMAT;
const DDS_ALPHA_MODE_CUSTOM: u32 = 0x4;
const DDS_ALPHA_MODE_MASK: u32 = 0x7;
const DDS_DIMENSION_TEXTURE2D: u32 = 3;
const DDS_RESOURCE_MISC_TEXTURECUBE: u32 = 0x4;
pub(super) fn dds_upload_plan(
    texture: &TextureAsset,
    format: &str,
    bytes: &[u8],
) -> Option<TextureUploadPlan> {
    if bytes.get(..4)? != b"DDS " {
        return None;
    }
    if !dds_main_header_is_upload_ready(bytes)? {
        return None;
    }
    if read_u32_le(bytes, 12)? != texture.height.max(1)
        || read_u32_le(bytes, 16)? != texture.width.max(1)
    {
        return None;
    }
    if dds_header_mip_count(bytes)? != texture_descriptor_mip_count(texture) {
        return None;
    }
    if dds_header_layer_count(bytes)? != texture_descriptor_layer_count(texture) {
        return None;
    }
    let value = format.trim().to_ascii_lowercase();
    let (normalized, data_offset, bytes_per_block) = match value.as_str() {
        "dds/dxt1" => dds_classic_fourcc_upload_layout(&value, bytes, 8)?,
        "dds/dxt3" | "dds/dxt5" => dds_classic_fourcc_upload_layout(&value, bytes, 16)?,
        "dds/ati1" | "dds/bc4u" | "dds/bc4s" => dds_classic_fourcc_upload_layout(&value, bytes, 8)?,
        "dds/ati2" | "dds/bc5u" | "dds/bc5s" => {
            dds_classic_fourcc_upload_layout(&value, bytes, 16)?
        }
        value if value.starts_with("dds/dxgi-") => {
            if dds_fourcc(bytes)? != "DX10" {
                return None;
            }
            if !dds_dx10_extension_header_is_upload_ready(bytes)? {
                return None;
            }
            let dxgi = value.trim_start_matches("dds/dxgi-").parse::<u32>().ok()?;
            if read_u32_le(bytes, 128)? != dxgi {
                return None;
            }
            let bytes_per_block = match dxgi {
                71 | 72 | 80 | 81 => 8,
                74 | 75 | 77 | 78 | 83 | 84 | 95 | 96 | 98 | 99 => 16,
                _ => return None,
            };
            (value.to_string(), 148, bytes_per_block)
        }
        _ => return None,
    };
    let subresources = dds_upload_subresources(texture, data_offset, 4, 4, bytes_per_block)?;
    Some(TextureUploadPlan {
        format: normalized,
        compression: TextureUploadCompressionFamily::Bc,
        data_offset,
        data_length: None,
        block_width: 4,
        block_height: 4,
        block_depth: 1,
        bytes_per_block,
        subresources,
    })
}

fn dds_upload_subresources(
    texture: &TextureAsset,
    data_offset: usize,
    block_width: u32,
    block_height: u32,
    bytes_per_block: u32,
) -> Option<Vec<TextureUploadSubresource>> {
    let mip_count = texture_descriptor_mip_count(texture);
    let layer_count = texture_descriptor_layer_count(texture);
    let mut level_layouts = Vec::with_capacity(usize::try_from(mip_count).ok()?);
    let mut bytes_per_layer = 0_usize;
    for mip_level in 0..mip_count {
        let width = mip_extent(texture.width.max(1), mip_level);
        let height = mip_extent(texture.height.max(1), mip_level);
        let block_columns = div_ceil(width, block_width);
        let block_rows = div_ceil(height, block_height);
        let bytes_per_row = block_columns.checked_mul(bytes_per_block)?;
        let data_length =
            usize::try_from(u64::from(bytes_per_row).checked_mul(u64::from(block_rows))?).ok()?;
        bytes_per_layer = bytes_per_layer.checked_add(data_length)?;
        level_layouts.push((mip_level, bytes_per_row, block_rows, data_length));
    }

    let capacity = usize::try_from(mip_count.checked_mul(layer_count)?).ok()?;
    let mut subresources = Vec::with_capacity(capacity);
    for array_layer in 0..layer_count {
        let mut offset = data_offset.checked_add(
            usize::try_from(array_layer)
                .ok()?
                .checked_mul(bytes_per_layer)?,
        )?;
        for (mip_level, bytes_per_row, block_rows, data_length) in &level_layouts {
            subresources.push(TextureUploadSubresource {
                mip_level: *mip_level,
                array_layer,
                data_offset: offset,
                data_length: *data_length,
                bytes_per_row: *bytes_per_row,
                block_rows: *block_rows,
            });
            offset = offset.checked_add(*data_length)?;
        }
    }
    Some(subresources)
}

fn div_ceil(value: u32, divisor: u32) -> u32 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor.max(1)
}

const fn mip_extent(value: u32, level: u32) -> u32 {
    let shifted = if level >= u32::BITS {
        0
    } else {
        value >> level
    };
    if shifted == 0 { 1 } else { shifted }
}

fn dds_classic_fourcc_upload_layout(
    normalized: &str,
    bytes: &[u8],
    bytes_per_block: u32,
) -> Option<(String, usize, u32)> {
    let expected = normalized.trim_start_matches("dds/");
    if dds_fourcc(bytes)?.to_ascii_lowercase() != expected {
        return None;
    }
    Some((normalized.to_string(), 128, bytes_per_block))
}

fn dds_main_header_is_upload_ready(bytes: &[u8]) -> Option<bool> {
    if read_u32_le(bytes, 4)? != 124 {
        return Some(false);
    }

    let flags = read_u32_le(bytes, 8)?;
    if flags & DDSD_REQUIRED_FLAGS != DDSD_REQUIRED_FLAGS
        || flags & DDSD_LINEARSIZE == 0
        || flags & DDSD_PITCH != 0
    {
        return Some(false);
    }

    if read_u32_le(bytes, 20)? == 0
        || read_u32_le(bytes, 24)? != 0
        || read_u32_le(bytes, 76)? != 32
        || read_u32_le(bytes, 80)? & DDPF_FOURCC == 0
        || read_u32_le(bytes, 108)? & DDSCAPS_TEXTURE == 0
    {
        return Some(false);
    }

    let caps = read_u32_le(bytes, 108)?;
    if dds_header_mip_count(bytes)? > 1
        && (caps & DDSCAPS_MIPMAP == 0 || caps & DDSCAPS_COMPLEX == 0)
    {
        return Some(false);
    }

    let caps2 = read_u32_le(bytes, 112)?;
    if caps2 & DDSCAPS2_VOLUME != 0 || flags & DDSD_DEPTH != 0 {
        return Some(false);
    }

    let face_flags = DDSCAPS2_CUBEMAP_ALL_FACES & !DDSCAPS2_CUBEMAP;
    if caps2 & DDSCAPS2_CUBEMAP == 0 && caps2 & face_flags != 0 {
        return Some(false);
    }
    if caps2 & DDSCAPS2_CUBEMAP != 0
        && (caps2 & DDSCAPS2_CUBEMAP_ALL_FACES != DDSCAPS2_CUBEMAP_ALL_FACES
            || caps & DDSCAPS_COMPLEX == 0)
    {
        return Some(false);
    }

    Some(true)
}

fn dds_fourcc(bytes: &[u8]) -> Option<&str> {
    std::str::from_utf8(bytes.get(84..88)?).ok()
}

fn dds_header_mip_count(bytes: &[u8]) -> Option<u32> {
    if read_u32_le(bytes, 8)? & DDSD_MIPMAPCOUNT == 0 {
        return Some(1);
    }
    let mip_count = read_u32_le(bytes, 28)?;
    if mip_count == 0 {
        return None;
    }
    Some(mip_count)
}

fn dds_header_layer_count(bytes: &[u8]) -> Option<u32> {
    let caps2 = read_u32_le(bytes, 112)?;
    let classic_faces = if caps2 & DDSCAPS2_CUBEMAP != 0 { 6 } else { 1 };
    if dds_fourcc(bytes)? == "DX10" {
        let misc_flag = read_u32_le(bytes, 136)?;
        let dx10_faces = if misc_flag & DDS_RESOURCE_MISC_TEXTURECUBE != 0 {
            6
        } else {
            1
        };
        return read_u32_le(bytes, 140)?.max(1).checked_mul(dx10_faces);
    }
    Some(classic_faces)
}

fn dds_dx10_extension_header_is_upload_ready(bytes: &[u8]) -> Option<bool> {
    if read_u32_le(bytes, 132)? != DDS_DIMENSION_TEXTURE2D {
        return Some(false);
    }

    let misc_flag = read_u32_le(bytes, 136)?;
    if misc_flag & !DDS_RESOURCE_MISC_TEXTURECUBE != 0 {
        return Some(false);
    }

    let caps2 = read_u32_le(bytes, 112)?;
    let classic_header_cubemap = caps2 & DDSCAPS2_CUBEMAP != 0;
    let dx10_texturecube = misc_flag & DDS_RESOURCE_MISC_TEXTURECUBE != 0;
    if classic_header_cubemap && dx10_texturecube {
        return Some(false);
    }
    if (classic_header_cubemap || dx10_texturecube)
        && read_u32_le(bytes, 108)? & DDSCAPS_COMPLEX == 0
    {
        return Some(false);
    }

    if read_u32_le(bytes, 140)? == 0 {
        return Some(false);
    }

    let misc_flags2 = read_u32_le(bytes, 144)?;
    if misc_flags2 & !DDS_ALPHA_MODE_MASK != 0 {
        return Some(false);
    }
    Some((misc_flags2 & DDS_ALPHA_MODE_MASK) <= DDS_ALPHA_MODE_CUSTOM)
}
