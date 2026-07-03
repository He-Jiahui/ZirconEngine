use super::*;
use crate::graphics::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};

#[test]
fn render_text_atlas_upload_full_page_uses_page_stride() {
    let page = page_spec(GlyphAtlasFormat::Sdf, 0, UVec2::new(256, 128));

    let command =
        glyph_atlas_upload_command(&page, GlyphAtlasUploadMode::FullPage, None, 256 * 128).unwrap();

    assert_eq!(command.mode, GlyphAtlasUploadMode::FullPage);
    assert_eq!(command.page_key, page.key);
    assert_eq!(command.rect, atlas_rect(0, 0, 256, 128));
    assert_eq!(command.source_offset, 0);
    assert_eq!(command.bytes_per_row, 256);
    assert_eq!(command.rows_per_image, 128);
    assert_eq!(command.upload_byte_len, 256 * 128);
}

#[test]
fn render_text_atlas_upload_partial_rgba_rect_uses_byte_stride() {
    let page = page_spec(GlyphAtlasFormat::Color, 0, UVec2::new(64, 32));

    let command = glyph_atlas_upload_command(
        &page,
        GlyphAtlasUploadMode::PartialRect,
        Some(atlas_rect(4, 3, 8, 5)),
        64 * 32 * 4,
    )
    .unwrap();

    assert_eq!(page.storage_format, GlyphAtlasStorageFormat::Rgba8Unorm);
    assert_eq!(command.mode, GlyphAtlasUploadMode::PartialRect);
    assert_eq!(
        command.sampling_semantics,
        GlyphAtlasSamplingSemantics::ColorRgba
    );
    assert_eq!(command.rect, atlas_rect(4, 3, 8, 5));
    assert_eq!(command.source_offset, (3 * 64 + 4) * 4);
    assert_eq!(command.bytes_per_row, 64 * 4);
    assert_eq!(command.rows_per_image, 32);
    assert_eq!(command.upload_byte_len, 8 * 5 * 4);
}

#[test]
fn render_text_atlas_upload_command_preserves_subpixel_sampling_semantics() {
    let page = page_spec(GlyphAtlasFormat::SubpixelMask, 0, UVec2::new(64, 32));

    let command = glyph_atlas_upload_command(
        &page,
        GlyphAtlasUploadMode::PartialRect,
        Some(atlas_rect(4, 3, 8, 5)),
        64 * 32 * 4,
    )
    .unwrap();

    assert_eq!(page.storage_format, GlyphAtlasStorageFormat::Rgba8Unorm);
    assert_eq!(command.bytes_per_row, 64 * 4);
    assert_eq!(
        command.sampling_semantics,
        GlyphAtlasSamplingSemantics::SubpixelCoverage
    );
}

#[test]
fn render_text_atlas_upload_skips_empty_and_out_of_range_sources() {
    let page = page_spec(GlyphAtlasFormat::Sdf, 0, UVec2::new(128, 128));

    assert_eq!(
        glyph_atlas_upload_command(
            &page,
            GlyphAtlasUploadMode::PartialRect,
            Some(atlas_rect(0, 0, 16, 16)),
            0,
        ),
        None
    );
    assert_eq!(
        glyph_atlas_upload_command(
            &page,
            GlyphAtlasUploadMode::PartialRect,
            Some(atlas_rect(120, 120, 16, 16)),
            128,
        ),
        None
    );
}

fn page_spec(format: GlyphAtlasFormat, page_index: u32, size: UVec2) -> GlyphAtlasPageSpec {
    GlyphAtlasPageSpec::new(GlyphAtlasPageKey::new(format, page_index), size)
}

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}
