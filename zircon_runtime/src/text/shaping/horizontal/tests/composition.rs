use std::sync::Arc;

use crate::core::framework::text::TextDirection;
use crate::text::{
    compiled_unicode_data_snapshot_id, ShapedGlyphClusterFlags, ShapedGlyphRotation,
    ShapedGlyphScript, TextOrientation, TextShapingFailureCode, TextShapingFailureDependency,
    TextShapingFailureDisposition, TextShapingFailurePhase, VerticalMode,
};

use super::*;

#[test]
fn composition_replaces_only_failed_hole_and_rebuilds_positions() {
    let source: Arc<str> = Arc::from("abc");
    let partial = partial(
        source.clone(),
        vec![glyph(10, 10, 11, 1.0), glyph(30, 12, 13, 3.0)],
    );
    let alternate = run(
        source,
        0,
        vec![
            glyph(100, 10, 11, 5.0),
            glyph(20, 11, 12, 2.0),
            glyph(300, 12, 13, 5.0),
        ],
    );

    let composed = compose_horizontal_partial(
        partial,
        alternate,
        &FontDatabase::default(),
        12.0,
        12.0,
        failure_receipt(),
    )
    .expect("the alternate glyph contained by the hole must be composable")
    .shaped;
    let line = &composed.lines[0];

    assert_eq!(
        line.glyphs
            .iter()
            .map(|glyph| glyph.glyph_id)
            .collect::<Vec<_>>(),
        vec![10, 20, 30]
    );
    assert_eq!(
        line.glyphs.iter().map(|glyph| glyph.x).collect::<Vec<_>>(),
        vec![0.0, 1.0, 3.0]
    );
    assert_eq!(line.measured_width, 6.0);
    assert_eq!(composed.measured_width, 6.0);
    let receipt = composed
        .horizontal_composition_receipt
        .as_deref()
        .expect("hybrid run must retain composition provenance");
    assert_eq!(
        receipt.alternate_ranges,
        vec![TextRange { start: 11, end: 12 }]
    );
    assert_eq!(
        receipt.first_failure.code,
        TextShapingFailureCode::BackendFaceParse
    );
    assert_eq!(
        receipt.first_failure.source_range,
        Some(TextRange { start: 11, end: 12 })
    );
}

#[test]
fn composition_rejects_alternate_glyph_that_crosses_a_hole() {
    let source: Arc<str> = Arc::from("abc");
    let partial = partial(
        source.clone(),
        vec![glyph(10, 10, 11, 1.0), glyph(30, 12, 13, 1.0)],
    );
    let alternate = run(source, 0, vec![glyph(99, 10, 12, 2.0)]);

    let (error, retained) = compose_horizontal_partial(
        partial,
        alternate,
        &FontDatabase::default(),
        12.0,
        12.0,
        failure_receipt(),
    )
    .expect_err("a glyph spanning direct and alternate ownership must fail closed");

    assert_eq!(error, HorizontalCompositionError::AlternateGlyphCrossesHole);
    assert_eq!(retained.lines[0].glyphs[0].glyph_id, 99);
}

#[test]
fn composition_rejects_hole_without_an_alternate_glyph() {
    let source: Arc<str> = Arc::from("abc");
    let partial = partial(
        source.clone(),
        vec![glyph(10, 10, 11, 1.0), glyph(30, 12, 13, 1.0)],
    );
    let alternate = run(
        source,
        0,
        vec![glyph(100, 10, 11, 1.0), glyph(300, 12, 13, 1.0)],
    );

    let (error, retained) = compose_horizontal_partial(
        partial,
        alternate,
        &FontDatabase::default(),
        12.0,
        12.0,
        failure_receipt(),
    )
    .expect_err("every failed direct range must have alternate coverage");

    assert_eq!(error, HorizontalCompositionError::MissingAlternateGlyph);
    assert_eq!(retained.lines[0].glyphs.len(), 2);
}

#[test]
fn composition_rejects_incompatible_line_topology() {
    let source: Arc<str> = Arc::from("abc");
    let partial = partial(
        source.clone(),
        vec![glyph(10, 10, 11, 1.0), glyph(30, 12, 13, 1.0)],
    );
    let alternate = run(source, 1, vec![glyph(20, 11, 12, 1.0)]);

    let (error, _) = compose_horizontal_partial(
        partial,
        alternate,
        &FontDatabase::default(),
        12.0,
        12.0,
        failure_receipt(),
    )
    .expect_err("line topology disagreement must retain the whole alternate run");

    assert_eq!(error, HorizontalCompositionError::IncompatibleLineTopology);
}

#[test]
fn composition_rejects_non_monotonic_alternate_source_order() {
    let source: Arc<str> = Arc::from("abc");
    let partial = partial(
        source.clone(),
        vec![glyph(10, 10, 11, 1.0), glyph(30, 12, 13, 1.0)],
    );
    let alternate = run(
        source,
        0,
        vec![glyph(30, 12, 13, 1.0), glyph(20, 11, 12, 1.0)],
    );

    let (error, _) = compose_horizontal_partial(
        partial,
        alternate,
        &FontDatabase::default(),
        12.0,
        12.0,
        failure_receipt(),
    )
    .expect_err("the linear hole scan requires monotonic source ranges");

    assert_eq!(error, HorizontalCompositionError::NonMonotonicGlyphOrder);
}

fn partial(source: Arc<str>, glyphs: Vec<ShapedGlyph>) -> HorizontalPartialShape {
    HorizontalPartialShape {
        direct: run(source, 0, glyphs),
        holes: vec![HorizontalDirectHole {
            range: TextRange { start: 1, end: 2 },
            error: DirectShapeError::InvalidSourceRange {
                range: TextRange { start: 1, end: 2 },
            },
        }],
    }
}

fn failure_receipt() -> TextShapingFailureReceipt {
    TextShapingFailureReceipt {
        code: TextShapingFailureCode::BackendFaceParse,
        phase: TextShapingFailurePhase::FontLoad,
        source_range: Some(TextRange { start: 1, end: 2 }),
        face: None,
        dependency: TextShapingFailureDependency::FontFace,
        disposition: TextShapingFailureDisposition::AlternateBackend,
        budget: None,
    }
}

fn run(source: Arc<str>, line_index: usize, glyphs: Vec<ShapedGlyph>) -> ShapedGlyphRun {
    let measured_width = glyphs.iter().map(|glyph| glyph.advance).sum();
    ShapedGlyphRun {
        source_text: source,
        source_range: TextRange { start: 10, end: 13 },
        unicode_data_snapshot: compiled_unicode_data_snapshot_id(),
        primary_face_id: None,
        direction: TextDirection::LeftToRight,
        orientation: TextOrientation::Horizontal,
        vertical_mode: VerticalMode::Mixed,
        include_kerning: true,
        measured_width,
        measured_height: 10.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![ShapedHardLine {
            line_index,
            source_range: TextRange { start: 10, end: 13 },
            visual_range: TextRange { start: 0, end: 3 },
            measured_width,
            baseline: 8.0,
            line_height: 10.0,
            glyphs,
        }],
    }
}

fn glyph(glyph_id: u32, start: usize, end: usize, advance: f32) -> ShapedGlyph {
    ShapedGlyph {
        glyph_id,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange { start, end },
        visual_range: TextRange {
            start: start - 10,
            end: end - 10,
        },
        advance,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::LeftToRight,
        bidi_level: 0,
        cluster_flags: ShapedGlyphClusterFlags {
            cluster_start: true,
            ..ShapedGlyphClusterFlags::default()
        },
        rotation: ShapedGlyphRotation::None,
        script: ShapedGlyphScript::default(),
    }
}
