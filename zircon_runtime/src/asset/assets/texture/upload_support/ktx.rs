use crate::core::framework::render::RenderImageDimension;

use super::super::TextureAsset;
use super::bytes::{read_u32_le, read_u64_le};
use super::layout::{ktx2_vk_compressed_layout, ktx_gl_compressed_layout};
use super::{texture_descriptor_mip_count, TextureUploadPlan};

pub(super) const KTX2_IDENTIFIER: &[u8] = b"\xABKTX 20\xBB\r\n\x1A\n";
pub(super) const KTX2_LEVEL_INDEX_OFFSET: usize = 80;
const KTX_LITTLE_ENDIAN: u32 = 0x0403_0201;
const KTX1_IDENTIFIER: &[u8] = b"\xABKTX 11\xBB\r\n\x1A\n";
const KTX_CUBEMAP_FACE_COUNT: u32 = 6;
const KTX_COMPRESSED_GL_TYPE_SIZE: u32 = 1;
const KTX_WORD_ALIGNMENT: usize = 4;
const KTX2_DFD_MIN_BYTE_LENGTH: usize = 16;
const KTX2_LEVEL_INDEX_ENTRY_SIZE: usize = 24;
pub(super) fn ktx_upload_plan(
    texture: &TextureAsset,
    format: &str,
    bytes: &[u8],
) -> Option<TextureUploadPlan> {
    let normalized = format.trim().to_ascii_lowercase();
    if bytes.get(..KTX1_IDENTIFIER.len())? != KTX1_IDENTIFIER {
        return None;
    }
    if read_u32_le(bytes, 12)? != KTX_LITTLE_ENDIAN {
        return None;
    }
    if !ktx1_header_is_upload_ready(bytes)? {
        return None;
    }
    let gl_internal_format = parse_ktx_gl_internal_format(&normalized)?;
    if read_u32_le(bytes, 28)? != gl_internal_format {
        return None;
    }
    if read_u32_le(bytes, 36)? != texture.width.max(1)
        || read_u32_le(bytes, 40)? != ktx_header_pixel_height(texture)
        || read_u32_le(bytes, 44)? != ktx_header_pixel_depth(texture)
    {
        return None;
    }
    if read_u32_le(bytes, 56)?.max(1) != texture_descriptor_mip_count(texture) {
        return None;
    }
    let array_elements = read_u32_le(bytes, 48)?.max(1);
    let faces = ktx_face_count(bytes, 52)?;
    if !ktx_layer_face_counts_match_descriptor(texture, array_elements, faces) {
        return None;
    }
    let layout = ktx_gl_compressed_layout(gl_internal_format)?;
    let key_value_data_len = usize::try_from(read_u32_le(bytes, 60)?).ok()?;
    let image_size_offset = 64_usize.checked_add(key_value_data_len)?;
    let data_length = usize::try_from(read_u32_le(bytes, image_size_offset)?).ok()?;
    let data_offset = image_size_offset.checked_add(4)?;

    Some(TextureUploadPlan {
        format: normalized,
        compression: layout.compression,
        data_offset,
        data_length: Some(data_length),
        block_width: layout.block_width,
        block_height: layout.block_height,
        block_depth: layout.block_depth,
        bytes_per_block: layout.bytes_per_block,
    })
}

pub(super) fn ktx2_upload_plan(
    texture: &TextureAsset,
    format: &str,
    bytes: &[u8],
) -> Option<TextureUploadPlan> {
    let normalized = format.trim().to_ascii_lowercase();
    if bytes.get(..KTX2_IDENTIFIER.len())? != KTX2_IDENTIFIER {
        return None;
    }
    let vk_format = parse_ktx2_vk_format(&normalized)?;
    if read_u32_le(bytes, 12)? != vk_format {
        return None;
    }
    if read_u32_le(bytes, 16)? == 0 {
        return None;
    }
    let supercompression = ktx2_supercompression(&normalized)?;
    if read_u32_le(bytes, 44)? != supercompression {
        return None;
    }
    if read_u32_le(bytes, 20)? != texture.width.max(1)
        || read_u32_le(bytes, 24)? != ktx_header_pixel_height(texture)
        || read_u32_le(bytes, 28)? != ktx_header_pixel_depth(texture)
    {
        return None;
    }
    let level_count_raw = read_u32_le(bytes, 40)?;
    if level_count_raw.max(1) != texture_descriptor_mip_count(texture) {
        return None;
    }
    let layer_count = read_u32_le(bytes, 32)?.max(1);
    let face_count = ktx_face_count(bytes, 36)?;
    if !ktx_layer_face_counts_match_descriptor(texture, layer_count, face_count) {
        return None;
    }
    let layout = ktx2_vk_compressed_layout(vk_format)?;
    let level_data_offset = usize::try_from(read_u64_le(bytes, KTX2_LEVEL_INDEX_OFFSET)?).ok()?;
    let level_data_len = usize::try_from(read_u64_le(bytes, KTX2_LEVEL_INDEX_OFFSET + 8)?).ok()?;
    let level_uncompressed_len =
        usize::try_from(read_u64_le(bytes, KTX2_LEVEL_INDEX_OFFSET + 16)?).ok()?;
    if level_data_len == 0 {
        return None;
    }
    let level_count = usize::try_from(level_count_raw.max(1)).ok()?;
    let level_index_end = KTX2_LEVEL_INDEX_OFFSET
        .checked_add(level_count.checked_mul(KTX2_LEVEL_INDEX_ENTRY_SIZE)?)?;
    if bytes.len() < level_index_end || level_data_offset < level_index_end {
        return None;
    }
    if !ktx2_dfd_header_is_upload_ready(bytes, level_index_end, level_data_offset, level_data_len)?
    {
        return None;
    }
    let declared_level_is_short = bytes.len().saturating_sub(level_data_offset) < level_data_len;
    if supercompression == 0 && level_uncompressed_len != level_data_len && !declared_level_is_short
    {
        return None;
    }

    Some(TextureUploadPlan {
        format: normalized,
        compression: layout.compression,
        data_offset: level_data_offset,
        data_length: Some(level_data_len),
        block_width: layout.block_width,
        block_height: layout.block_height,
        block_depth: layout.block_depth,
        bytes_per_block: layout.bytes_per_block,
    })
}

fn ktx1_header_is_upload_ready(bytes: &[u8]) -> Option<bool> {
    let gl_type = read_u32_le(bytes, 16)?;
    let gl_type_size = read_u32_le(bytes, 20)?;
    let gl_format = read_u32_le(bytes, 24)?;
    let gl_internal_format = read_u32_le(bytes, 28)?;
    let gl_base_internal_format = read_u32_le(bytes, 32)?;
    if gl_type != 0 || gl_format != 0 || gl_type_size != KTX_COMPRESSED_GL_TYPE_SIZE {
        return Some(false);
    }
    if gl_internal_format == 0 || gl_base_internal_format == 0 {
        return Some(false);
    }
    let key_value_data_len = usize::try_from(read_u32_le(bytes, 60)?).ok()?;
    Some(key_value_data_len % KTX_WORD_ALIGNMENT == 0)
}

fn ktx2_dfd_header_is_upload_ready(
    bytes: &[u8],
    level_index_end: usize,
    level_data_offset: usize,
    level_data_len: usize,
) -> Option<bool> {
    let dfd_byte_offset = usize::try_from(read_u32_le(bytes, 48)?).ok()?;
    let dfd_byte_length = usize::try_from(read_u32_le(bytes, 52)?).ok()?;
    if dfd_byte_offset == 0 || dfd_byte_length < KTX2_DFD_MIN_BYTE_LENGTH {
        return Some(false);
    }
    if dfd_byte_offset % KTX_WORD_ALIGNMENT != 0 || dfd_byte_length % KTX_WORD_ALIGNMENT != 0 {
        return Some(false);
    }
    let dfd_end = dfd_byte_offset.checked_add(dfd_byte_length)?;
    if dfd_byte_offset < level_index_end || dfd_end > bytes.len() {
        return Some(false);
    }
    let declared_total_size = usize::try_from(read_u32_le(bytes, dfd_byte_offset)?).ok()?;
    if declared_total_size != dfd_byte_length {
        return Some(false);
    }
    let level_data_end = level_data_offset.checked_add(level_data_len)?;
    Some(!ranges_overlap(
        dfd_byte_offset,
        dfd_end,
        level_data_offset,
        level_data_end,
    ))
}

fn ranges_overlap(
    first_start: usize,
    first_end: usize,
    second_start: usize,
    second_end: usize,
) -> bool {
    first_start < second_end && second_start < first_end
}

pub(super) fn ktx2_supercompression(format: &str) -> Option<u32> {
    format
        .split('/')
        .find_map(|part| part.strip_prefix("supercompression-"))
        .and_then(|value| value.parse::<u32>().ok())
}

fn parse_ktx_gl_internal_format(format: &str) -> Option<u32> {
    let value = format.strip_prefix("ktx/gl-internal-0x")?;
    u32::from_str_radix(value, 16).ok()
}

fn parse_ktx2_vk_format(format: &str) -> Option<u32> {
    format
        .split('/')
        .find_map(|part| part.strip_prefix("vk-"))
        .and_then(|value| value.parse::<u32>().ok())
}

fn ktx_header_pixel_height(texture: &TextureAsset) -> u32 {
    if texture.render_image_descriptor().dimension == RenderImageDimension::D1 {
        0
    } else {
        texture.height.max(1)
    }
}

fn ktx_header_pixel_depth(texture: &TextureAsset) -> u32 {
    let descriptor = texture.render_image_descriptor();
    if descriptor.dimension == RenderImageDimension::D3 {
        descriptor.depth_or_array_layers.max(1)
    } else {
        0
    }
}

fn ktx_layer_face_counts_match_descriptor(texture: &TextureAsset, layers: u32, faces: u32) -> bool {
    let descriptor = texture.render_image_descriptor();
    let expected_layers = if descriptor.dimension == RenderImageDimension::D3 {
        1
    } else {
        descriptor.array_layer_count.max(1)
    };
    layers.checked_mul(faces) == Some(expected_layers)
}

fn ktx_face_count(bytes: &[u8], offset: usize) -> Option<u32> {
    match read_u32_le(bytes, offset)? {
        1 => Some(1),
        KTX_CUBEMAP_FACE_COUNT => Some(KTX_CUBEMAP_FACE_COUNT),
        _ => None,
    }
}
