use super::super::render_contract::GlyphAtlasBlendMode;
use super::*;

#[test]
fn render_text_atlas_draw_plan_builds_subpixel_quad_with_background_composite() {
    let placement = GlyphRasterPlacement::from_raster_input(
        GlyphAtlasFormat::SubpixelMask,
        GlyphSmoothingMode::Subpixel,
        false,
        42.45,
    );
    let glyph = glyph(
        GlyphAtlasFormat::SubpixelMask,
        2,
        GlyphAtlasScreenRect::from_raster_placement(placement, 12.0, 20.0, 12.0),
    );

    let quad = glyph_atlas_draw_quad(glyph, GlyphAtlasScreenRect::new(0.0, 0.0, 200.0, 80.0))
        .expect("subpixel glyph should produce a draw quad");

    assert_near(quad.vertices[0].position_px[0], 42.0 + 1.0 / 3.0);
    assert_near(quad.vertices[1].position_px[0], 62.0 + 1.0 / 3.0);
    assert_eq!(quad.page_key.page_index, 2);
    assert_eq!(
        quad.render_contract.blend_mode,
        GlyphAtlasBlendMode::SubpixelBackgroundComposite
    );
    assert!(quad.render_contract.requires_background_composite());
    assert!(quad
        .vertices
        .iter()
        .all(|vertex| vertex.background_color == [0.1, 0.12, 0.14, 1.0]));
    assert!(quad.vertices.iter().all(|vertex| vertex.page_index == 2));
}

#[test]
fn render_text_atlas_draw_plan_clips_position_and_uv_without_padding_bleed() {
    let glyph = GlyphAtlasDrawGlyph {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
        atlas_size: UVec2::new(100, 50),
        atlas_rect: GlyphAtlasRect {
            x: 10,
            y: 5,
            width: 40,
            height: 20,
        },
        content_size: UVec2::new(20, 10),
        screen_rect: GlyphAtlasScreenRect::new(10.0, 10.0, 20.0, 10.0),
        foreground_color: [1.0, 1.0, 1.0, 1.0],
        background_color: [0.0, 0.0, 0.0, 1.0],
    };

    let quad = glyph_atlas_draw_quad(glyph, GlyphAtlasScreenRect::new(15.0, 12.0, 10.0, 5.0))
        .expect("clipped glyph should remain visible");

    assert_eq!(quad.vertices[0].position_px, [15.0, 12.0]);
    assert_eq!(quad.vertices[2].position_px, [25.0, 17.0]);
    assert_near(quad.vertices[0].uv[0], 0.15);
    assert_near(quad.vertices[0].uv[1], 0.14);
    assert_near(quad.vertices[2].uv[0], 0.25);
    assert_near(quad.vertices[2].uv[1], 0.24);
}

#[test]
fn render_text_atlas_draw_plan_keeps_color_rgba_distinct_from_subpixel() {
    let color_quad = glyph_atlas_draw_quad(
        glyph(
            GlyphAtlasFormat::Color,
            0,
            GlyphAtlasScreenRect::new(4.0, 5.0, 18.0, 18.0),
        ),
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 64.0),
    )
    .expect("color glyph should produce a draw quad");

    assert_eq!(
        color_quad.render_contract.blend_mode,
        GlyphAtlasBlendMode::SourceRgba
    );
    assert!(!color_quad.render_contract.requires_background_composite());
}

#[test]
fn render_text_atlas_draw_plan_normalizes_subpixel_background_composite_input() {
    let glyph = GlyphAtlasDrawGlyph {
        page_key: GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0),
        atlas_size: UVec2::new(128, 64),
        atlas_rect: GlyphAtlasRect {
            x: 16,
            y: 8,
            width: 24,
            height: 18,
        },
        content_size: UVec2::new(20, 12),
        screen_rect: GlyphAtlasScreenRect::new(6.0, 10.0, 20.0, 12.0),
        foreground_color: [0.9, 0.9, 0.85, 1.0],
        background_color: [f32::NAN, 1.25, -0.25, 0.2],
    };

    let quad = glyph_atlas_draw_quad(glyph, GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 64.0))
        .expect("subpixel glyph should produce a draw quad");

    assert!(quad.render_contract.requires_background_composite());
    assert!(quad
        .vertices
        .iter()
        .all(|vertex| vertex.background_color == [0.0, 1.0, 0.0, 1.0]));
}

#[test]
fn render_text_atlas_draw_plan_rejects_empty_or_offscreen_quads() {
    let empty = glyph_atlas_draw_quad(
        glyph(
            GlyphAtlasFormat::AlphaMask,
            0,
            GlyphAtlasScreenRect::new(5.0, 5.0, 0.0, 12.0),
        ),
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 64.0),
    );
    let offscreen = glyph_atlas_draw_quad(
        glyph(
            GlyphAtlasFormat::AlphaMask,
            0,
            GlyphAtlasScreenRect::new(80.0, 5.0, 12.0, 12.0),
        ),
        GlyphAtlasScreenRect::new(0.0, 0.0, 64.0, 64.0),
    );

    assert!(empty.is_none());
    assert!(offscreen.is_none());
}

fn glyph(
    format: GlyphAtlasFormat,
    page_index: u32,
    screen_rect: GlyphAtlasScreenRect,
) -> GlyphAtlasDrawGlyph {
    GlyphAtlasDrawGlyph {
        page_key: GlyphAtlasPageKey::new(format, page_index),
        atlas_size: UVec2::new(128, 64),
        atlas_rect: GlyphAtlasRect {
            x: 16,
            y: 8,
            width: 24,
            height: 18,
        },
        content_size: UVec2::new(20, 12),
        screen_rect,
        foreground_color: [0.9, 0.9, 0.85, 1.0],
        background_color: [0.1, 0.12, 0.14, 1.0],
    }
}

fn assert_near(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 0.001,
        "expected {actual} to be near {expected}"
    );
}
