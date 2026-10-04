use unicode_segmentation::UnicodeSegmentation;

use super::super::super::{UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange};

#[cfg(test)]
#[path = "cluster/tests/performance_tests.rs"]
mod performance_tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiTextVisualSourceCluster {
    pub(super) source_range: UiTextRange,
    pub(super) visual_range: UiTextRange,
    pub(super) direction: UiTextDirection,
    pub(super) source_isomorphic: bool,
}

/// 按视觉文本的 grapheme 建立 source 映射；跨样式 run 的组合字符仍聚合成一个 cluster。
pub(super) fn visual_source_clusters(line: &UiResolvedTextLine) -> Vec<UiTextVisualSourceCluster> {
    if !runs_are_visually_ordered(&line.runs) {
        return visual_source_clusters_linear(line);
    }

    // Layout normally provides one visual advance per grapheme, so reserve the
    // exact cluster count before a geometry or hit-test consumer walks it.
    let mut clusters = Vec::with_capacity(line.glyph_advances.len());
    let mut run_start = 0;
    for (start, grapheme) in line.text.grapheme_indices(true) {
        let visual_range = UiTextRange {
            start: line.visual_range.start + start,
            end: line.visual_range.start + start + grapheme.len(),
        };
        while line
            .runs
            .get(run_start)
            .is_some_and(|run| run.visual_range.end <= visual_range.start)
        {
            run_start += 1;
        }
        if let Some(cluster) =
            cluster_for_visual_grapheme_in_runs(&line.runs[run_start..], visual_range, true)
        {
            clusters.push(cluster);
        }
    }
    clusters
}

fn visual_source_clusters_linear(line: &UiResolvedTextLine) -> Vec<UiTextVisualSourceCluster> {
    let mut clusters = Vec::with_capacity(line.glyph_advances.len());
    for (start, grapheme) in line.text.grapheme_indices(true) {
        let visual_range = UiTextRange {
            start: line.visual_range.start + start,
            end: line.visual_range.start + start + grapheme.len(),
        };
        if let Some(cluster) = cluster_for_visual_grapheme_in_runs(&line.runs, visual_range, false)
        {
            clusters.push(cluster);
        }
    }
    clusters
}

fn runs_are_visually_ordered(runs: &[UiResolvedTextRun]) -> bool {
    runs.windows(2)
        .all(|pair| pair[0].visual_range.start <= pair[1].visual_range.start)
}

pub(super) fn logical_start_visual_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.visual_range.end
    } else {
        cluster.visual_range.start
    }
}

pub(super) fn logical_end_visual_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.visual_range.start
    } else {
        cluster.visual_range.end
    }
}

pub(super) fn leading_source_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.end
    } else {
        cluster.source_range.start
    }
}

pub(super) fn trailing_source_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.start
    } else {
        cluster.source_range.end
    }
}

fn cluster_for_visual_grapheme_in_runs<'a>(
    runs: &'a [UiResolvedTextRun],
    visual_range: UiTextRange,
    runs_are_ordered: bool,
) -> Option<UiTextVisualSourceCluster> {
    let mut source_start = usize::MAX;
    let mut source_end = 0;
    let mut direction = None;
    let mut only_overlapping_run = None;
    let mut has_multiple_overlapping_runs = false;
    for run in runs {
        if runs_are_ordered && run.visual_range.start >= visual_range.end {
            break;
        }
        if run.visual_range.start >= visual_range.end || visual_range.start >= run.visual_range.end
        {
            continue;
        }
        if only_overlapping_run.replace(run).is_some() {
            has_multiple_overlapping_runs = true;
        }
        source_start = source_start.min(run.source_range.start);
        source_end = source_end.max(run.source_range.end);
        direction.get_or_insert(run.direction);
    }
    let direction = direction?;
    if !has_multiple_overlapping_runs {
        if let Some(run) = only_overlapping_run {
            // Preserve per-grapheme source edges only for the bijective single-run case.
            // A grapheme spanning runs (such as a combining cluster) must remain aggregated.
            if let Some(source_range) =
                source_range_for_isomorphic_visual_grapheme(run, visual_range)
            {
                return Some(UiTextVisualSourceCluster {
                    source_range,
                    visual_range,
                    direction,
                    source_isomorphic: true,
                });
            }
        }
    }
    let source_range = UiTextRange {
        start: source_start,
        end: source_end,
    };
    Some(UiTextVisualSourceCluster {
        source_range,
        visual_range,
        direction,
        source_isomorphic: source_range.end.saturating_sub(source_range.start)
            == visual_range.end.saturating_sub(visual_range.start),
    })
}

fn source_range_for_isomorphic_visual_grapheme(
    run: &UiResolvedTextRun,
    visual_range: UiTextRange,
) -> Option<UiTextRange> {
    if visual_range.start < run.visual_range.start || visual_range.end > run.visual_range.end {
        return None;
    }
    let visual_length = run.visual_range.end.checked_sub(run.visual_range.start)?;
    let source_length = run.source_range.end.checked_sub(run.source_range.start)?;
    if visual_length != source_length {
        return None;
    }
    let visual_start = visual_range.start.checked_sub(run.visual_range.start)?;
    let visual_end = visual_range.end.checked_sub(run.visual_range.start)?;
    let (start, end) = if matches!(run.direction, UiTextDirection::RightToLeft) {
        (
            run.source_range.end.checked_sub(visual_end)?,
            run.source_range.end.checked_sub(visual_start)?,
        )
    } else {
        (
            run.source_range.start.checked_add(visual_start)?,
            run.source_range.start.checked_add(visual_end)?,
        )
    };
    Some(UiTextRange { start, end })
}
