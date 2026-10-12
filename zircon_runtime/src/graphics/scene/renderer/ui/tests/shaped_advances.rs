use super::super::shaped_advances::{
    resolved_horizontal_shaped_glyph_advances, visual_range_topology_requires_projection,
};
use super::text_batch;
use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiShapedGlyph;
use crate::text::ShapedGlyphRotation;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::UiTextRange;

fn shaped_glyph(
    source_scalar: char,
    source_range: UiTextRange,
    advance: f32,
) -> ScreenSpaceUiShapedGlyph {
    ScreenSpaceUiShapedGlyph {
        glyph_id: 0,
        font_id: None,
        font_instance_id: None,
        source_scalar,
        source_range,
        advance,
        offset_x: 0.0,
        offset_y: 0.0,
        rotation: ShapedGlyphRotation::None,
        requires_atlas_slot: true,
    }
}

#[test]
fn sdf_shaped_draw_plan_prefers_tatweel_layout_advances_for_parity() {
    let mut text = text_batch("A\u{0640}B", UiFrame::new(4.0, 8.0, 41.0, 20.0));
    text.source_range = Some(UiTextRange { start: 0, end: 2 });
    text.glyph_advances = vec![8.0, 24.0, 9.0];
    text.shaped_glyphs = vec![
        shaped_glyph('A', UiTextRange { start: 0, end: 1 }, 7.0),
        shaped_glyph('\u{0640}', UiTextRange { start: 1, end: 3 }, 10.0),
        shaped_glyph('B', UiTextRange { start: 3, end: 4 }, 8.0),
    ];

    let advances = resolved_horizontal_shaped_glyph_advances(&text);

    assert_eq!(advances, vec![8.0, 24.0, 9.0]);
    assert!((advances.iter().sum::<f32>() - text.frame.width).abs() < 0.1);
}

#[test]
fn sdf_shaped_draw_plan_keeps_visual_bidi_advances_on_the_direct_path() {
    // Layout materializes the visual line "A בא" and reorders the advances with it.
    // There is no ligature or cluster-count topology to project in this case.
    let mut text = text_batch("A בא", UiFrame::new(4.0, 8.0, 10.0, 20.0));
    text.source_range = Some(UiTextRange { start: 0, end: 6 });
    text.glyph_advances = vec![1.0, 2.0, 4.0, 3.0];
    text.shaped_glyphs = vec![
        shaped_glyph('A', UiTextRange { start: 0, end: 1 }, 7.0),
        shaped_glyph(' ', UiTextRange { start: 1, end: 2 }, 4.0),
        shaped_glyph('ב', UiTextRange { start: 2, end: 4 }, 9.0),
        shaped_glyph('א', UiTextRange { start: 4, end: 6 }, 8.0),
    ];

    assert!(!visual_range_topology_requires_projection(&text));
    let advances = resolved_horizontal_shaped_glyph_advances(&text);

    assert_eq!(advances, vec![1.0, 2.0, 4.0, 3.0]);
    assert!((advances.iter().sum::<f32>() - text.frame.width).abs() < 0.1);
}

#[test]
fn sdf_shaped_draw_plan_projects_ligature_layout_advances_for_parity() {
    let mut text = text_batch("لا", UiFrame::new(4.0, 8.0, 30.0, 20.0));
    text.source_range = Some(UiTextRange { start: 0, end: 4 });
    text.glyph_advances = vec![11.0, 19.0];
    text.shaped_glyphs = vec![shaped_glyph('ل', UiTextRange { start: 0, end: 4 }, 13.0)];

    let advances = resolved_horizontal_shaped_glyph_advances(&text);

    assert_eq!(advances, vec![30.0]);
    assert!((advances.iter().sum::<f32>() - text.frame.width).abs() < 0.1);
}

#[test]
fn sdf_shaped_draw_plan_distributes_cluster_advance_across_nonzero_glyphs() {
    let mut text = text_batch("a", UiFrame::new(4.0, 8.0, 30.0, 20.0));
    text.source_range = Some(UiTextRange { start: 0, end: 1 });
    text.glyph_advances = vec![30.0];
    text.shaped_glyphs = vec![
        shaped_glyph('a', UiTextRange { start: 0, end: 1 }, 8.0),
        shaped_glyph('a', UiTextRange { start: 0, end: 1 }, 4.0),
    ];

    let advances = resolved_horizontal_shaped_glyph_advances(&text);

    assert_eq!(advances, vec![20.0, 10.0]);
    assert!((advances.iter().sum::<f32>() - text.frame.width).abs() < 0.1);
}

#[test]
fn sdf_shaped_draw_plan_prefers_source_ranges_when_mark_and_ligature_counts_cancel() {
    let mut text = text_batch("e\u{301}fi", UiFrame::new(4.0, 8.0, 32.0, 20.0));
    text.source_range = Some(UiTextRange { start: 0, end: 5 });
    text.glyph_advances = vec![12.0, 20.0];
    text.shaped_glyphs = vec![
        shaped_glyph('e', UiTextRange { start: 0, end: 1 }, 6.0),
        shaped_glyph('\u{301}', UiTextRange { start: 1, end: 3 }, 0.0),
        shaped_glyph('f', UiTextRange { start: 3, end: 5 }, 20.0),
    ];

    let advances = resolved_horizontal_shaped_glyph_advances(&text);

    assert_eq!(advances, vec![12.0, 0.0, 20.0]);
    assert!((advances.iter().sum::<f32>() - text.frame.width).abs() < 0.1);
}

#[test]
fn sdf_shaped_draw_plan_keeps_rebased_virtual_tatweel_and_ligature_advances() {
    let mut text = text_batch("سـلا", UiFrame::new(4.0, 8.0, 37.0, 20.0));
    // The source line starts at byte 42 and contains "سلا" (six UTF-8 bytes). Visual fallback
    // re-shapes the materialized tatweel, so glyph ranges are visual offsets rebased by 42.
    text.source_range = Some(UiTextRange { start: 42, end: 48 });
    text.glyph_advances = vec![7.0, 10.0, 8.0, 12.0];
    text.shaped_glyphs = vec![
        shaped_glyph('س', UiTextRange { start: 42, end: 44 }, 7.0),
        shaped_glyph('ـ', UiTextRange { start: 44, end: 46 }, 9.0),
        shaped_glyph('ل', UiTextRange { start: 46, end: 50 }, 15.0),
    ];

    let advances = resolved_horizontal_shaped_glyph_advances(&text);

    assert_eq!(advances, vec![7.0, 10.0, 20.0]);
    assert!((advances.iter().sum::<f32>() - text.frame.width).abs() < 0.1);
}
