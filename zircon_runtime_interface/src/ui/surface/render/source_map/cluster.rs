//! 先对整行视觉文本分字素，再投影到相交 run；这里不重新整形或重排文本。
//! 返回的簇表拥有区间值，输入解析行只在构造期间被借用。
use unicode_segmentation::UnicodeSegmentation;

use super::super::super::{UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange};

#[cfg(test)]
#[path = "cluster/tests/perf.rs"]
mod performance_tests;

/// 一个视觉字素及其源区间投影；等长标记只描述字节跨度，不验证文本相同。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct UiTextVisualSourceCluster {
    // 单 run 等长时可缩到字素子区间，否则保存相交 run 的源包络。
    pub(super) source_range: UiTextRange,
    // 整行视觉文本中该字素的完整字节区间。
    pub(super) visual_range: UiTextRange,
    // 使用首个相交 run 的方向，不为跨 run 字素重新计算方向。
    pub(super) direction: UiTextDirection,
    pub(super) source_isomorphic: bool,
}

/// 按视觉文本的 grapheme 建立 source 映射；跨样式 run 的组合字符仍聚合成一个 cluster。
///
/// # 参数与返回
/// `line.text` 先整体分段；返回至少与一个 run 相交的簇，不逐 run 重新切字素。
/// run 的视觉起点有序时前移游标，否则扫描全部 run；不校验源与视觉文本一致性。
pub(super) fn visual_source_clusters(line: &UiResolvedTextLine) -> Vec<UiTextVisualSourceCluster> {
    if !runs_are_visually_ordered(&line.runs) {
        return visual_source_clusters_linear(line);
    }

    // 此容量来自度量项数，只是预留提示，此处尚未证明它与投影后簇数相等。
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

// 这里只要求视觉起点非递减，不代表源范围有序或 run 互不重叠。
fn runs_are_visually_ordered(runs: &[UiResolvedTextRun]) -> bool {
    runs.windows(2)
        .all(|pair| pair[0].visual_range.start <= pair[1].visual_range.start)
}

/// 返回逻辑起点对应的视觉字节边缘；RTL 使用视觉末端。
pub(super) fn logical_start_visual_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.visual_range.end
    } else {
        cluster.visual_range.start
    }
}

/// 返回逻辑终点对应的视觉字节边缘；RTL 使用视觉起端。
pub(super) fn logical_end_visual_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.visual_range.start
    } else {
        cluster.visual_range.end
    }
}

/// 视觉前缘对应的源字节端点，RTL 取源区间末端。
pub(super) fn leading_source_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.end
    } else {
        cluster.source_range.start
    }
}

/// 视觉后缘对应的源字节端点，RTL 取源区间起端。
pub(super) fn trailing_source_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.start
    } else {
        cluster.source_range.end
    }
}

// 方向取首个相交 run；无相交项时返回空，单 run 尝试等长子映射，多 run 保留源包络。
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

// 包含关系与等长跨度只准许 checked 算术捷径；不比较文本内容或验证 UTF-8 边界。
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
