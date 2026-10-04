use std::sync::Arc;

use crate::core::framework::text::TextDirection;
use crate::text::{
    compiled_unicode_data_snapshot_id, ShapedGlyphClusterFlags, ShapedGlyphRotation,
    ShapedGlyphRun, ShapedGlyphScript, ShapedHardLine, TextOrientation, VerticalMode,
};

use super::*;

#[test]
fn candidate_rejection_profile_codes_are_stable() {
    for (rejection, expected) in [
        (ArabicTatweelCandidateRejection::MissingInsertion, 1),
        (ArabicTatweelCandidateRejection::InvalidSourceIdentity, 2),
        (ArabicTatweelCandidateRejection::InvalidInsertionRange, 3),
        (ArabicTatweelCandidateRejection::MalformedClusterOrder, 4),
        (ArabicTatweelCandidateRejection::MissingInsertionCluster, 5),
        (ArabicTatweelCandidateRejection::MixedSourceCluster, 6),
        (ArabicTatweelCandidateRejection::MissingJoiningContext, 7),
        (ArabicTatweelCandidateRejection::InvalidGlyph, 8),
        (ArabicTatweelCandidateRejection::MissingFace, 9),
        (ArabicTatweelCandidateRejection::MixedClusterFace, 10),
        (ArabicTatweelCandidateRejection::JoiningFaceMismatch, 11),
        (ArabicTatweelCandidateRejection::NonRtlJoiningContext, 12),
        (ArabicTatweelCandidateRejection::NonExpandingCandidate, 13),
    ] {
        assert_eq!(rejection.profile_code(), expected);
    }
}
use crate::text::layout::{MeasuredTextLine, TextLineMetrics};

#[test]
fn accepts_independent_tatweel_cluster_in_one_rtl_face_run() {
    let measured = measured_candidate([FontFaceId(7); 3], [11, 12, 13], [0..2, 2..4, 4..6]);

    let receipt = validate_arabic_tatweel_candidate(&measured, "سـل", &[2], 20.0)
        .expect("same-face Tatweel cluster is safe to project");

    assert_eq!(receipt.width(), 30.0);
    assert_eq!(receipt.insertion_count(), 1);
}

#[test]
fn rejects_tatweel_cluster_mixed_with_source_owned_neighbor() {
    let measured = measured_candidate([FontFaceId(7); 3], [11, 12, 13], [0..2, 2..6, 4..6]);

    assert_eq!(
        validate_arabic_tatweel_candidate(&measured, "سـل", &[2], 20.0),
        Err(ArabicTatweelCandidateRejection::MixedSourceCluster)
    );
}

#[test]
fn rejects_tatweel_from_a_different_fallback_face() {
    let measured = measured_candidate(
        [FontFaceId(7), FontFaceId(8), FontFaceId(7)],
        [11, 12, 13],
        [0..2, 2..4, 4..6],
    );

    assert_eq!(
        validate_arabic_tatweel_candidate(&measured, "سـل", &[2], 20.0),
        Err(ArabicTatweelCandidateRejection::JoiningFaceMismatch)
    );
}

#[test]
fn rejects_missing_glyph_even_when_width_grew() {
    let measured = measured_candidate([FontFaceId(7); 3], [11, 0, 13], [0..2, 2..4, 4..6]);

    assert_eq!(
        validate_arabic_tatweel_candidate(&measured, "سـل", &[2], 20.0),
        Err(ArabicTatweelCandidateRejection::InvalidGlyph)
    );
}

fn measured_candidate(
    faces: [FontFaceId; 3],
    glyph_ids: [u32; 3],
    ranges: [std::ops::Range<usize>; 3],
) -> MeasuredTextLine {
    let glyphs = glyph_ids
        .into_iter()
        .zip(faces)
        .zip(ranges)
        .map(|((glyph_id, face), range)| ShapedGlyph {
            glyph_id,
            font_id: Some(face),
            font_instance_id: None,
            source_range: TextRange {
                start: range.start,
                end: range.end,
            },
            visual_range: TextRange {
                start: range.start,
                end: range.end,
            },
            advance: 10.0,
            x: 0.0,
            y: 0.0,
            offset_x: 0.0,
            offset_y: 0.0,
            direction: TextDirection::RightToLeft,
            bidi_level: 1,
            cluster_flags: ShapedGlyphClusterFlags {
                cluster_start: true,
                rtl: true,
                ..ShapedGlyphClusterFlags::default()
            },
            rotation: ShapedGlyphRotation::None,
            script: ShapedGlyphScript::default(),
        })
        .collect::<Vec<_>>();
    let shaped = Arc::new(ShapedGlyphRun {
        source_text: Arc::from("سـل"),
        source_range: TextRange { start: 0, end: 6 },
        unicode_data_snapshot: compiled_unicode_data_snapshot_id(),
        primary_face_id: Some(FontFaceId(7)),
        direction: TextDirection::RightToLeft,
        orientation: TextOrientation::Horizontal,
        vertical_mode: VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 30.0,
        measured_height: 12.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![ShapedHardLine {
            line_index: 0,
            source_range: TextRange { start: 0, end: 6 },
            visual_range: TextRange { start: 0, end: 6 },
            measured_width: 30.0,
            baseline: 9.0,
            line_height: 12.0,
            glyphs,
        }],
    });
    MeasuredTextLine {
        shaped,
        metrics: TextLineMetrics {
            width: 30.0,
            baseline: 9.0,
            line_height: 12.0,
        },
        grapheme_advances: vec![10.0; 3],
        glyph_clusters: Vec::new(),
    }
}
