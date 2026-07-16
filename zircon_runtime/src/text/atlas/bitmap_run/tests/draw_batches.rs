use super::super::super::render_batch::glyph_atlas_draw_batch_plan;
use super::super::super::render_contract::GlyphAtlasBlendMode;
use super::super::super::render_plan::GlyphAtlasScreenRect;
use super::super::super::GlyphAtlasFormat;
use super::source;
use crate::core::math::UVec2;

#[test]
fn render_text_atlas_bitmap_run_feeds_draw_batches_without_rgba_semantic_merge() {
    let plan = super::glyph_atlas_bitmap_run_plan_with_padding(
        [
            source(GlyphAtlasFormat::SubpixelMask, UVec2::new(8, 8), 8.0, 256),
            source(GlyphAtlasFormat::Color, UVec2::new(8, 8), 22.0, 256),
        ],
        UVec2::new(32, 32),
        17,
        1,
        2,
    );
    let draw_plan = glyph_atlas_draw_batch_plan(
        plan.draw_glyphs,
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 32.0),
    );

    assert_eq!(draw_plan.visible_glyph_count, 2);
    assert_eq!(draw_plan.batches.len(), 2);
    assert!(draw_plan.requires_background_composite);
    assert_eq!(
        draw_plan.batches[0].key.render_contract.blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
    assert_eq!(
        draw_plan.batches[1].key.render_contract.blend_mode,
        GlyphAtlasBlendMode::SourceRgba
    );
}
