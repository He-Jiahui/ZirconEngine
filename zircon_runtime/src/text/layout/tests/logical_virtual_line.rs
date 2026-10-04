use std::sync::Arc;

use crate::core::framework::text::{
    TextDirection, TextGlyph, TextGlyphFlags, TextGlyphRotation as ProjectedGlyphRotation,
};
use crate::text::shaping::{TextShapeRunProvider, TextShapingOutcome};
use crate::text::{
    ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun, ShapedGlyphScript,
    ShapedHardLine, TextOrientation, TextRange, TextStyle, VerticalMode,
};

use super::{LogicalVirtualFragmentRole, LogicalVirtualLineSequence, LogicalVirtualSourceReceipt};

#[test]
fn logical_virtual_sequence_projects_ellipsis_to_its_zero_width_anchor() {
    let mut sequence = LogicalVirtualLineSequence::new(
        Arc::from("a…b"),
        TextDirection::LeftToRight,
        vec![range(0, 1), range(1, 1), range(1, 2)],
    )
    .expect("virtual display clusters create a logical sidecar");
    sequence
        .resolve_visual_order()
        .expect("display-owned UAX#9 order");

    let glyphs = sequence
        .project_logical_glyphs(
            vec![glyph(0, 0..1), glyph(1, 1..4), glyph(2, 4..5)],
            &[4.0, 16.0, 4.0],
        )
        .expect("logical glyphs project to the visual sequence");

    assert_eq!(glyphs[0].source_range, 0..1);
    assert_eq!(glyphs[0].advance, 4.0);
    assert_eq!(glyphs[1].source_range, 1..1);
    assert_eq!(glyphs[1].advance, 16.0);
    assert!(glyphs[1].flags.virtual_glyph);
    assert_eq!(glyphs[2].source_range, 1..2);
    assert_eq!(glyphs[2].advance, 4.0);
}

#[test]
fn logical_virtual_projection_keeps_multicluster_glyph_advance_finite() {
    let mut sequence = LogicalVirtualLineSequence::new(
        Arc::from("ab"),
        TextDirection::LeftToRight,
        vec![range(0, 1), range(1, 2)],
    )
    .expect("two visual clusters create a logical sidecar");
    sequence
        .resolve_visual_order()
        .expect("display-owned UAX#9 order");

    let glyphs = sequence
        .project_logical_glyphs(vec![glyph(0, 0..2)], &[f32::MAX, f32::MAX])
        .expect("one backend glyph may own both logical clusters");

    assert_eq!(glyphs.len(), 1);
    assert_eq!(glyphs[0].advance, f32::MAX);
    assert!(glyphs[0].advance.is_finite());
}

#[test]
fn logical_virtual_sequence_keeps_external_clusters_out_of_glyph_projection() {
    let mut sequence = LogicalVirtualLineSequence::new_with_source_receipts_and_external_clusters(
        Arc::from("a\u{fffc}\u{2026}"),
        TextDirection::LeftToRight,
        vec![range(0, 1), range(1, 4), range(6, 6)],
        vec![None, None, Some(range(4, 6))],
        vec![None, None, Some(range(4, 6))],
        vec![false, true, false],
    )
    .expect("external display cluster creates a logical sidecar");
    sequence
        .resolve_visual_order()
        .expect("display-owned UAX#9 order");

    let glyphs = sequence
        .project_logical_glyphs(vec![glyph(1, 0..1), glyph(2, 4..7)], &[4.0, 16.0, 8.0])
        .expect("text glyphs project around the external cluster");

    assert_eq!(glyphs.len(), 2);
    assert_eq!(glyphs[0].source_range, 0..1);
    assert_eq!(glyphs[0].advance, 4.0);
    assert_eq!(glyphs[1].source_range, 6..6);
    assert_eq!(glyphs[1].advance, 8.0);
    assert!(glyphs[1].flags.virtual_glyph);
}

#[test]
fn logical_virtual_sequence_reorders_rtl_tatweel_without_losing_its_anchor() {
    let mut sequence = LogicalVirtualLineSequence::new(
        Arc::from("سـلام"),
        TextDirection::RightToLeft,
        vec![
            range(0, 2),
            range(2, 2),
            range(2, 4),
            range(4, 6),
            range(6, 8),
        ],
    )
    .expect("tatweel creates a virtual display cluster");
    sequence
        .resolve_visual_order()
        .expect("RTL display sequence resolves UAX#9 order");

    let glyphs = sequence
        .project_logical_glyphs(
            vec![
                glyph(0, 0..2),
                glyph(1, 2..4),
                glyph(2, 4..6),
                glyph(3, 6..8),
                glyph(4, 8..10),
            ],
            &[1.0, 2.0, 3.0, 4.0, 5.0],
        )
        .expect("logical RTL glyphs project to physical order");

    assert_eq!(glyphs.len(), 5);
    assert!(glyphs.iter().all(|glyph| glyph.flags.right_to_left));
    let tatweel = glyphs
        .iter()
        .find(|glyph| glyph.flags.virtual_glyph)
        .expect("tatweel remains a virtual glyph");
    assert_eq!(tatweel.source_range, 2..2);
    assert!(tatweel.advance > 0.0);
}

#[test]
fn logical_virtual_sequence_identity_tracks_rebuild_input_and_rejection() {
    let first = LogicalVirtualLineSequence::new(
        Arc::from("a\u{2026}b"),
        TextDirection::LeftToRight,
        vec![range(0, 1), range(1, 1), range(1, 2)],
    )
    .expect("virtual display clusters create a logical sidecar");
    let mut same = first.clone();

    assert!(first.has_same_artifact_identity(&same));
    same.reject_artifact_projection();
    assert!(!first.has_same_artifact_identity(&same));
}

#[test]
fn logical_virtual_sequence_identity_tracks_explicit_source_receipts() {
    let first = LogicalVirtualLineSequence::new_with_source_receipts(
        Arc::from("\u{2026}b"),
        TextDirection::LeftToRight,
        vec![range(3, 3), range(3, 4)],
        vec![Some(range(3, 4)), None],
        vec![Some(range(0, 3)), None],
    )
    .expect("virtual display clusters create a logical sidecar");
    let different = LogicalVirtualLineSequence::new_with_source_receipts(
        Arc::from("\u{2026}b"),
        TextDirection::LeftToRight,
        vec![range(3, 3), range(3, 4)],
        vec![Some(range(3, 4)), None],
        vec![Some(range(0, 2)), None],
    )
    .expect("virtual display clusters create a logical sidecar");

    assert!(!first.has_same_artifact_identity(&different));
    assert_eq!(
        first.visual_source_receipts(),
        vec![
            Some(LogicalVirtualSourceReceipt {
                style_source_range: range(3, 4),
                replaced_source_range: Some(range(0, 3)),
            }),
            None,
        ]
    );
}

#[test]
fn logical_virtual_sequence_identity_tracks_typed_virtual_role() {
    let soft_hyphen =
        LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
            Arc::from("a-"),
            TextDirection::LeftToRight,
            vec![range(0, 1), range(3, 3)],
            vec![None, Some(range(1, 3))],
            vec![None, Some(range(1, 3))],
            vec![false, false],
            vec![None, Some(LogicalVirtualFragmentRole::DiscretionaryHyphen)],
        )
        .expect("typed virtual sequence");
    let mut ellipsis = soft_hyphen.clone();
    ellipsis.clusters[1].virtual_role = Some(LogicalVirtualFragmentRole::Ellipsis);

    assert!(!soft_hyphen.has_same_artifact_identity(&ellipsis));
}

#[test]
fn logical_virtual_sequence_rejects_role_and_display_mismatch() {
    assert!(
        LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
            Arc::from("a-"),
            TextDirection::LeftToRight,
            vec![range(0, 1), range(3, 3)],
            vec![None, Some(range(1, 3))],
            vec![None, Some(range(1, 3))],
            vec![false, false],
            vec![None, Some(LogicalVirtualFragmentRole::Ellipsis)],
        )
        .is_none()
    );
}

#[test]
fn logical_virtual_sequence_reuses_one_shape_for_metrics_and_advances() {
    let mut sequence = LogicalVirtualLineSequence::new(
        Arc::from("a\u{2026}b"),
        TextDirection::LeftToRight,
        vec![range(0, 1), range(1, 1), range(1, 2)],
    )
    .expect("virtual display clusters create a logical sidecar");
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
        shaped: Arc::clone(&shaped),
        shape_calls: 0,
    };

    sequence
        .shape_fragment_with_provider(&TextStyle::default(), &mut provider)
        .into_result()
        .expect("shape the virtual logical fragment");
    sequence
        .shape_fragment_with_provider(&TextStyle::default(), &mut provider)
        .into_result()
        .expect("reuse the current-generation fragment");

    let fragment = sequence
        .fragment_for_revision(provider.font_collection_revision())
        .expect("current generation retains the logical fragment");
    assert!(Arc::ptr_eq(fragment.shaped(), &shaped));
    assert_eq!(fragment.metrics().baseline, 14.0);
    assert_eq!(fragment.metrics().line_height, 19.0);
    assert_eq!(fragment.grapheme_advances(), &[4.0, 13.0, 4.0]);
    assert_eq!(fragment.glyph_clusters().len(), 3);
    assert_eq!(provider.shape_calls, 1);
}

fn range(start: usize, end: usize) -> TextRange {
    TextRange { start, end }
}

fn glyph(glyph_id: u32, source_range: std::ops::Range<usize>) -> TextGlyph {
    TextGlyph {
        glyph_id,
        source_range: source_range.clone(),
        visual_range: source_range,
        advance: 1.0,
        position: [0.0, 0.0],
        offset: [0.0, 0.0],
        font_face: None,
        font_instance: None,
        rotation: ProjectedGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags {
            cluster_start: true,
            ..TextGlyphFlags::default()
        },
        requires_rasterization: false,
    }
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
