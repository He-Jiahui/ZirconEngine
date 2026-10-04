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
#[path = "tests/virtual_fragment_sequence.rs"]
mod tests;
