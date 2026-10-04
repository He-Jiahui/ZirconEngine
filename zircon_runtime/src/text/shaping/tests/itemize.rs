use crate::core::framework::text::TextDirection;
use crate::text::shaping::bidi::BidiParagraph;
use crate::text::shaping::script_segment::ParagraphTextAnalysis;
use crate::text::{BackendShapeRequest, HardLine, TextRange, TextStyle};

use super::{
    logical_segments_for_line, restore_backend_cluster_logical_order, virtual_hard_break_glyph,
    ItemizationError,
};

#[derive(Clone, Copy)]
struct BackendGlyph {
    source_offset: usize,
    glyph_id: u32,
}

#[test]
fn rtl_backend_clusters_restore_logical_order_without_reversing_cluster_glyphs() {
    let mut glyphs = vec![
        backend_glyph(4, 40),
        backend_glyph(2, 20),
        backend_glyph(2, 21),
        backend_glyph(0, 10),
    ];

    restore_backend_cluster_logical_order(&mut glyphs, TextDirection::RightToLeft, |glyph| {
        glyph.source_offset
    })
    .expect("monotonic RTL clusters restore to logical order");

    assert_eq!(
        glyphs
            .iter()
            .map(|glyph| (glyph.source_offset, glyph.glyph_id))
            .collect::<Vec<_>>(),
        vec![(0, 10), (2, 20), (2, 21), (4, 40)]
    );
}

#[test]
fn itemization_reports_an_invalid_line_source_range() {
    let text = "A";
    let bidi = BidiParagraph::new(text, TextDirection::LeftToRight);
    let analysis = ParagraphTextAnalysis::new(text, None);

    let Err(error) = logical_segments_for_line(text, 0..2, &[], &analysis, &bidi, None) else {
        panic!("out-of-bounds line range must remain a typed itemization failure");
    };

    assert_eq!(
        error,
        ItemizationError::InvalidSourceRange {
            range: TextRange { start: 0, end: 2 }
        }
    );
}

#[test]
fn virtual_hard_break_reports_an_invalid_separator_range() {
    let text = "A";
    let style = TextStyle::default();
    let request = BackendShapeRequest::horizontal_with_kerning(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 1 },
        true,
    );
    let bidi = BidiParagraph::new(text, TextDirection::LeftToRight);
    let analysis = ParagraphTextAnalysis::new(text, None);
    let line = HardLine {
        content: 0..1,
        separator: 1..2,
    };

    let error = virtual_hard_break_glyph(request, &line, &bidi, &analysis)
        .expect_err("invalid separator range must not become an absent virtual glyph");

    assert_eq!(
        error,
        ItemizationError::InvalidSourceRange {
            range: TextRange { start: 1, end: 2 }
        }
    );
}

const fn backend_glyph(source_offset: usize, glyph_id: u32) -> BackendGlyph {
    BackendGlyph {
        source_offset,
        glyph_id,
    }
}
