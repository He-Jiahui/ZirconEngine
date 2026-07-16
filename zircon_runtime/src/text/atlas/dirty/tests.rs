use super::*;
use crate::text::atlas::GlyphAtlasFormat;

#[test]
fn render_text_atlas_partial_upload_merges_dirty_rects() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(page_key, atlas_rect(32, 0, 32, 32));
    dirty_page.mark_dirty(page_key, atlas_rect(96, 64, 32, 32));

    assert_eq!(dirty_page.merged_rect(), Some(atlas_rect(32, 0, 96, 96)));
}

#[test]
fn render_text_atlas_partial_upload_ignores_other_pages_and_empty_rects() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
    let other_page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(other_page_key, atlas_rect(0, 0, 32, 32));
    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 0, 32));

    assert_eq!(dirty_page.merged_rect(), None);
}

fn atlas_rect(x: u32, y: u32, width: u32, height: u32) -> GlyphAtlasRect {
    GlyphAtlasRect {
        x,
        y,
        width,
        height,
    }
}
