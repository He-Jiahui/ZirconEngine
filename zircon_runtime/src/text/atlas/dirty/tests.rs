use super::*;
use crate::text::atlas::GlyphAtlasFormat;

#[test]
fn render_text_atlas_partial_upload_retains_distant_dirty_rects() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(page_key, atlas_rect(32, 0, 32, 32));
    dirty_page.mark_dirty(page_key, atlas_rect(96, 64, 32, 32));

    assert_eq!(
        dirty_page.regions(),
        &[atlas_rect(32, 0, 32, 32), atlas_rect(96, 64, 32, 32)]
    );
    assert_eq!(dirty_page.merged_rect(), Some(atlas_rect(32, 0, 96, 96)));
}

#[test]
fn render_text_atlas_partial_upload_does_not_merge_gapped_dirty_rects() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 4, 4));
    dirty_page.mark_dirty(page_key, atlas_rect(6, 0, 4, 4));

    assert_eq!(
        dirty_page.regions(),
        &[atlas_rect(0, 0, 4, 4), atlas_rect(6, 0, 4, 4)]
    );
}

#[test]
fn render_text_atlas_partial_upload_merges_gapped_rects_when_retained_slots_are_known_safe() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page =
        GlyphAtlasDirtyPage::new_with_retained_regions(page_key, vec![atlas_rect(0, 0, 4, 4)]);

    dirty_page.mark_dirty(page_key, atlas_rect(6, 0, 4, 4));
    dirty_page.mark_dirty(page_key, atlas_rect(12, 0, 4, 4));

    assert_eq!(dirty_page.regions(), &[atlas_rect(6, 0, 10, 4)]);
}

#[test]
fn render_text_atlas_partial_upload_does_not_merge_l_shaped_dirty_rects() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 10, 8));
    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 8, 10));

    assert_eq!(
        dirty_page.regions(),
        &[atlas_rect(0, 0, 10, 8), atlas_rect(0, 0, 8, 10)]
    );
}

#[test]
fn render_text_atlas_partial_upload_promotes_high_coverage_when_page_has_no_retained_slots() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new_with_retained_regions(page_key, Vec::new());

    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 24, 32));

    assert_eq!(
        dirty_page.regions_for_page(atlas_rect(0, 0, 32, 32)),
        vec![atlas_rect(0, 0, 32, 32)]
    );
}

#[test]
fn render_text_atlas_partial_upload_preserves_regions_above_count_limit() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 2, 2));
    dirty_page.mark_dirty(page_key, atlas_rect(8, 0, 2, 2));
    dirty_page.mark_dirty(page_key, atlas_rect(16, 0, 2, 2));

    assert_eq!(
        dirty_page.regions_for_page(atlas_rect(0, 0, 32, 32)),
        vec![
            atlas_rect(0, 0, 2, 2),
            atlas_rect(8, 0, 2, 2),
            atlas_rect(16, 0, 2, 2),
        ]
    );
}

#[test]
fn render_text_atlas_replayable_shadow_caps_regions_even_across_retained_slots() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page =
        GlyphAtlasDirtyPage::new_with_replayable_shadow(page_key, vec![atlas_rect(0, 0, 2, 2)]);

    for x in (0..90_000).step_by(10_000) {
        dirty_page.mark_dirty(page_key, atlas_rect(x, 0, 1, 1));
    }

    assert!(dirty_page.regions().len() <= 8);
    assert!(dirty_page
        .regions()
        .iter()
        .any(|region| region.x == 0 && region.width > 1));
}

#[test]
fn render_text_atlas_partial_upload_preserves_regions_at_high_coverage() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);

    dirty_page.mark_dirty(page_key, atlas_rect(0, 0, 24, 32));

    assert_eq!(
        dirty_page.regions_for_page(atlas_rect(0, 0, 32, 32)),
        vec![atlas_rect(0, 0, 24, 32)]
    );
}

#[test]
fn render_text_atlas_page_rebuild_explicitly_requests_a_full_page_upload() {
    let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0);
    let mut dirty_page = GlyphAtlasDirtyPage::new(page_key);
    let page_rect = atlas_rect(0, 0, 32, 32);

    dirty_page.mark_full_page_dirty(page_key, page_rect);
    dirty_page.mark_dirty(page_key, atlas_rect(8, 8, 4, 4));

    assert!(dirty_page.regions().is_empty());
    assert_eq!(dirty_page.regions_for_page(page_rect), vec![page_rect]);
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
