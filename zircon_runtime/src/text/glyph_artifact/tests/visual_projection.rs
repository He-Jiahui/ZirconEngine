use super::*;
use crate::core::framework::text::{TextGlyphFlags, TextGlyphRotation};
use zircon_runtime_interface::ui::surface::UiTextDirection;

#[test]
fn resolved_source_projection_keeps_multicluster_glyph_advance_finite() {
    let source_order = [visual_cluster(0, 0..1), visual_cluster(1, 1..2)];
    let mut glyphs = [ProjectedGlyph {
        glyph: glyph(0..2),
        source_index: 0,
        visual_index: 0,
        source_clusters: 0..2,
    }];

    apply_resolved_advances(&mut glyphs, &source_order, &[f32::MAX, f32::MAX], 2);

    assert_eq!(glyphs[0].glyph.advance, f32::MAX);
    assert!(glyphs[0].glyph.advance.is_finite());
}

fn visual_cluster(visual_index: usize, source_range: std::ops::Range<usize>) -> VisualCluster {
    VisualCluster {
        source_range: UiTextRange {
            start: source_range.start,
            end: source_range.end,
        },
        visual_range: UiTextRange {
            start: source_range.start,
            end: source_range.end,
        },
        visual_index,
        direction: Some(UiTextDirection::LeftToRight),
    }
}

fn glyph(source_range: std::ops::Range<usize>) -> TextGlyph {
    TextGlyph {
        glyph_id: 1,
        source_range: source_range.clone(),
        visual_range: source_range,
        advance: 0.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags::default(),
        requires_rasterization: false,
    }
}
