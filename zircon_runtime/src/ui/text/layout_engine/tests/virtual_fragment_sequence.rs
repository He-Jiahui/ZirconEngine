use std::sync::Arc;

use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

use super::super::candidate_line::{append_segment, insert_virtual_text, CandidateLine};
use super::super::visual_order;
use super::{
    apply_non_virtual_visual_order, capture, capture_with_external_source_ranges,
    has_virtual_fragment, reject_virtual_sequence_to_renderer_fallback,
    shape_and_apply_visual_order_with_sequences,
};
use crate::core::framework::text::TextDirection;
use crate::text::shaping::{BidiLineOrder, TextShapeRunProvider, TextShapingOutcome};
use crate::text::{
    ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript,
    ShapedHardLine, TextOrientation, TextRange, TextStyle, VerticalMode,
};

#[test]
fn capture_retains_logical_tatweel_anchor_before_visual_order() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "سلام",
        UiTextRange { start: 0, end: 8 },
    );
    assert!(insert_virtual_text(&mut line, 2, "ـ"));
    assert!(has_virtual_fragment(&line));

    let sequence = capture(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft,
    )
    .expect("source-congruent logical virtual line captures a sidecar");
    assert_eq!(sequence.text(), "سـلام");
    assert_eq!(sequence.base_direction(), TextDirection::RightToLeft);
    assert_eq!(
        sequence
            .logical_cluster_receipts()
            .filter_map(|(_, _, owner, _, _)| owner)
            .collect::<Vec<_>>(),
        vec![TextRange { start: 0, end: 8 }]
    );
    assert!(sequence
        .logical_cluster_receipts()
        .all(|(_, _, _, replaced, external)| replaced.is_none() && !external));
}

#[test]
fn capture_rejects_out_of_order_virtual_receipts() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "سلام",
        UiTextRange { start: 0, end: 8 },
    );
    assert!(insert_virtual_text(&mut line, 2, "ـ"));
    assert!(insert_virtual_text(&mut line, 6, "ـ"));
    line.virtual_source_receipts.swap(0, 1);

    assert!(capture(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft,
    )
    .is_none());
}

#[test]
fn capture_refuses_non_isomorphic_source_run_without_disabling_visual_fallback() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "ab",
        UiTextRange { start: 0, end: 2 },
    );
    assert!(insert_virtual_text(&mut line, 1, "…"));
    line.runs[0].source_range.end = 2;

    assert!(has_virtual_fragment(&line));
    assert!(capture(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight
    )
    .is_none());
}

#[test]
fn capture_marks_only_compiled_inline_ranges_as_external_clusters() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "a\u{fffc}",
        UiTextRange { start: 0, end: 4 },
    );
    assert!(insert_virtual_text(&mut line, 4, "\u{2026}"));

    let external = capture_with_external_source_ranges(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
        &[UiTextRange { start: 1, end: 4 }],
    )
    .expect("compiled inline range captures an external cluster");
    let literal = capture(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
    )
    .expect("a literal object replacement character remains text");

    assert_eq!(
        external
            .logical_cluster_receipts()
            .map(|(_, _, _, _, external)| external)
            .collect::<Vec<_>>(),
        vec![false, true, false]
    );
    assert!(literal
        .logical_cluster_receipts()
        .all(|(_, _, _, _, external)| !external));
}

#[test]
fn capture_accepts_external_cluster_without_virtual_text() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "a\u{fffc}b",
        UiTextRange { start: 0, end: 5 },
    );

    let sequence = capture_with_external_source_ranges(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
        &[UiTextRange { start: 1, end: 4 }],
    )
    .expect("an external layout block alone requires a logical sidecar");

    assert!(!has_virtual_fragment(&line));
    assert_eq!(
        sequence
            .logical_cluster_receipts()
            .map(|(_, _, _, _, external)| external)
            .collect::<Vec<_>>(),
        vec![false, true, false]
    );
}

#[test]
fn ordinary_line_consumes_the_canonical_order_without_reanalyzing_paragraph_text() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "abc אבג",
        UiTextRange { start: 0, end: 10 },
    );
    let order = BidiLineOrder {
        resolved_base_direction: TextDirection::LeftToRight,
        logical_levels: vec![0, 0, 0, 0, 1, 1, 1],
        visual_indices: vec![0, 1, 2, 3, 6, 5, 4],
        unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
    };
    let mut advances = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];

    apply_non_virtual_visual_order(
        &mut line,
        "x",
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
        Some(&order),
        Some(&mut advances),
    )
    .expect("canonical receipt makes paragraph re-analysis unnecessary");

    assert_eq!(line.text, "abc גבא");
    assert_eq!(advances, vec![1.0, 2.0, 3.0, 4.0, 7.0, 6.0, 5.0]);
}

#[test]
fn rejected_virtual_sequence_keeps_the_resolved_layout_fallback_eligible() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "abc",
        UiTextRange { start: 0, end: 3 },
    );
    assert!(insert_virtual_text(&mut line, 1, "…"));
    let mut sequences = Some(vec![capture(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
    )]);
    let mut advances = Some(vec![Some(vec![1.0, 2.0, 3.0, 4.0])]);

    let outcome = reject_virtual_sequence_to_renderer_fallback(&mut sequences, &mut advances, 0);

    assert!(matches!(outcome, TextShapingOutcome::Ready(())));
    assert!(!sequences.expect("virtual sequence collection")[0]
        .as_ref()
        .expect("virtual sequence remains as a renderer-fallback marker")
        .artifact_projection_allowed());
    assert!(advances.expect("virtual advance collection")[0].is_none());
    assert_eq!(line.text, "a…bc");
}

#[test]
fn virtual_bidi_advance_failure_rejects_only_the_private_artifact_route() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "abc",
        UiTextRange { start: 0, end: 3 },
    );
    assert!(insert_virtual_text(&mut line, 1, "…"));
    let original_text = line.text.clone();
    let mut sequences = Some(vec![capture(
        &line,
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
    )]);
    let mut advances = Some(vec![Some(vec![1.0])]);

    let bidi_outcome = visual_order::apply_visual_order_with_virtual_sequence(
        &mut line,
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
        sequences
            .as_mut()
            .and_then(|sequences| sequences.get_mut(0))
            .and_then(Option::as_mut),
        advances
            .as_mut()
            .and_then(|advances| advances.get_mut(0))
            .and_then(Option::as_mut),
    );
    assert!(bidi_outcome.is_err());

    let outcome = reject_virtual_sequence_to_renderer_fallback(&mut sequences, &mut advances, 0);

    assert!(matches!(outcome, TextShapingOutcome::Ready(())));
    assert!(!sequences.expect("virtual sequence collection")[0]
        .as_ref()
        .expect("virtual sequence remains as a renderer-fallback marker")
        .artifact_projection_allowed());
    assert!(advances.expect("virtual advance collection")[0].is_none());
    assert_eq!(line.text, original_text);
}

#[test]
fn virtual_sequence_shapes_one_fragment_before_visual_order() {
    let mut line = CandidateLine::empty();
    append_segment(
        &mut line,
        UiTextRunKind::Plain,
        "ab",
        UiTextRange { start: 0, end: 2 },
    );
    assert!(insert_virtual_text(&mut line, 1, "\u{2026}"));
    let shaped = Arc::new(ShapedGlyphRun {
        source_text: Arc::from("a\u{2026}b"),
        source_range: TextRange { start: 0, end: 5 },
        unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
        primary_face_id: None,
        direction: TextDirection::LeftToRight,
        orientation: TextOrientation::Horizontal,
        vertical_mode: VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 21.0,
        measured_height: 19.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![ShapedHardLine {
            line_index: 0,
            source_range: TextRange { start: 0, end: 5 },
            visual_range: TextRange { start: 0, end: 5 },
            measured_width: 21.0,
            baseline: 14.0,
            line_height: 19.0,
            glyphs: vec![
                shaped_glyph(0, 0, 1, 4.0),
                shaped_glyph(1, 1, 4, 13.0),
                shaped_glyph(2, 4, 5, 4.0),
            ],
        }],
    });
    let mut provider = CountingShapeRunProvider {
        shaped,
        shape_calls: 0,
    };
    let mut lines = vec![line];
    let mut advances = None;
    let mut final_metrics = vec![crate::text::layout::TextLineMetrics {
        width: 0.0,
        baseline: 8.0,
        line_height: 10.0,
    }];

    let sequences = shape_and_apply_visual_order_with_sequences(
        &mut lines,
        "ab",
        zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight,
        &TextStyle::default(),
        &mut provider,
        &mut advances,
        &mut final_metrics,
        None,
    )
    .into_result()
    .expect("shape and retain the virtual logical fragment")
    .expect("one virtual line retains a sidecar");

    let sequence = sequences[0]
        .as_ref()
        .expect("source-congruent virtual line has a canonical fragment");
    let fragment = sequence
        .fragment_for_revision(provider.font_collection_revision())
        .expect("fragment stays current through visual ordering");
    assert_eq!(fragment.metrics().line_height, 19.0);
    assert_eq!(fragment.grapheme_advances(), &[4.0, 13.0, 4.0]);
    assert_eq!(advances, Some(vec![Some(vec![4.0, 13.0, 4.0])]));
    assert_eq!(final_metrics[0].baseline, 14.0);
    assert_eq!(final_metrics[0].line_height, 19.0);
    assert_eq!(provider.shape_calls, 1);
}

fn shaped_glyph(glyph_id: u32, start: usize, end: usize, advance: f32) -> ShapedGlyph {
    ShapedGlyph {
        glyph_id,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange { start, end },
        visual_range: TextRange { start, end },
        advance,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::LeftToRight,
        bidi_level: 0,
        cluster_flags: ShapedGlyphClusterFlags::default(),
        rotation: ShapedGlyphRotation::None,
        script: ShapedGlyphScript::default(),
    }
}

struct CountingShapeRunProvider {
    shaped: Arc<ShapedGlyphRun>,
    shape_calls: usize,
}

impl TextShapeRunProvider for CountingShapeRunProvider {
    fn shape_horizontal_range_with_kerning(
        &mut self,
        _text: &str,
        _style: &TextStyle,
        _direction: TextDirection,
        _source_range: TextRange,
        _include_kerning: bool,
    ) -> TextShapingOutcome {
        self.shape_calls = self.shape_calls.saturating_add(1);
        TextShapingOutcome::Ready(Arc::clone(&self.shaped))
    }
}
