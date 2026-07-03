use super::super::render_contract::GlyphAtlasBlendMode;
use super::super::{GlyphAtlasFormat, GlyphAtlasRect, GlyphRasterPlacement, GlyphSmoothingMode};
use super::*;
use crate::core::math::UVec2;

#[test]
fn render_text_atlas_draw_batch_plan_groups_quads_by_page_and_contract() {
    let subpixel_placement = GlyphRasterPlacement::from_raster_input(
        GlyphAtlasFormat::SubpixelMask,
        GlyphSmoothingMode::Subpixel,
        false,
        24.45,
    );
    let plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(4.0, 8.0, 12.0, 10.0),
            ),
            glyph(
                GlyphAtlasFormat::SubpixelMask,
                0,
                GlyphAtlasScreenRect::from_raster_placement(subpixel_placement, 8.0, 12.0, 10.0),
            ),
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(40.0, 8.0, 12.0, 10.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 80.0, 32.0),
    );

    assert_eq!(plan.visible_glyph_count, 3);
    assert_eq!(plan.skipped_glyph_count, 0);
    assert_eq!(plan.vertex_count, 18);
    assert_eq!(plan.batches.len(), 2);
    assert!(plan.requires_background_composite);
    assert_eq!(
        plan.batches[0].key.page_key.format,
        GlyphAtlasFormat::AlphaMask
    );
    assert_eq!(plan.batches[0].quads.len(), 2);
    assert_eq!(
        plan.batches[1].key.page_key.format,
        GlyphAtlasFormat::SubpixelMask
    );
    assert_eq!(
        plan.batches[1].key.render_contract.blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
    assert_near(
        plan.batches[1].quads[0].vertices[0].position_px[0],
        24.0 + 1.0 / 3.0,
    );
    assert_eq!(plan.batches[0].quads[0].vertices[0].position_px, [4.0, 8.0]);
    assert_eq!(
        plan.batches[0].quads[1].vertices[0].position_px,
        [40.0, 8.0]
    );
}

#[test]
fn render_text_atlas_draw_batch_plan_records_clipped_glyphs_without_empty_batches() {
    let plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(2.0, 4.0, 10.0, 10.0),
            ),
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(70.0, 4.0, 10.0, 10.0),
            ),
            glyph(
                GlyphAtlasFormat::AlphaMask,
                0,
                GlyphAtlasScreenRect::new(8.0, 4.0, 0.0, 10.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 32.0, 32.0),
    );

    assert_eq!(plan.visible_glyph_count, 1);
    assert_eq!(plan.skipped_glyph_count, 2);
    assert_eq!(plan.vertex_count, 6);
    assert_eq!(plan.batches.len(), 1);
    assert_eq!(plan.batches[0].quads.len(), 1);
}

#[test]
fn render_text_atlas_draw_batch_plan_keeps_rgba_storage_semantics_separate() {
    let plan = glyph_atlas_draw_batch_plan(
        [
            glyph(
                GlyphAtlasFormat::SubpixelMask,
                4,
                GlyphAtlasScreenRect::new(8.0, 6.0, 14.0, 12.0),
            ),
            glyph(
                GlyphAtlasFormat::Color,
                4,
                GlyphAtlasScreenRect::new(26.0, 6.0, 14.0, 12.0),
            ),
        ],
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 32.0),
    );

    assert_eq!(plan.batches.len(), 2);
    assert_eq!(
        plan.batches[0].key.render_contract.blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
    assert_eq!(
        plan.batches[1].key.render_contract.blend_mode,
        GlyphAtlasBlendMode::SourceRgba
    );
    assert_ne!(
        plan.batches[0].key.render_contract,
        plan.batches[1].key.render_contract
    );
}

#[test]
fn render_text_atlas_draw_batch_plan_preserves_background_color_per_vertex() {
    let plan = glyph_atlas_draw_batch_plan(
        [glyph_with_background(
            GlyphAtlasFormat::SubpixelMask,
            1,
            GlyphAtlasScreenRect::new(10.0, 10.0, 16.0, 14.0),
            [0.21, 0.23, 0.25, 1.0],
        )],
        GlyphAtlasScreenRect::new(0.0, 0.0, 48.0, 48.0),
    );

    assert_eq!(plan.batches.len(), 1);
    assert!(plan.requires_background_composite);
    assert!(plan.batches[0].quads[0]
        .vertices
        .iter()
        .all(|vertex| vertex.background_color == [0.21, 0.23, 0.25, 1.0]));
}

fn glyph(
    format: GlyphAtlasFormat,
    page_index: u32,
    screen_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasDrawGlyph {
    glyph_with_background(format, page_index, screen_rect, [0.08, 0.1, 0.12, 1.0])
}

fn glyph_with_background(
    format: GlyphAtlasFormat,
    page_index: u32,
    screen_rect: GlyphAtlasScreenRect,
    background_color: [f32; 4],
) -> GlyphAtlasDrawGlyph {
    GlyphAtlasDrawGlyph {
        page_key: GlyphAtlasPageKey::new(format, page_index),
        atlas_size: UVec2::new(128, 64),
        atlas_rect: GlyphAtlasRect {
            x: 12,
            y: 8,
            width: 24,
            height: 18,
        },
        content_size: UVec2::new(18, 12),
        screen_rect,
        foreground_color: [0.92, 0.94, 0.9, 1.0],
        background_color,
    }
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected {actual} to be near {expected}"
    );
}
