use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{UiTextDirection, UiTextRange};

use crate::core::framework::text::TextLayoutError;
use crate::text::font::FontCollectionRevision;
use crate::text::layout::{
    LogicalVirtualFragmentRole, LogicalVirtualLineSequence, TextLineMetrics,
};
use crate::text::shaping::{
    BidiInvariantError, BidiLineOrder, TextLayoutOutcome, TextShapeRunProvider, TextShapingOutcome,
};
use crate::text::{TextRange, TextStyle};

use super::candidate_line::CandidateLine;
use super::physical_line_metrics::PhysicalLineFragments;
use super::visual_order;

pub(super) fn has_virtual_fragment(line: &CandidateLine) -> bool {
    line.runs
        .iter()
        .any(|run| !run.text.is_empty() && run.source_range.start == run.source_range.end)
}

/// Builds the private logical sidecar while the candidate still retains logical display order.
/// Any non-isomorphic source cluster refuses the sidecar but remains eligible for the resolved-
/// layout visual fallback; layout must not fail merely because the artifact cannot own a generated
/// run.
pub(super) fn capture(
    line: &CandidateLine,
    base_direction: UiTextDirection,
) -> Option<LogicalVirtualLineSequence> {
    capture_with_external_source_ranges(line, base_direction, &[])
}

pub(super) fn capture_with_external_source_ranges(
    line: &CandidateLine,
    base_direction: UiTextDirection,
    external_source_ranges: &[UiTextRange],
) -> Option<LogicalVirtualLineSequence> {
    if (!has_virtual_fragment(line) && external_source_ranges.is_empty())
        || line.text.is_empty()
        || line.runs.is_empty()
    {
        return None;
    }
    if external_source_ranges
        .iter()
        .any(|range| range.start >= range.end)
        || external_source_ranges
            .windows(2)
            .any(|ranges| ranges[0].end > ranges[1].start)
    {
        return None;
    }
    if line.virtual_source_receipts.iter().any(|receipt| {
        receipt.visual_range.start >= receipt.visual_range.end
            || receipt.visual_range.end > line.text.len()
    }) || line
        .virtual_source_receipts
        .windows(2)
        .any(|receipts| receipts[0].visual_range.end > receipts[1].visual_range.start)
    {
        return None;
    }

    let mut source_ranges = Vec::new();
    let mut style_owner_source_ranges = Vec::new();
    let mut replaced_source_ranges = Vec::new();
    let mut external_clusters = Vec::new();
    let mut virtual_roles = Vec::<Option<LogicalVirtualFragmentRole>>::new();
    let mut run_index = 0_usize;
    let mut external_range_index = 0_usize;
    let mut virtual_receipt_index = 0_usize;
    for (cluster_start, grapheme) in line.text.grapheme_indices(true) {
        let cluster_end = cluster_start + grapheme.len();
        while line
            .runs
            .get(run_index)
            .is_some_and(|run| run.visual_range.end <= cluster_start)
        {
            run_index = run_index.saturating_add(1);
        }
        let run = line.runs.get(run_index)?;
        if run.visual_range.start > cluster_start || run.visual_range.end < cluster_end {
            return None;
        }
        let source_range = source_range_for_cluster(
            run.source_range,
            run.text.len(),
            cluster_start.saturating_sub(run.visual_range.start),
            cluster_end.saturating_sub(run.visual_range.start),
        )?;
        while line
            .virtual_source_receipts
            .get(virtual_receipt_index)
            .is_some_and(|receipt| receipt.visual_range.end <= cluster_start)
        {
            virtual_receipt_index = virtual_receipt_index.saturating_add(1);
        }
        let receipt = line
            .virtual_source_receipts
            .get(virtual_receipt_index)
            .copied()
            .filter(|receipt| {
                receipt.visual_range.start <= cluster_start
                    && cluster_end <= receipt.visual_range.end
            });
        if source_range.start == source_range.end {
            receipt?;
        } else if receipt.is_some()
            || line
                .virtual_source_receipts
                .get(virtual_receipt_index)
                .is_some_and(|receipt| {
                    receipt.visual_range.start < cluster_end
                        && cluster_start < receipt.visual_range.end
                })
        {
            return None;
        }
        style_owner_source_ranges.push(receipt.map(|receipt| receipt.style_source_range.into()));
        replaced_source_ranges
            .push(receipt.and_then(|receipt| receipt.replaced_source_range.map(Into::into)));
        virtual_roles.push(receipt.map(|receipt| receipt.virtual_role));
        while external_source_ranges
            .get(external_range_index)
            .is_some_and(|range| range.end <= source_range.start)
        {
            external_range_index = external_range_index.saturating_add(1);
        }
        let external = external_source_ranges
            .get(external_range_index)
            .is_some_and(|range| TextRange::from(*range) == source_range);
        if external_source_ranges
            .get(external_range_index)
            .is_some_and(|range| {
                range.start < source_range.end && source_range.start < range.end && !external
            })
        {
            return None;
        }
        external_clusters.push(external);
        source_ranges.push(source_range);
    }
    LogicalVirtualLineSequence::new_with_source_receipts_external_clusters_and_roles(
        Arc::from(line.text.as_str()),
        base_direction.into(),
        source_ranges,
        style_owner_source_ranges,
        replaced_source_ranges,
        external_clusters,
        virtual_roles,
    )
}

/// Shapes generated display input and resolves final display order while logical candidates are
/// still available. Ordinary lines continue through the source-owned bidi path. A generated line
/// retains a private canonical fragment before its candidate becomes physical text, so metrics,
/// advances, and later glyph-artifact projection share one logical shape.
pub(super) fn shape_and_apply_visual_order_with_sequences<P>(
    lines: &mut [CandidateLine],
    paragraph_text: &str,
    base_direction: UiTextDirection,
    style: &TextStyle,
    provider: &mut P,
    visual_fragment_advances: &mut Option<Vec<Option<Vec<f32>>>>,
    physical_metrics: &mut [TextLineMetrics],
    physical_line_fragments: Option<&PhysicalLineFragments>,
) -> TextLayoutOutcome<Option<Vec<Option<LogicalVirtualLineSequence>>>>
where
    P: TextShapeRunProvider + ?Sized,
{
    let font_revision = provider.font_collection_revision();
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    let mut logical_virtual_fragment_shape_request_count = 0_usize;
    let mut sequences = lines.iter().any(has_virtual_fragment).then(|| {
        lines
            .iter()
            .map(|line| capture(line, base_direction))
            .collect::<Vec<_>>()
    });
    if let Some(sequences) = &mut sequences {
        let advances = visual_fragment_advances.get_or_insert_with(|| vec![None; lines.len()]);
        if advances.len() != lines.len() || sequences.len() != lines.len() {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        }
        for (index, sequence) in sequences.iter_mut().enumerate() {
            let Some(sequence) = sequence else {
                continue;
            };
            match sequence.shape_fragment_with_provider(style, provider) {
                TextShapingOutcome::Ready(()) => {
                    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
                    {
                        logical_virtual_fragment_shape_request_count =
                            logical_virtual_fragment_shape_request_count.saturating_add(1);
                    }
                    let Some(fragment) = sequence.fragment_for_revision(font_revision) else {
                        return TextShapingOutcome::deferred(
                            TextLayoutError::FontGenerationChanged,
                        );
                    };
                    advances[index] = Some(fragment.grapheme_advances().to_vec());
                }
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            }
        }
    }
    if provider.font_collection_revision() != font_revision {
        return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
    }
    #[cfg(any(feature = "profiling", feature = "profiling-tracy"))]
    if super::layout_profile_metrics_enabled() {
        crate::profile_counter!(
            "runtime",
            "logical_virtual_fragment_shape_request_count",
            logical_virtual_fragment_shape_request_count
        );
    }
    for (index, line) in lines.iter_mut().enumerate() {
        let canonical_order =
            physical_line_fragments.and_then(|fragments| fragments.visual_order_for_layout(index));
        let logical_advances = visual_fragment_advances
            .as_mut()
            .and_then(|advances| advances.get_mut(index))
            .and_then(Option::as_mut);
        if has_virtual_fragment(line) {
            let sequence = sequences
                .as_mut()
                .and_then(|sequences| sequences.get_mut(index))
                .and_then(Option::as_mut);
            if visual_order::apply_visual_order_with_virtual_sequence(
                line,
                base_direction,
                sequence,
                logical_advances,
            )
            .is_err()
            {
                match reject_virtual_sequence_to_renderer_fallback(
                    &mut sequences,
                    visual_fragment_advances,
                    index,
                ) {
                    TextShapingOutcome::Ready(()) => continue,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                }
            }
        } else if apply_non_virtual_visual_order(
            line,
            paragraph_text,
            base_direction,
            canonical_order,
            logical_advances,
        )
        .is_err()
        {
            return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
        }
    }
    apply_canonical_fragment_metrics(physical_metrics, sequences.as_deref(), font_revision)
        .map(|()| sequences)
}

fn apply_non_virtual_visual_order(
    line: &mut CandidateLine,
    paragraph_text: &str,
    base_direction: UiTextDirection,
    canonical_order: Option<&BidiLineOrder>,
    logical_advances: Option<&mut Vec<f32>>,
) -> Result<(), BidiInvariantError> {
    match (canonical_order, logical_advances) {
        (Some(order), Some(advances)) => {
            visual_order::apply_visual_order_from_bidi_order_with_advances(line, order, advances)
        }
        (Some(order), None) => visual_order::apply_visual_order_from_bidi_order(line, order),
        (None, Some(advances)) => {
            crate::profile_scope!("runtime", "text.layout", "resolve_visual_order_fallback");
            visual_order::apply_visual_order_with_advances(
                line,
                paragraph_text,
                base_direction,
                advances,
            )
        }
        (None, None) => {
            crate::profile_scope!("runtime", "text.layout", "resolve_visual_order_fallback");
            visual_order::apply_visual_order(line, paragraph_text, base_direction)
        }
    }
}

/// Rejects only the private virtual artifact route after an untrusted display-BiDi result.
///
/// The candidate remains untouched for the established resolved-layout renderer fallback. A
/// sequence/advance collection mismatch is an internal ownership violation, not content input,
/// and therefore remains a layout failure.
fn reject_virtual_sequence_to_renderer_fallback(
    sequences: &mut Option<Vec<Option<LogicalVirtualLineSequence>>>,
    visual_fragment_advances: &mut Option<Vec<Option<Vec<f32>>>>,
    index: usize,
) -> TextLayoutOutcome<()> {
    let Some(sequences) = sequences.as_mut() else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    let Some(sequence) = sequences.get_mut(index) else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    let Some(sequence) = sequence.as_mut() else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    sequence.reject_artifact_projection();

    let Some(advances) = visual_fragment_advances.as_mut() else {
        return TextShapingOutcome::Ready(());
    };
    let Some(advances) = advances.get_mut(index) else {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    };
    *advances = None;
    TextShapingOutcome::Ready(())
}

/// Replaces the sample fallback with final metrics from each retained logical virtual fragment.
/// The alignment check makes request-local sidecars fail closed if clipping or publication ever
/// loses their one-to-one correspondence with final physical lines.
pub(super) fn apply_canonical_fragment_metrics(
    physical_metrics: &mut [TextLineMetrics],
    sequences: Option<&[Option<LogicalVirtualLineSequence>]>,
    font_revision: FontCollectionRevision,
) -> TextLayoutOutcome<()> {
    let Some(sequences) = sequences else {
        return TextShapingOutcome::Ready(());
    };
    if sequences.len() != physical_metrics.len() {
        return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
    }
    for (metrics, sequence) in physical_metrics.iter_mut().zip(sequences) {
        let Some(sequence) = sequence else {
            continue;
        };
        if !sequence.artifact_projection_allowed() {
            continue;
        }
        let Some(fragment) = sequence.fragment_for_revision(font_revision) else {
            return TextShapingOutcome::deferred(TextLayoutError::FontGenerationChanged);
        };
        *metrics = fragment.metrics();
    }
    TextShapingOutcome::Ready(())
}

fn source_range_for_cluster(
    source_range: UiTextRange,
    text_len: usize,
    local_start: usize,
    local_end: usize,
) -> Option<TextRange> {
    if source_range.start == source_range.end {
        return Some(TextRange {
            start: source_range.start,
            end: source_range.end,
        });
    }
    let source_len = source_range.end.checked_sub(source_range.start)?;
    if source_len != text_len || local_start > local_end || local_end > text_len {
        return None;
    }
    Some(TextRange {
        start: source_range.start.checked_add(local_start)?,
        end: source_range.start.checked_add(local_end)?,
    })
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

    use super::super::candidate_line::{CandidateLine, append_segment, insert_virtual_text};
    use super::super::visual_order;
    use super::{
        apply_non_virtual_visual_order, capture, capture_with_external_source_ranges,
        has_virtual_fragment, reject_virtual_sequence_to_renderer_fallback,
        shape_and_apply_visual_order_with_sequences,
    };
    use crate::core::framework::text::TextDirection;
    use crate::text::shaping::{BidiLineOrder, TextShapeRunProvider, TextShapingOutcome};
    use crate::text::{
        ShapedGlyph, ShapedGlyphClusterFlags, ShapedGlyphRotation, ShapedGlyphRun,
        ShapedGlyphScript, ShapedHardLine, TextOrientation, TextRange, TextStyle, VerticalMode,
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
        assert!(
            sequence
                .logical_cluster_receipts()
                .all(|(_, _, _, replaced, external)| replaced.is_none() && !external)
        );
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

        assert!(
            capture(
                &line,
                zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft,
            )
            .is_none()
        );
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
        assert!(
            capture(
                &line,
                zircon_runtime_interface::ui::surface::UiTextDirection::LeftToRight
            )
            .is_none()
        );
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
        assert!(
            literal
                .logical_cluster_receipts()
                .all(|(_, _, _, _, external)| !external)
        );
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

        let outcome =
            reject_virtual_sequence_to_renderer_fallback(&mut sequences, &mut advances, 0);

        assert!(matches!(outcome, TextShapingOutcome::Ready(())));
        assert!(
            !sequences.expect("virtual sequence collection")[0]
                .as_ref()
                .expect("virtual sequence remains as a renderer-fallback marker")
                .artifact_projection_allowed()
        );
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

        let outcome =
            reject_virtual_sequence_to_renderer_fallback(&mut sequences, &mut advances, 0);

        assert!(matches!(outcome, TextShapingOutcome::Ready(())));
        assert!(
            !sequences.expect("virtual sequence collection")[0]
                .as_ref()
                .expect("virtual sequence remains as a renderer-fallback marker")
                .artifact_projection_allowed()
        );
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
}
