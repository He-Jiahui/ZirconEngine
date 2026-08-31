use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange,
};

use crate::core::framework::text::TextGlyph;

use super::source_slice;

/// Maps glyphs shaped from a secure display line back to its presentation-owned source clusters.
/// Each secure run is exactly one visual grapheme, so a glyph spanning two runs is invalid rather
/// than an opportunity to infer an original UTF-8 range from the bullet string.
pub(super) fn presentation_glyphs_for_line(
    line: &UiResolvedTextLine,
    mut glyphs: Vec<TextGlyph>,
) -> Option<Vec<TextGlyph>> {
    if line.runs.is_empty() || line.glyph_advances.len() != line.runs.len() {
        return None;
    }
    let mut expected_visual_start = line.visual_range.start;
    for run in &line.runs {
        if run.text.graphemes(true).count() != 1
            || run.visual_range.start != expected_visual_start
            || run.visual_range.end.saturating_sub(run.visual_range.start) != run.text.len()
            || run.source_range.start >= run.source_range.end
            || line.text.get(run.visual_range.start..run.visual_range.end)
                != Some(run.text.as_str())
        {
            return None;
        }
        expected_visual_start = run.visual_range.end;
    }
    if expected_visual_start != line.visual_range.end {
        return None;
    }

    // `shape_presentation_line` shapes this display text in physical LTR order. With the
    // validated contiguous visual runs above, a single cursor maps every glyph in O(G + R).
    // Treat a backend order violation as a failed secure projection instead of searching all
    // runs and potentially inferring a source range from an unrelated mask grapheme.
    let mut first_glyph_for_run = vec![None; line.runs.len()];
    let mut run_index = 0_usize;
    let mut previous_visual_range: Option<UiTextRange> = None;
    for (glyph_index, glyph) in glyphs.iter_mut().enumerate() {
        let visual_range = UiTextRange {
            start: glyph
                .source_range
                .start
                .checked_add(line.visual_range.start)?,
            end: glyph
                .source_range
                .end
                .checked_add(line.visual_range.start)?,
        };
        if visual_range.start >= visual_range.end
            || visual_range.end > line.visual_range.end
            || previous_visual_range.is_some_and(|previous| {
                visual_range.start < previous.start
                    || (visual_range.start == previous.start && visual_range.end != previous.end)
            })
        {
            return None;
        }
        while line
            .runs
            .get(run_index)
            .is_some_and(|run| run.visual_range.end <= visual_range.start)
        {
            run_index = run_index.saturating_add(1);
        }
        let run = line.runs.get(run_index)?;
        if visual_range.start < run.visual_range.start || visual_range.end > run.visual_range.end {
            return None;
        }
        glyph.source_range = run.source_range.start..run.source_range.end;
        glyph.flags.right_to_left = matches!(run.direction, UiTextDirection::RightToLeft);
        glyph.advance = 0.0;
        first_glyph_for_run[run_index].get_or_insert(glyph_index);
        previous_visual_range = Some(visual_range);
    }
    for (run_index, advance) in line.glyph_advances.iter().copied().enumerate() {
        let glyph_index = first_glyph_for_run[run_index]?;
        if advance.is_finite() {
            glyphs[glyph_index].advance += advance.max(0.0);
        }
    }
    Some(glyphs)
}

/// Maps glyphs shaped from a resolved line's already-physical visual text back to source ranges.
///
/// Unlike secure presentation, this path admits zero-width source runs for generated ellipses and
/// kashidas. A glyph may span ordinary source clusters, or virtual clusters anchored at one
/// source offset, but never both; that keeps selection and caret ownership unambiguous.
pub(super) fn visual_glyphs_for_visual_line(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
    mut glyphs: Vec<TextGlyph>,
) -> Option<Vec<TextGlyph>> {
    if line.visual_range.start != 0 || !visual_line_runs_are_contiguous(line) {
        return None;
    }
    let clusters = visual_clusters_for_line(source_text, source_text_origin, line);
    if clusters.is_empty() || clusters.len() != line.glyph_advances.len() {
        return None;
    }

    let mut first_glyph_for_cluster = vec![None; clusters.len()];
    let mut cluster_start = 0_usize;
    let mut cluster_end = 0_usize;
    let mut previous_visual_range: Option<UiTextRange> = None;
    for (glyph_index, glyph) in glyphs.iter_mut().enumerate() {
        let visual_range = UiTextRange {
            start: glyph.source_range.start,
            end: glyph.source_range.end,
        };
        if visual_range.start >= visual_range.end
            || visual_range.end > line.visual_range.end
            || previous_visual_range.is_some_and(|previous| {
                visual_range.start < previous.start
                    || (visual_range.start == previous.start && visual_range.end != previous.end)
            })
        {
            return None;
        }
        // The physical shaper order is monotonic, so these cursors keep the
        // projection linear in glyphs plus visual clusters.
        while clusters
            .get(cluster_start)
            .is_some_and(|cluster| cluster.visual_range.end <= visual_range.start)
        {
            cluster_start = cluster_start.saturating_add(1);
        }
        cluster_end = cluster_end.max(cluster_start);
        while clusters
            .get(cluster_end)
            .is_some_and(|cluster| cluster.visual_range.start < visual_range.end)
        {
            cluster_end = cluster_end.saturating_add(1);
        }
        if cluster_start == cluster_end
            || clusters
                .get(cluster_end.saturating_sub(1))
                .is_none_or(|cluster| cluster.visual_range.end < visual_range.end)
        {
            return None;
        }
        let glyph_clusters = &clusters[cluster_start..cluster_end];
        let virtual_glyph = glyph_clusters
            .iter()
            .all(|cluster| cluster.source_range.start == cluster.source_range.end);
        if (!virtual_glyph
            && glyph_clusters
                .iter()
                .any(|cluster| cluster.source_range.start == cluster.source_range.end))
            || (virtual_glyph
                && glyph_clusters
                    .iter()
                    .any(|cluster| cluster.source_range != glyph_clusters[0].source_range))
        {
            return None;
        }
        let direction = glyph_clusters[0].direction?;
        if glyph_clusters
            .iter()
            .any(|cluster| cluster.direction != Some(direction))
        {
            return None;
        }
        let source_range = glyph_clusters
            .iter()
            .copied()
            .fold(glyph_clusters[0].source_range, |range, cluster| {
                merge_ranges(Some(range), cluster.source_range)
            });
        glyph.source_range = source_range.start..source_range.end;
        glyph.flags.right_to_left = matches!(direction, UiTextDirection::RightToLeft);
        glyph.flags.virtual_glyph = virtual_glyph;
        glyph.advance = 0.0;
        for cluster in &clusters[cluster_start..cluster_end] {
            first_glyph_for_cluster[cluster.visual_index].get_or_insert(glyph_index);
        }
        previous_visual_range = Some(visual_range);
    }
    for (cluster_index, advance) in line.glyph_advances.iter().copied().enumerate() {
        let glyph_index = first_glyph_for_cluster[cluster_index]?;
        if advance.is_finite() {
            glyphs[glyph_index].advance += advance.max(0.0);
        }
    }
    Some(glyphs)
}

fn visual_line_runs_are_contiguous(line: &UiResolvedTextLine) -> bool {
    if line.runs.is_empty() {
        return false;
    }
    let mut expected_start = line.visual_range.start;
    for run in &line.runs {
        if run.text.is_empty()
            || run.visual_range.start != expected_start
            || run.visual_range.end.saturating_sub(run.visual_range.start) != run.text.len()
            || line.text.get(run.visual_range.start..run.visual_range.end)
                != Some(run.text.as_str())
        {
            return false;
        }
        expected_start = run.visual_range.end;
    }
    expected_start == line.visual_range.end
}

#[derive(Clone, Copy)]
pub(super) struct VisualCluster {
    pub(super) source_range: UiTextRange,
    pub(super) visual_range: UiTextRange,
    pub(super) visual_index: usize,
    direction: Option<UiTextDirection>,
}

pub(super) struct ProjectedGlyph {
    pub(super) glyph: TextGlyph,
    pub(super) source_index: usize,
    pub(super) visual_index: usize,
    pub(super) source_clusters: std::ops::Range<usize>,
}

pub(super) fn visual_clusters_for_line(
    source_text: &str,
    source_text_origin: usize,
    line: &UiResolvedTextLine,
) -> Vec<VisualCluster> {
    if line.runs.is_empty() {
        let mut source_graphemes = source_slice(source_text, source_text_origin, line.source_range)
            .map(|source| {
                source
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| UiTextRange {
                        start: line.source_range.start + start,
                        end: line.source_range.start + start + grapheme.len(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if matches!(line.direction, UiTextDirection::RightToLeft) {
            source_graphemes.reverse();
        }
        return line
            .text
            .grapheme_indices(true)
            .enumerate()
            .map(|(visual_index, (start, grapheme))| VisualCluster {
                source_range: source_graphemes
                    .get(visual_index)
                    .copied()
                    .unwrap_or(line.source_range),
                visual_range: UiTextRange {
                    start: line.visual_range.start + start,
                    end: line.visual_range.start + start + grapheme.len(),
                },
                visual_index,
                direction: Some(line.direction),
            })
            .collect();
    }
    let mut run_maps = line
        .runs
        .iter()
        .map(|run| RunSourceMap::new(source_text, source_text_origin, run))
        .collect::<Vec<_>>();
    let mut first_run = 0_usize;
    line.text
        .grapheme_indices(true)
        .enumerate()
        .map(|(visual_index, (start, grapheme))| {
            let visual_range = UiTextRange {
                start: line.visual_range.start + start,
                end: line.visual_range.start + start + grapheme.len(),
            };
            while run_maps
                .get(first_run)
                .is_some_and(|run| run.visual_range.end <= visual_range.start)
            {
                first_run += 1;
            }
            let mut source_range = None;
            let mut direction = None;
            let mut mixed_direction = false;
            for run in run_maps[first_run..]
                .iter_mut()
                .take_while(|run| run.visual_range.start < visual_range.end)
            {
                if let Some(range) = run.source_range_for_visual(visual_range) {
                    source_range = Some(merge_ranges(source_range, range));
                }
                if let Some(current) = direction {
                    mixed_direction |= current != run.direction;
                } else {
                    direction = Some(run.direction);
                }
            }
            VisualCluster {
                source_range: source_range.unwrap_or(line.source_range),
                visual_range,
                visual_index,
                direction: (!mixed_direction).then_some(direction).flatten(),
            }
        })
        .collect()
}

pub(super) fn source_cluster_range_for_glyph(
    source_order: &[VisualCluster],
    glyph: &TextGlyph,
) -> std::ops::Range<usize> {
    let start = source_order
        .partition_point(|cluster| cluster.source_range.end <= glyph.source_range.start);
    let end =
        source_order.partition_point(|cluster| cluster.source_range.start < glyph.source_range.end);
    start..end
}

pub(super) fn apply_resolved_advances(
    glyphs: &mut [ProjectedGlyph],
    source_order: &[VisualCluster],
    advances: &[f32],
    cluster_count: usize,
) {
    if advances.len() != cluster_count {
        return;
    }
    for glyph in glyphs
        .iter_mut()
        .filter(|glyph| !glyph.source_clusters.is_empty())
    {
        glyph.glyph.advance = 0.0;
    }
    let mut first_glyph_by_cluster = vec![None; cluster_count];
    for (glyph_index, glyph) in glyphs.iter().enumerate() {
        for cluster in &source_order[glyph.source_clusters.clone()] {
            first_glyph_by_cluster[cluster.visual_index].get_or_insert(glyph_index);
        }
    }
    for (cluster_index, advance) in advances.iter().copied().enumerate() {
        let Some(glyph_index) = first_glyph_by_cluster[cluster_index] else {
            continue;
        };
        if advance.is_finite() {
            glyphs[glyph_index].glyph.advance += advance.max(0.0);
        }
    }
}

struct RunSourceMap {
    visual_range: UiTextRange,
    visual_graphemes: Vec<UiTextRange>,
    source_graphemes: Vec<UiTextRange>,
    direction: UiTextDirection,
    visual_grapheme_cursor: usize,
}

impl RunSourceMap {
    fn new(source_text: &str, source_text_origin: usize, run: &UiResolvedTextRun) -> Self {
        let visual_graphemes = run
            .text
            .grapheme_indices(true)
            .map(|(start, grapheme)| UiTextRange {
                start: run.visual_range.start + start,
                end: run.visual_range.start + start + grapheme.len(),
            })
            .collect::<Vec<_>>();
        let mut source_graphemes = source_slice(source_text, source_text_origin, run.source_range)
            .map(|source| {
                source
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| UiTextRange {
                        start: run.source_range.start + start,
                        end: run.source_range.start + start + grapheme.len(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if source_graphemes.len() != visual_graphemes.len() {
            source_graphemes = vec![run.source_range; visual_graphemes.len()];
        } else if matches!(run.direction, UiTextDirection::RightToLeft) {
            source_graphemes.reverse();
        }
        Self {
            visual_range: run.visual_range,
            visual_graphemes,
            source_graphemes,
            direction: run.direction,
            visual_grapheme_cursor: 0,
        }
    }

    fn source_range_for_visual(&mut self, visual_range: UiTextRange) -> Option<UiTextRange> {
        while self
            .visual_graphemes
            .get(self.visual_grapheme_cursor)
            .is_some_and(|range| range.end <= visual_range.start)
        {
            self.visual_grapheme_cursor = self.visual_grapheme_cursor.saturating_add(1);
        }
        self.visual_graphemes
            .get(self.visual_grapheme_cursor)
            .filter(|range| ranges_overlap(**range, visual_range))?;
        self.source_graphemes
            .get(self.visual_grapheme_cursor)
            .copied()
    }
}

fn merge_ranges(current: Option<UiTextRange>, next: UiTextRange) -> UiTextRange {
    let Some(current) = current else {
        return next;
    };
    UiTextRange {
        start: current.start.min(next.start),
        end: current.end.max(next.end),
    }
}

fn ranges_overlap(left: UiTextRange, right: UiTextRange) -> bool {
    left.start < right.end && right.start < left.end
}
