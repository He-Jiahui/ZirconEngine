use crate::text::shaping::{
    BidiInvariantError, BidiLineOrder, analyze_bidi_line, mirrored_bidi_char,
};
use crate::text::{TextRange, layout::LogicalVirtualLineSequence};
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind,
};

use super::super::grapheme::grapheme_indices;
use super::candidate_line::CandidateLine;
use super::range_mapping::source_subrange;

#[derive(Clone, Debug)]
struct VisualTextToken {
    owner_run_index: usize,
    text_range: std::ops::Range<usize>,
    source_range: UiTextRange,
}

#[derive(Clone, Debug)]
struct VisualTextCluster {
    logical_range: UiTextRange,
    token_range: std::ops::Range<usize>,
}

#[derive(Clone, Debug)]
struct VisualRunProjection {
    owner_run_index: usize,
    kind: UiTextRunKind,
    visual_range: std::ops::Range<usize>,
    source_range: UiTextRange,
    direction: UiTextDirection,
}

/// Projects the shared Text 02 UAX#9 line order into the existing resolved-line
/// adapter. Source ranges remain logical; only the line text/run traversal is
/// materialized in visual order for current UI consumers.
pub(super) fn apply_visual_order(
    line: &mut CandidateLine,
    paragraph_text: &str,
    base_direction: UiTextDirection,
) -> Result<(), BidiInvariantError> {
    apply_visual_order_inner(line, paragraph_text, base_direction, None)
}

pub(super) fn apply_visual_order_with_advances(
    line: &mut CandidateLine,
    paragraph_text: &str,
    base_direction: UiTextDirection,
    logical_advances: &mut Vec<f32>,
) -> Result<(), BidiInvariantError> {
    apply_visual_order_inner(line, paragraph_text, base_direction, Some(logical_advances))
}

/// Generated display fragments have no non-empty source range that the source-owned bidi
/// signature can consume. Resolve UAX#9 from the logical display sequence instead, then retain
/// the exact visual permutation in the private sidecar for artifact shaping.
pub(super) fn apply_visual_order_with_virtual_sequence(
    line: &mut CandidateLine,
    base_direction: UiTextDirection,
    sequence: Option<&mut LogicalVirtualLineSequence>,
    logical_advances: Option<&mut Vec<f32>>,
) -> Result<(), BidiInvariantError> {
    if line.runs.is_empty() || line.text.is_empty() {
        return Ok(());
    }
    let (clusters, tokens) = logical_text_clusters(&line.runs, &line.text);
    let display_ranges = grapheme_indices(&line.text)
        .map(|(start, grapheme)| TextRange {
            start,
            end: start + grapheme.len(),
        })
        .collect::<Vec<_>>();
    if display_ranges.len() != clusters.len() {
        return Err(BidiInvariantError::ProjectionCardinalityMismatch {
            cluster_count: clusters.len(),
            visual_index_count: display_ranges.len(),
            level_count: display_ranges.len(),
        });
    }
    let order = analyze_bidi_line(
        &line.text,
        base_direction.into(),
        TextRange {
            start: 0,
            end: line.text.len(),
        },
        &display_ranges,
    )?;
    if let Some(sequence) = sequence {
        sequence.record_visual_order(&order)?;
    }
    apply_visual_order_from_bidi_order_with_clusters(
        line,
        &clusters,
        &tokens,
        &order,
        logical_advances,
        false,
    )
}

/// Applies source-owned UAX#9 ordering that was resolved before the display string was
/// substituted. Secure text uses this path so neutral mask glyphs are never analyzed as `Auto`.
pub(super) fn apply_visual_order_from_bidi_order(
    line: &mut CandidateLine,
    order: &BidiLineOrder,
) -> Result<(), BidiInvariantError> {
    apply_visual_order_from_bidi_order_inner(line, order, None, false)
}

pub(super) fn apply_visual_order_from_bidi_order_with_advances(
    line: &mut CandidateLine,
    order: &BidiLineOrder,
    logical_advances: &mut Vec<f32>,
) -> Result<(), BidiInvariantError> {
    apply_visual_order_from_bidi_order_inner(line, order, Some(logical_advances), false)
}

/// Secure presentation keeps one run per display grapheme so its source-offset map cannot infer
/// non-isomorphic UTF-8 ranges from a merged sequence of mask glyphs.
pub(super) fn apply_visual_order_from_bidi_order_for_presentation_with_advances(
    line: &mut CandidateLine,
    order: &BidiLineOrder,
    logical_advances: &mut Vec<f32>,
) -> Result<(), BidiInvariantError> {
    apply_visual_order_from_bidi_order_inner(line, order, Some(logical_advances), true)
}

fn apply_visual_order_inner(
    line: &mut CandidateLine,
    paragraph_text: &str,
    base_direction: UiTextDirection,
    logical_advances: Option<&mut Vec<f32>>,
) -> Result<(), BidiInvariantError> {
    if line.runs.is_empty() || line.text.is_empty() {
        return Ok(());
    }
    let (clusters, tokens) = logical_text_clusters(&line.runs, &line.text);
    let ranges = clusters
        .iter()
        .map(|cluster| cluster.logical_range.into())
        .collect::<Vec<_>>();
    let order = analyze_bidi_line(
        paragraph_text,
        base_direction.into(),
        line.source_range.into(),
        &ranges,
    )?;
    apply_visual_order_from_bidi_order_with_clusters(
        line,
        &clusters,
        &tokens,
        &order,
        logical_advances,
        false,
    )
}

fn apply_visual_order_from_bidi_order_inner(
    line: &mut CandidateLine,
    order: &BidiLineOrder,
    logical_advances: Option<&mut Vec<f32>>,
    preserve_cluster_runs: bool,
) -> Result<(), BidiInvariantError> {
    if line.runs.is_empty() || line.text.is_empty() {
        return Ok(());
    }
    let (clusters, tokens) = logical_text_clusters(&line.runs, &line.text);
    apply_visual_order_from_bidi_order_with_clusters(
        line,
        &clusters,
        &tokens,
        order,
        logical_advances,
        preserve_cluster_runs,
    )
}

fn apply_visual_order_from_bidi_order_with_clusters(
    line: &mut CandidateLine,
    clusters: &[VisualTextCluster],
    tokens: &[VisualTextToken],
    order: &BidiLineOrder,
    logical_advances: Option<&mut Vec<f32>>,
    preserve_cluster_runs: bool,
) -> Result<(), BidiInvariantError> {
    if order.visual_indices.len() != clusters.len() || order.logical_levels.len() != clusters.len()
    {
        return Err(BidiInvariantError::ProjectionCardinalityMismatch {
            cluster_count: clusters.len(),
            visual_index_count: order.visual_indices.len(),
            level_count: order.logical_levels.len(),
        });
    }
    let reordered_advances = match logical_advances.as_deref() {
        Some(advances) if advances.len() == clusters.len() => Some(
            order
                .visual_indices
                .iter()
                .map(|logical_index| advances[*logical_index])
                .collect::<Vec<_>>(),
        ),
        Some(advances) => {
            return Err(BidiInvariantError::AdvanceCardinalityMismatch {
                cluster_count: clusters.len(),
                advance_count: advances.len(),
            });
        }
        None => None,
    };

    let mut visual_text = String::with_capacity(line.text.len());
    let mut visual_runs = Vec::new();
    for logical_index in order.visual_indices.iter().copied() {
        let Some(cluster) = clusters.get(logical_index) else {
            return Err(BidiInvariantError::MissingLogicalCluster {
                logical_index,
                cluster_count: clusters.len(),
            });
        };
        let bidi_level = order.logical_levels[logical_index];
        push_visual_cluster(
            &mut visual_text,
            &mut visual_runs,
            &line.runs,
            &tokens[cluster.token_range.clone()],
            bidi_level,
            preserve_cluster_runs,
        );
    }

    let visual_runs = visual_runs
        .into_iter()
        .filter_map(|projection: VisualRunProjection| {
            Some(UiResolvedTextRun {
                kind: projection.kind,
                text: visual_text
                    .get(projection.visual_range.clone())?
                    .to_string(),
                source_range: projection.source_range,
                visual_range: UiTextRange {
                    start: projection.visual_range.start,
                    end: projection.visual_range.end,
                },
                direction: projection.direction,
            })
        })
        .collect();
    line.text = visual_text;
    line.runs = visual_runs;
    if let (Some(advances), Some(reordered_advances)) = (logical_advances, reordered_advances) {
        *advances = reordered_advances;
    }
    Ok(())
}

fn direction_for_bidi_level(bidi_level: u8) -> UiTextDirection {
    if bidi_level % 2 == 1 {
        UiTextDirection::RightToLeft
    } else {
        UiTextDirection::LeftToRight
    }
}

fn logical_text_clusters(
    runs: &[UiResolvedTextRun],
    logical_text: &str,
) -> (Vec<VisualTextCluster>, Vec<VisualTextToken>) {
    let mut clusters = Vec::<VisualTextCluster>::new();
    let mut tokens = Vec::<VisualTextToken>::new();
    let mut owner_run_index = 0;
    for (cluster_start, grapheme) in grapheme_indices(logical_text) {
        let cluster_end = cluster_start + grapheme.len();
        while runs
            .get(owner_run_index)
            .is_some_and(|run| run.visual_range.end <= cluster_start)
        {
            owner_run_index += 1;
        }
        let first_token = tokens.len();
        let mut run_index = owner_run_index;
        while let Some(run) = runs.get(run_index) {
            if run.visual_range.start >= cluster_end {
                break;
            }
            let part_start = cluster_start.max(run.visual_range.start);
            let part_end = cluster_end.min(run.visual_range.end);
            if part_start < part_end {
                tokens.push(visual_token(
                    run_index,
                    run,
                    part_start - run.visual_range.start,
                    part_end - run.visual_range.start,
                ));
            }
            run_index += 1;
        }
        let after_last_token = tokens.len();
        if let (Some(first), Some(last)) = (
            tokens.get(first_token),
            tokens.get(after_last_token.saturating_sub(1)),
        ) {
            clusters.push(VisualTextCluster {
                logical_range: UiTextRange {
                    start: first.source_range.start.min(last.source_range.start),
                    end: first.source_range.end.max(last.source_range.end),
                },
                token_range: first_token..after_last_token,
            });
        }
    }
    (clusters, tokens)
}

fn visual_token(
    owner_run_index: usize,
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
) -> VisualTextToken {
    VisualTextToken {
        owner_run_index,
        text_range: start..end,
        source_range: source_subrange(run.source_range, run.text.len(), start, end),
    }
}

fn push_visual_cluster(
    visual_text: &mut String,
    projections: &mut Vec<VisualRunProjection>,
    runs: &[UiResolvedTextRun],
    tokens: &[VisualTextToken],
    bidi_level: u8,
    preserve_cluster_runs: bool,
) {
    let direction = direction_for_bidi_level(bidi_level);
    for token in tokens {
        let Some(run) = runs.get(token.owner_run_index) else {
            continue;
        };
        let Some(text) = run.text.get(token.text_range.clone()) else {
            continue;
        };
        push_visual_fragment(
            visual_text,
            projections,
            token.owner_run_index,
            run.kind,
            text,
            token.source_range,
            direction,
            bidi_level,
            preserve_cluster_runs,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn push_visual_fragment(
    visual_text: &mut String,
    projections: &mut Vec<VisualRunProjection>,
    owner_run_index: usize,
    kind: UiTextRunKind,
    text: &str,
    source_range: UiTextRange,
    direction: UiTextDirection,
    bidi_level: u8,
    preserve_cluster_runs: bool,
) {
    let visual_start = visual_text.len();
    push_mirrored_text(visual_text, text, bidi_level);
    let visual_end = visual_text.len();
    if !preserve_cluster_runs {
        if let Some(last) = projections.last_mut() {
            if last.owner_run_index == owner_run_index
                && last.kind == kind
                && last.direction == direction
                && last.source_range.end == source_range.start
                && last.visual_range.end == visual_start
            {
                last.visual_range.end = visual_end;
                last.source_range.end = source_range.end;
                return;
            }
        }
    }
    projections.push(VisualRunProjection {
        owner_run_index,
        kind,
        visual_range: visual_start..visual_end,
        source_range,
        direction,
    });
}

fn push_mirrored_text(output: &mut String, text: &str, bidi_level: u8) {
    let mut chars = text.chars();
    let Some(character) = chars.next() else {
        return;
    };
    if chars.next().is_none() {
        if let Some(mirrored) = mirrored_bidi_char(character, bidi_level) {
            output.push(mirrored);
            return;
        }
    }
    output.push_str(text);
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::surface::{UiTextRange, UiTextRunKind};

    use super::super::candidate_line::{CandidateLine, append_segment, insert_virtual_text};
    use super::super::virtual_fragment_sequence::capture;
    use super::apply_visual_order_with_virtual_sequence;

    #[test]
    fn virtual_tatweel_uses_display_bidi_order_before_the_line_is_materialized() {
        let mut line = CandidateLine::empty();
        append_segment(
            &mut line,
            UiTextRunKind::Plain,
            "سلام",
            UiTextRange { start: 0, end: 8 },
        );
        assert!(insert_virtual_text(&mut line, 2, "ـ"));
        let mut sequence = capture(
            &line,
            zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft,
        )
        .expect("virtual source anchor retains the logical sidecar");

        apply_visual_order_with_virtual_sequence(
            &mut line,
            zircon_runtime_interface::ui::surface::UiTextDirection::RightToLeft,
            Some(&mut sequence),
            None,
        )
        .expect("display-owned UAX#9 accepts zero-width source anchors");

        assert_eq!(line.text, "مالـس");
        assert!(line.runs.iter().any(|run| {
            run.text == "ـ" && run.source_range.start == 2 && run.source_range.end == 2
        }));
    }
}
