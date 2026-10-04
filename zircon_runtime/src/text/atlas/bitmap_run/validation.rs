//! 在消耗图集容量前验证位图来源的形状契约。
//! 此入口只接受覆盖率或颜色位图，SDF 由专用路径处理；像素必须是无行间填充的紧密排列。

use super::super::GlyphAtlasFormat;
use super::failure::GlyphAtlasBitmapAllocationFailureReason;
use super::types::GlyphAtlasBitmapSource;

pub(super) fn validate_bitmap_source(
    source: GlyphAtlasBitmapSource,
) -> Result<(), GlyphAtlasBitmapAllocationFailureReason> {
    if !is_bitmap_format(source.format) {
        return Err(GlyphAtlasBitmapAllocationFailureReason::UnsupportedFormat);
    }

    if source.content_size.x == 0 || source.content_size.y == 0 {
        return Err(GlyphAtlasBitmapAllocationFailureReason::EmptyContent);
    }

    let expected = expected_bitmap_source_len(source);
    if source.source_byte_len != expected {
        return Err(
            GlyphAtlasBitmapAllocationFailureReason::DataLengthMismatch {
                expected,
                actual: source.source_byte_len,
            },
        );
    }

    Ok(())
}

fn expected_bitmap_source_len(source: GlyphAtlasBitmapSource) -> usize {
    (source.content_size.x as usize)
        .saturating_mul(source.content_size.y as usize)
        .saturating_mul(source.format.storage_format().bytes_per_pixel() as usize)
}

fn is_bitmap_format(format: GlyphAtlasFormat) -> bool {
    matches!(
        format,
        GlyphAtlasFormat::AlphaMask | GlyphAtlasFormat::SubpixelMask | GlyphAtlasFormat::Color
    )
}
