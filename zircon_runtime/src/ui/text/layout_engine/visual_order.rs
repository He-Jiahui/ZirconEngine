use crate::text::shaping::{analyze_bidi_line, mirrored_bidi_char};
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind,
};

use super::super::grapheme::{grapheme_indices, leading_grapheme_continuation_len};
use super::candidate_line::CandidateLine;
use super::range_mapping::source_subrange;

#[derive(Clone, Debug)]
struct VisualTextToken {
    owner_run_index: usize,
    kind: UiTextRunKind,
    text: String,
    source_range: UiTextRange,
}

#[derive(Clone, Debug)]
struct VisualTextCluster {
    logical_range: UiTextRange,
    parts: Vec<VisualTextToken>,
}

#[derive(Clone, Debug)]
struct VisualTextFragment {
    owner_run_index: usize,
    kind: UiTextRunKind,
    text: String,
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
) {
    apply_visual_order_inner(line, paragraph_text, base_direction, None);
}

pub(super) fn apply_visual_order_with_advances(
    line: &mut CandidateLine,
    paragraph_text: &str,
    base_direction: UiTextDirection,
    logical_advances: &mut Vec<f32>,
) {
    apply_visual_order_inner(line, paragraph_text, base_direction, Some(logical_advances));
}

fn apply_visual_order_inner(
    line: &mut CandidateLine,
    paragraph_text: &str,
    base_direction: UiTextDirection,
    logical_advances: Option<&mut Vec<f32>>,
) {
    if line.runs.is_empty() || line.text.is_empty() {
        return;
    }
    let clusters = logical_text_clusters(&line.runs);
    let ranges = clusters
        .iter()
        .map(|cluster| cluster.logical_range.into())
        .collect::<Vec<_>>();
    let order = analyze_bidi_line(
        paragraph_text,
        base_direction.into(),
        line.source_range.into(),
        &ranges,
    );
    if order.visual_indices.len() != clusters.len() || order.logical_levels.len() != clusters.len()
    {
        return;
    }
    let reordered_advances = match logical_advances.as_deref() {
        Some(advances) if advances.len() == clusters.len() => Some(
            order
                .visual_indices
                .iter()
                .map(|logical_index| advances[*logical_index])
                .collect::<Vec<_>>(),
        ),
        Some(_) => return,
        None => None,
    };

    let mut visual_fragments = Vec::new();
    for logical_index in order.visual_indices.iter().copied() {
        let Some(cluster) = clusters.get(logical_index).cloned() else {
            return;
        };
        let bidi_level = order.logical_levels[logical_index];
        push_visual_cluster(&mut visual_fragments, cluster, bidi_level);
    }

    let mut visual_text = String::new();
    let mut visual_runs = Vec::with_capacity(visual_fragments.len());
    for fragment in visual_fragments {
        let visual_start = visual_text.len();
        visual_text.push_str(&fragment.text);
        visual_runs.push(UiResolvedTextRun {
            kind: fragment.kind,
            text: fragment.text,
            source_range: fragment.source_range,
            visual_range: UiTextRange {
                start: visual_start,
                end: visual_text.len(),
            },
            direction: fragment.direction,
        });
    }
    line.text = visual_text;
    line.runs = visual_runs;
    if let (Some(advances), Some(reordered_advances)) = (logical_advances, reordered_advances) {
        *advances = reordered_advances;
    }
}

fn direction_for_bidi_level(bidi_level: u8) -> UiTextDirection {
    if bidi_level % 2 == 1 {
        UiTextDirection::RightToLeft
    } else {
        UiTextDirection::LeftToRight
    }
}

fn logical_text_clusters(runs: &[UiResolvedTextRun]) -> Vec<VisualTextCluster> {
    let mut clusters = Vec::<VisualTextCluster>::new();
    let mut emitted_text = String::new();
    for (owner_run_index, run) in runs.iter().enumerate() {
        let mut consumed = 0;
        if !clusters.is_empty() {
            let continuation_len = leading_grapheme_continuation_len(&emitted_text, &run.text);
            if continuation_len > 0 {
                let token = visual_token(owner_run_index, run, 0, continuation_len);
                if let Some(cluster) = clusters.last_mut() {
                    cluster.logical_range.end = token.source_range.end;
                    cluster.parts.push(token);
                    emitted_text.push_str(&run.text[..continuation_len]);
                    consumed = continuation_len;
                }
            }
        }

        for (offset, grapheme) in grapheme_indices(&run.text[consumed..]) {
            let offset = consumed + offset;
            let end = offset + grapheme.len();
            clusters.push(VisualTextCluster {
                logical_range: source_subrange(run.source_range, run.text.len(), offset, end),
                parts: vec![visual_token(owner_run_index, run, offset, end)],
            });
            emitted_text.push_str(grapheme);
        }
    }
    clusters.retain(|cluster| !cluster.parts.is_empty());
    clusters
}

fn visual_token(
    owner_run_index: usize,
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
) -> VisualTextToken {
    VisualTextToken {
        owner_run_index,
        kind: run.kind,
        text: run.text[start..end].to_string(),
        source_range: source_subrange(run.source_range, run.text.len(), start, end),
    }
}

fn push_visual_cluster(
    fragments: &mut Vec<VisualTextFragment>,
    cluster: VisualTextCluster,
    bidi_level: u8,
) {
    let direction = direction_for_bidi_level(bidi_level);
    for token in cluster.parts {
        push_visual_fragment(
            fragments,
            VisualTextFragment {
                owner_run_index: token.owner_run_index,
                kind: token.kind,
                text: mirrored_visual_text(token.text, bidi_level),
                source_range: token.source_range,
                direction,
            },
        );
    }
}

fn mirrored_visual_text(text: String, bidi_level: u8) -> String {
    let mirrored = {
        let mut chars = text.chars();
        let Some(ch) = chars.next() else {
            return text;
        };
        if chars.next().is_some() {
            return text;
        }
        mirrored_bidi_char(ch, bidi_level)
    };
    mirrored.map(|ch| ch.to_string()).unwrap_or(text)
}

fn push_visual_fragment(fragments: &mut Vec<VisualTextFragment>, fragment: VisualTextFragment) {
    if let Some(last) = fragments.last_mut() {
        if last.owner_run_index == fragment.owner_run_index
            && last.kind == fragment.kind
            && last.direction == fragment.direction
            && last.source_range.end == fragment.source_range.start
        {
            last.text.push_str(&fragment.text);
            last.source_range.end = fragment.source_range.end;
            return;
        }
    }
    fragments.push(fragment);
}
