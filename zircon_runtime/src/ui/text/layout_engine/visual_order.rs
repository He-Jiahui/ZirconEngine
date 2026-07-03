use zircon_runtime_interface::ui::surface::{
    UiResolvedTextRun, UiTextDirection, UiTextRange, UiTextRunKind,
};

use super::super::grapheme::{grapheme_indices, leading_grapheme_continuation_len};
use super::candidate_line::CandidateLine;
use super::direction::{is_ltr_char, is_rtl_char};
use super::range_mapping::source_subrange;

#[derive(Clone, Debug)]
struct VisualTextToken {
    kind: UiTextRunKind,
    text: String,
    source_range: UiTextRange,
    direction: Option<UiTextDirection>,
}

#[derive(Clone, Debug)]
struct VisualTextCluster {
    parts: Vec<VisualTextToken>,
    direction: Option<UiTextDirection>,
    neutral: bool,
}

#[derive(Clone, Debug)]
struct VisualTextFragment {
    kind: UiTextRunKind,
    text: String,
    source_range: UiTextRange,
    direction: UiTextDirection,
    neutral: bool,
}

// This is a low-fidelity BiDi scaffold: it preserves source/visual byte ranges and
// mirrors single-codepoint RTL punctuation while deferring full glyph shaping, UAX#9
// level resolution, and cluster handling to the text backends.
pub(super) fn apply_visual_order(line: &mut CandidateLine, base_direction: UiTextDirection) {
    if line.runs.is_empty() {
        return;
    }
    let visual_fragments = visual_text_fragments(&line.runs, base_direction);
    if visual_fragments.is_empty() {
        return;
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
}

fn visual_text_fragments(
    runs: &[UiResolvedTextRun],
    base_direction: UiTextDirection,
) -> Vec<VisualTextFragment> {
    let clusters = visual_text_clusters(runs);
    let has_rtl = clusters.iter().any(|cluster| {
        cluster
            .direction
            .is_some_and(|direction| matches!(direction, UiTextDirection::RightToLeft))
    });
    if !has_rtl {
        return runs
            .iter()
            .map(|run| VisualTextFragment {
                kind: run.kind,
                text: run.text.clone(),
                source_range: run.source_range,
                direction: run.direction,
                neutral: false,
            })
            .collect();
    }
    let clusters = assign_neutral_cluster_directions(clusters, base_direction);

    let mut spans = Vec::<Vec<VisualTextCluster>>::new();
    let mut current = Vec::<VisualTextCluster>::new();
    let mut current_direction = None;
    for cluster in clusters {
        let direction = cluster.direction.unwrap_or_else(|| {
            default_visual_direction(base_direction).unwrap_or(UiTextDirection::LeftToRight)
        });
        if current_direction.is_some_and(|current| current != direction) {
            spans.push(current);
            current = Vec::new();
        }
        current_direction = Some(direction);
        current.push(VisualTextCluster {
            direction: Some(direction),
            ..cluster
        });
    }
    if !current.is_empty() {
        spans.push(current);
    }
    if matches!(base_direction, UiTextDirection::RightToLeft) {
        spans.reverse();
    }

    let mut fragments = Vec::new();
    for mut span in spans {
        let span_direction = span
            .first()
            .and_then(|cluster| cluster.direction)
            .unwrap_or(UiTextDirection::LeftToRight);
        if matches!(span_direction, UiTextDirection::RightToLeft) {
            span.reverse();
            for cluster in span {
                push_visual_cluster(&mut fragments, cluster, UiTextDirection::RightToLeft);
            }
        } else {
            for cluster in span {
                push_visual_cluster(&mut fragments, cluster, UiTextDirection::LeftToRight);
            }
        }
    }
    fragments
}

fn assign_neutral_cluster_directions(
    mut clusters: Vec<VisualTextCluster>,
    base_direction: UiTextDirection,
) -> Vec<VisualTextCluster> {
    let fallback = default_visual_direction(base_direction).unwrap_or(UiTextDirection::LeftToRight);
    for index in 0..clusters.len() {
        if clusters[index].direction.is_some() {
            continue;
        }
        let previous = clusters[..index]
            .iter()
            .rev()
            .find_map(|cluster| cluster.direction);
        let next = clusters[index + 1..]
            .iter()
            .find_map(|cluster| cluster.direction);
        clusters[index].direction = Some(neutral_token_direction(previous, next, fallback));
    }
    clusters
}

fn neutral_token_direction(
    previous: Option<UiTextDirection>,
    next: Option<UiTextDirection>,
    fallback: UiTextDirection,
) -> UiTextDirection {
    match (previous, next) {
        (Some(previous), Some(next)) if previous == next => previous,
        // Keep LTR/RTL boundary separators on the LTR side, but let punctuation inside an
        // RTL phrase travel with the surrounding RTL span until a real shaper replaces this.
        (Some(UiTextDirection::LeftToRight), Some(UiTextDirection::RightToLeft))
        | (Some(UiTextDirection::RightToLeft), Some(UiTextDirection::LeftToRight)) => {
            UiTextDirection::LeftToRight
        }
        (Some(previous), Some(_)) => previous,
        (Some(previous), None) => previous,
        (None, Some(next)) => next,
        (None, None) => fallback,
    }
}

fn visual_text_clusters(runs: &[UiResolvedTextRun]) -> Vec<VisualTextCluster> {
    let mut clusters = Vec::new();
    let mut emitted_text = String::new();
    for run in runs {
        let mut consumed = 0;
        if !clusters.is_empty() {
            let continuation_len = leading_grapheme_continuation_len(&emitted_text, &run.text);
            if continuation_len > 0 {
                let token = visual_token(
                    run,
                    0,
                    continuation_len,
                    grapheme_direction(&run.text[..continuation_len]),
                );
                if let Some(cluster) = clusters.last_mut() {
                    push_visual_cluster_part(cluster, token);
                    emitted_text.push_str(&run.text[..continuation_len]);
                    consumed = continuation_len;
                }
            }
        }

        for (offset, grapheme) in grapheme_indices(&run.text[consumed..]) {
            let offset = consumed + offset;
            let direction = grapheme_direction(grapheme);
            clusters.push(VisualTextCluster {
                parts: vec![visual_token(
                    run,
                    offset,
                    offset + grapheme.len(),
                    direction,
                )],
                direction,
                neutral: direction.is_none(),
            });
            emitted_text.push_str(grapheme);
        }
    }
    clusters.retain(|cluster| !cluster.parts.is_empty());
    clusters
}

fn visual_token(
    run: &UiResolvedTextRun,
    start: usize,
    end: usize,
    direction: Option<UiTextDirection>,
) -> VisualTextToken {
    VisualTextToken {
        kind: run.kind,
        text: run.text[start..end].to_string(),
        source_range: source_subrange(run.source_range, run.text.len(), start, end),
        direction,
    }
}

fn push_visual_cluster_part(cluster: &mut VisualTextCluster, token: VisualTextToken) {
    if cluster.direction.is_none() {
        cluster.direction = token.direction;
    }
    if token.direction.is_some() {
        cluster.neutral = false;
    }
    cluster.parts.push(token);
}

fn push_visual_cluster(
    fragments: &mut Vec<VisualTextFragment>,
    cluster: VisualTextCluster,
    direction: UiTextDirection,
) {
    for token in cluster.parts {
        push_visual_fragment(
            fragments,
            VisualTextFragment {
                kind: token.kind,
                text: mirrored_visual_text(token.text, direction),
                source_range: token.source_range,
                direction,
                neutral: cluster.neutral,
            },
        );
    }
}

fn source_text_direction(ch: char) -> Option<UiTextDirection> {
    if is_rtl_char(ch) {
        Some(UiTextDirection::RightToLeft)
    } else if is_ltr_char(ch) {
        Some(UiTextDirection::LeftToRight)
    } else {
        None
    }
}

fn grapheme_direction(grapheme: &str) -> Option<UiTextDirection> {
    grapheme.chars().find_map(source_text_direction)
}

fn mirrored_visual_text(text: String, direction: UiTextDirection) -> String {
    if !matches!(direction, UiTextDirection::RightToLeft) {
        return text;
    }
    let mirrored = {
        let mut chars = text.chars();
        let Some(ch) = chars.next() else {
            return text;
        };
        if chars.next().is_some() {
            return text;
        }
        mirrored_bidi_char(ch)
    };
    mirrored.map(|ch| ch.to_string()).unwrap_or(text)
}

fn mirrored_bidi_char(ch: char) -> Option<char> {
    Some(match ch {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        '‹' => '›',
        '›' => '‹',
        '≤' => '≥',
        '≥' => '≤',
        '∈' => '∋',
        '∋' => '∈',
        '⊂' => '⊃',
        '⊃' => '⊂',
        '⊆' => '⊇',
        '⊇' => '⊆',
        '←' => '→',
        '→' => '←',
        _ => return None,
    })
}

fn push_visual_fragment(fragments: &mut Vec<VisualTextFragment>, fragment: VisualTextFragment) {
    if let Some(last) = fragments.last_mut() {
        if last.kind == fragment.kind
            && last.direction == fragment.direction
            && last.source_range.end == fragment.source_range.start
        {
            last.text.push_str(&fragment.text);
            last.source_range.end = fragment.source_range.end;
            last.neutral &= fragment.neutral;
            return;
        }
        if last.kind == fragment.kind
            && last.direction == fragment.direction
            && !last.neutral
            && !fragment.neutral
            && fragment.source_range.end == last.source_range.start
        {
            last.text.push_str(&fragment.text);
            last.source_range.start = fragment.source_range.start;
            return;
        }
    }
    fragments.push(fragment);
}

fn default_visual_direction(direction: UiTextDirection) -> Option<UiTextDirection> {
    match direction {
        UiTextDirection::LeftToRight => Some(UiTextDirection::LeftToRight),
        UiTextDirection::RightToLeft => Some(UiTextDirection::RightToLeft),
        _ => None,
    }
}
