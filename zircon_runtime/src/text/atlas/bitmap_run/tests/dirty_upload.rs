use super::super::super::{GlyphAtlasFormat, GlyphAtlasUploadMode};
use super::{atlas_rect, source};
use crate::core::math::UVec2;

#[test]
fn render_text_atlas_bitmap_run_promotes_full_page_dirty_to_full_upload() {
    let plan = super::glyph_atlas_bitmap_run_plan_with_padding(
        [source(
            GlyphAtlasFormat::AlphaMask,
            UVec2::new(16, 16),
            8.0,
            256,
        )],
        UVec2::new(16, 16),
        23,
        1,
        0,
    );

    assert!(plan.allocation_failures.is_empty());
    assert_eq!(plan.dirty_pages.len(), 1);
    assert_eq!(plan.upload_commands.len(), 1);
    assert_eq!(plan.upload_commands[0].mode, GlyphAtlasUploadMode::FullPage);
    assert_eq!(plan.upload_commands[0].rect, atlas_rect(0, 0, 16, 16));
    assert_eq!(plan.upload_commands[0].bytes_per_row, 16);
    assert_eq!(plan.upload_commands[0].rows_per_image, 16);
    assert_eq!(plan.upload_commands[0].upload_byte_len, 256);
}
