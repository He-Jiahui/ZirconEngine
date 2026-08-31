use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLine, UiTextCaret, UiTextCaretAffinity, UiTextRange,
};

use super::ResolvedTextGlyphArtifact;
use super::snapshot::matching_artifact_line;
use crate::core::framework::text::TextGlyph;
use crate::text::layout::{LogicalVirtualLineSequence, LogicalVisualClusterReceipt};
use crate::text::{TextRange, text_glyph_clusters};

#[derive(Clone, Copy)]
struct VisualSourceGeometryReceipt {
    visual_index: usize,
    source_range: TextRange,
    right_to_left: bool,
}

/// Returns the visual advance for an interior source offset that a shaped glyph keeps whole.
///
/// The serializable resolved-line DTO carries grapheme advances, while this process-local
/// artifact retains the backend glyph cluster ranges. Returning `None` leaves callers on the DTO
/// path when the layout no longer matches the artifact or the offset is already a legal boundary.
pub(crate) fn resolved_text_glyph_artifact_caret_advance(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
    caret: &UiTextCaret,
) -> Option<f32> {
    let glyphs = matching_artifact_line(artifact, line_index, layout_line)?;
    let sequence = matching_virtual_line_sequence(artifact, line_index, layout_line);
    if let Some(receipt) = unique_special_geometry_for_offset(sequence, caret.offset)? {
        let (leading, trailing) =
            visual_cluster_advance_span(&layout_line.glyph_advances, receipt.visual_index)?;
        let logical_start = matches!(caret.affinity, UiTextCaretAffinity::Upstream);
        return Some(if receipt.right_to_left == logical_start {
            trailing
        } else {
            leading
        });
    }
    let mut clusters = text_glyph_clusters(glyphs);
    let cluster = loop {
        let cluster = clusters.next()?;
        let right_to_left = cluster.right_to_left?;
        if cluster.source_range.start < caret.offset && caret.offset < cluster.source_range.end {
            break (cluster, right_to_left);
        }
    };
    let (leading, trailing) = if let Some(sequence) = sequence {
        let (leading, trailing, right_to_left) = sequence_glyph_cluster_advance_span(
            sequence,
            &layout_line.glyph_advances,
            cluster.0.source_range,
        )?;
        (right_to_left == cluster.1).then_some((leading, trailing))?
    } else {
        let leading = text_glyph_clusters(glyphs)
            .take_while(|candidate| candidate.glyph_start < cluster.0.glyph_start)
            .map(|candidate| candidate.advance)
            .sum::<f32>();
        (leading, leading + cluster.0.advance)
    };
    let logical_start = matches!(caret.affinity, UiTextCaretAffinity::Upstream);
    Some(if cluster.1 == logical_start {
        trailing
    } else {
        leading
    })
}

/// Resolves a physical visual advance to a legal source caret using backend glyph clusters.
///
/// The serialized DTO has one advance per visual grapheme, which cannot represent a ligature or
/// another backend cluster spanning multiple source offsets. Returning `None` keeps the caller on
/// the DTO source-map path when the text-owned glyph line is unavailable or no longer matches.
pub(crate) fn resolved_text_glyph_artifact_caret_at_advance(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
    visual_advance: f32,
) -> Option<UiTextCaret> {
    let glyphs = matching_artifact_line(artifact, line_index, layout_line)?;
    let visual_advance = finite_non_negative(visual_advance);
    if let Some(sequence) = matching_virtual_line_sequence(artifact, line_index, layout_line) {
        let (receipt, leading, trailing) =
            visual_receipt_for_advance(sequence, &layout_line.glyph_advances, visual_advance)?;
        if let Some(source_range) = special_geometry_source_range(receipt) {
            return Some(cluster_caret(
                source_range,
                receipt.right_to_left,
                visual_advance <= leading + (trailing - leading) * 0.5,
            ));
        }
        let cluster = text_glyph_clusters(glyphs).find(|cluster| {
            sequence_cluster_belongs_to_glyph(receipt.source_range, cluster.source_range)
        })?;
        let right_to_left = cluster.right_to_left?;
        let (leading, trailing, receipt_right_to_left) = sequence_glyph_cluster_advance_span(
            sequence,
            &layout_line.glyph_advances,
            cluster.source_range,
        )?;
        if receipt_right_to_left != right_to_left {
            return None;
        }
        return Some(cluster_caret(
            cluster.source_range,
            right_to_left,
            visual_advance <= leading + (trailing - leading) * 0.5,
        ));
    }
    let mut advance = 0.0;
    let mut clusters = text_glyph_clusters(glyphs).peekable();
    while let Some(cluster) = clusters.next() {
        let right_to_left = cluster.right_to_left?;
        if visual_advance <= advance + cluster.advance * 0.5 {
            return Some(cluster_caret(cluster.source_range, right_to_left, true));
        }
        advance += cluster.advance;
        if clusters.peek().is_none() {
            return Some(cluster_caret(cluster.source_range, right_to_left, false));
        }
    }
    None
}

/// Returns physical advance spans for source ranges that overlap shaped glyph clusters.
pub(crate) fn resolved_text_glyph_artifact_range_advance_spans(
    artifact: &ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
    range: UiTextRange,
) -> Option<Vec<(f32, f32)>> {
    let glyphs = matching_artifact_line(artifact, line_index, layout_line)?;
    if let Some(sequence) = matching_virtual_line_sequence(artifact, line_index, layout_line) {
        return sequence_range_advance_spans(sequence, glyphs, &layout_line.glyph_advances, range);
    }
    let mut spans = Vec::new();
    let mut span_start = None;
    let mut advance = 0.0;
    for cluster in text_glyph_clusters(glyphs) {
        cluster.right_to_left?;
        if source_ranges_overlap(cluster.source_range, range) {
            span_start.get_or_insert(advance);
        } else if let Some(start) = span_start.take() {
            spans.push((start, advance));
        }
        advance += cluster.advance;
    }
    if let Some(start) = span_start {
        spans.push((start, advance));
    }
    merge_advance_spans(spans)
}

fn matching_virtual_line_sequence<'a>(
    artifact: &'a ResolvedTextGlyphArtifact,
    line_index: usize,
    layout_line: &UiResolvedTextLine,
) -> Option<&'a LogicalVirtualLineSequence> {
    matching_artifact_line(artifact, line_index, layout_line)?;
    let sequence = artifact
        .logical_virtual_line_sequences
        .as_deref()?
        .get(line_index)?
        .as_ref()?;
    (sequence.artifact_projection_allowed()
        && sequence.visual_cluster_count() == layout_line.glyph_advances.len())
    .then_some(sequence)
}

fn unique_special_geometry_for_offset(
    sequence: Option<&LogicalVirtualLineSequence>,
    source_offset: usize,
) -> Option<Option<VisualSourceGeometryReceipt>> {
    let Some(sequence) = sequence else {
        return Some(None);
    };
    let mut matching = sequence
        .visual_cluster_receipts()
        .filter_map(|receipt| {
            special_geometry_source_range(receipt).map(|source_range| (receipt, source_range))
        })
        .filter(|(_, source_range)| {
            source_range.start < source_offset && source_offset < source_range.end
        });
    let receipt = matching
        .next()
        .map(|(receipt, source_range)| VisualSourceGeometryReceipt {
            visual_index: receipt.visual_index,
            source_range,
            right_to_left: receipt.right_to_left,
        });
    matching.next().is_none().then_some(receipt)
}

fn visual_receipt_for_advance(
    sequence: &LogicalVirtualLineSequence,
    advances: &[f32],
    visual_advance: f32,
) -> Option<(LogicalVisualClusterReceipt, f32, f32)> {
    let mut advance = 0.0;
    let mut last = None;
    for (receipt, cluster_advance) in sequence
        .visual_cluster_receipts()
        .zip(advances.iter().copied())
    {
        let trailing = advance + finite_non_negative(cluster_advance);
        last = Some((receipt, advance, trailing));
        if advance <= visual_advance && visual_advance < trailing {
            return last;
        }
        advance = trailing;
    }
    (visual_advance >= advance).then_some(last).flatten()
}

fn visual_cluster_advance_span(advances: &[f32], visual_index: usize) -> Option<(f32, f32)> {
    let marker_advance = finite_non_negative(*advances.get(visual_index)?);
    let leading = advances
        .iter()
        .take(visual_index)
        .copied()
        .map(finite_non_negative)
        .sum::<f32>();
    Some((leading, leading + marker_advance))
}

fn special_geometry_source_range(receipt: LogicalVisualClusterReceipt) -> Option<TextRange> {
    receipt
        .replaced_source_range
        .or(receipt.external.then_some(receipt.source_range))
}

fn sequence_glyph_cluster_advance_span(
    sequence: &LogicalVirtualLineSequence,
    advances: &[f32],
    glyph_source_range: TextRange,
) -> Option<(f32, f32, bool)> {
    let mut advance = 0.0;
    let mut span = None;
    let mut ended = false;
    let mut right_to_left = None;
    for (receipt, cluster_advance) in sequence
        .visual_cluster_receipts()
        .zip(advances.iter().copied())
    {
        let trailing = advance + finite_non_negative(cluster_advance);
        let belongs = !receipt.external
            && sequence_cluster_belongs_to_glyph(receipt.source_range, glyph_source_range);
        if belongs {
            if ended || right_to_left.is_some_and(|rtl| rtl != receipt.right_to_left) {
                return None;
            }
            right_to_left.get_or_insert(receipt.right_to_left);
            let leading = span.map_or(advance, |(leading, _)| leading);
            span = Some((leading, trailing));
        } else if span.is_some() {
            ended = true;
        }
        advance = trailing;
    }
    span.zip(right_to_left)
        .map(|((leading, trailing), right_to_left)| (leading, trailing, right_to_left))
}

fn sequence_cluster_belongs_to_glyph(
    sequence_source_range: TextRange,
    glyph_source_range: TextRange,
) -> bool {
    if sequence_source_range.start == sequence_source_range.end
        || glyph_source_range.start == glyph_source_range.end
    {
        sequence_source_range == glyph_source_range
    } else {
        glyph_source_range.start <= sequence_source_range.start
            && sequence_source_range.end <= glyph_source_range.end
    }
}

fn sequence_range_advance_spans(
    sequence: &LogicalVirtualLineSequence,
    glyphs: &[TextGlyph],
    advances: &[f32],
    range: UiTextRange,
) -> Option<Vec<(f32, f32)>> {
    let mut glyph_clusters = text_glyph_clusters(glyphs);
    let mut current_glyph = glyph_clusters.next();
    let mut spans = Vec::new();
    let mut advance = 0.0;
    for (receipt, cluster_advance) in sequence
        .visual_cluster_receipts()
        .zip(advances.iter().copied())
    {
        let trailing = advance + finite_non_negative(cluster_advance);
        let source_range = if let Some(source_range) = special_geometry_source_range(receipt) {
            source_range
        } else {
            while current_glyph.as_ref().is_some_and(|glyph| {
                !sequence_cluster_belongs_to_glyph(receipt.source_range, glyph.source_range)
            }) {
                current_glyph = glyph_clusters.next();
            }
            let glyph = current_glyph.as_ref()?;
            glyph.right_to_left?;
            glyph.source_range
        };
        if source_ranges_overlap(source_range, range) {
            spans.push((advance, trailing));
        }
        advance = trailing;
    }
    merge_advance_spans(spans)
}

fn merge_advance_spans(mut spans: Vec<(f32, f32)>) -> Option<Vec<(f32, f32)>> {
    spans.sort_by(|left, right| left.0.total_cmp(&right.0));
    let mut merged = Vec::<(f32, f32)>::with_capacity(spans.len());
    for span in spans {
        if let Some(previous) = merged.last_mut() {
            if span.0 <= previous.1 {
                previous.1 = previous.1.max(span.1);
                continue;
            }
        }
        merged.push(span);
    }
    (!merged.is_empty()).then_some(merged)
}

fn cluster_caret(
    source_range: TextRange,
    right_to_left: bool,
    leading_visual_edge: bool,
) -> UiTextCaret {
    let offset = if right_to_left == leading_visual_edge {
        source_range.end
    } else {
        source_range.start
    };
    UiTextCaret {
        offset,
        affinity: if leading_visual_edge {
            UiTextCaretAffinity::Downstream
        } else {
            UiTextCaretAffinity::Upstream
        },
    }
}

fn source_ranges_overlap(source_range: TextRange, range: UiTextRange) -> bool {
    range.start < source_range.end && source_range.start < range.end
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
