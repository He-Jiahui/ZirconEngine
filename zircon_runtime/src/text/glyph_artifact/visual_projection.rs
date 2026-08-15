use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::surface::{
    UiResolvedTextLine, UiResolvedTextRun, UiTextDirection, UiTextRange,
};

use crate::core::framework::text::TextGlyph;

use super::source_slice;

#[derive(Clone, Copy)]
pub(super) struct VisualCluster {
    pub(super) source_range: UiTextRange,
    pub(super) visual_index: usize,
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
            .map(|(visual_index, _)| VisualCluster {
                source_range: source_graphemes
                    .get(visual_index)
                    .copied()
                    .unwrap_or(line.source_range),
                visual_index,
            })
            .collect();
    }
    let run_maps = line
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
            for run in run_maps[first_run..]
                .iter()
                .take_while(|run| run.visual_range.start < visual_range.end)
            {
                if let Some(range) = run.source_range_for_visual(visual_range) {
                    source_range = Some(merge_ranges(source_range, range));
                }
            }
            VisualCluster {
                source_range: source_range.unwrap_or(line.source_range),
                visual_index,
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
        }
    }

    fn source_range_for_visual(&self, visual_range: UiTextRange) -> Option<UiTextRange> {
        let index = self
            .visual_graphemes
            .partition_point(|range| range.end <= visual_range.start);
        self.visual_graphemes
            .get(index)
            .filter(|range| ranges_overlap(**range, visual_range))?;
        self.source_graphemes.get(index).copied()
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
