use crate::core::framework::render::RenderImageDimension;

use super::super::TextureAsset;
use super::bytes::read_u24_le;
use super::layout::is_supported_astc_block;
use super::{TextureUploadCompressionFamily, TextureUploadPlan};

const ASTC_IDENTIFIER: &[u8] = b"\x13\xAB\xA1\x5C";
pub(super) fn astc_upload_plan(
    texture: &TextureAsset,
    format: &str,
    bytes: &[u8],
) -> Option<TextureUploadPlan> {
    let value = format.trim().to_ascii_lowercase();
    let dimensions = value.strip_prefix("astc/")?;
    let mut parts = dimensions.split('x');
    let block_width = parts.next()?.parse::<u32>().ok()?;
    let block_height = parts.next()?.parse::<u32>().ok()?;
    let block_depth = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some()
        || block_width == 0
        || block_height == 0
        || block_depth == 0
        || !is_supported_astc_block(block_width, block_height, block_depth)
    {
        return None;
    }
    if bytes.get(..ASTC_IDENTIFIER.len())? != ASTC_IDENTIFIER {
        return None;
    }
    if u32::from(bytes.get(4).copied()?) != block_width
        || u32::from(bytes.get(5).copied()?) != block_height
        || u32::from(bytes.get(6).copied()?) != block_depth
    {
        return None;
    }
    let header_width = read_u24_le(bytes, 7)?;
    let header_height = read_u24_le(bytes, 10)?;
    let header_depth = read_u24_le(bytes, 13)?;
    if block_depth == 1 && header_depth != 1 {
        return None;
    }
    let descriptor = texture.render_image_descriptor();
    let expected_depth = if descriptor.dimension == RenderImageDimension::D3 {
        descriptor.depth_or_array_layers.max(1)
    } else {
        1
    };
    if header_width != texture.width
        || header_height != texture.height
        || header_depth != expected_depth
    {
        return None;
    }
    Some(TextureUploadPlan {
        format: value,
        compression: TextureUploadCompressionFamily::Astc,
        data_offset: 16,
        data_length: None,
        block_width,
        block_height,
        block_depth,
        bytes_per_block: 16,
    })
}
