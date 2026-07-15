use unicode_segmentation::UnicodeSegmentation;

use super::super::{
    UiResolvedTextLine, UiResolvedTextRun, UiTextCaret, UiTextCaretAffinity, UiTextDirection,
    UiTextRange,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiTextVisualBoundaryBias {
    LeadingCurrent,
    TrailingPrevious,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UiTextVisualSpan {
    pub visual_range: UiTextRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UiTextVisualSourceCluster {
    source_range: UiTextRange,
    visual_range: UiTextRange,
    direction: UiTextDirection,
    source_isomorphic: bool,
}

/// One resolved line's authoritative source-byte to visual-cluster map.
///
/// The map is built from the post-UAX#9 resolved runs, so runtime hit testing,
/// editable paint and IME geometry do not reconstruct direction or cluster
/// ownership independently.
pub struct UiTextLineSourceMap<'a> {
    line: &'a UiResolvedTextLine,
    clusters: Vec<UiTextVisualSourceCluster>,
}

impl<'a> UiTextLineSourceMap<'a> {
    pub fn new(line: &'a UiResolvedTextLine) -> Self {
        Self {
            line,
            clusters: visual_source_clusters(line),
        }
    }

    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    pub fn visual_offset_for_caret(&self, caret: &UiTextCaret) -> usize {
        if self.clusters.is_empty() {
            return self.line.visual_range.start;
        }

        let exact = match caret.affinity {
            UiTextCaretAffinity::Upstream => self
                .clusters
                .iter()
                .filter(|cluster| cluster.source_range.end == caret.offset)
                .max_by_key(|cluster| cluster.source_range.start)
                .map(|cluster| self.source_edge_visual_offset(cluster, false)),
            UiTextCaretAffinity::Downstream => self
                .clusters
                .iter()
                .filter(|cluster| cluster.source_range.start == caret.offset)
                .min_by_key(|cluster| cluster.source_range.end)
                .map(|cluster| self.source_edge_visual_offset(cluster, true)),
        };
        if let Some(offset) = exact {
            return offset;
        }

        if let Some(cluster) = self.clusters.iter().find(|cluster| {
            caret.offset > cluster.source_range.start && caret.offset < cluster.source_range.end
        }) {
            return match caret.affinity {
                UiTextCaretAffinity::Upstream => self.source_edge_visual_offset(cluster, true),
                UiTextCaretAffinity::Downstream => self.source_edge_visual_offset(cluster, false),
            };
        }

        if caret.offset <= self.line.source_range.start {
            return self
                .clusters
                .iter()
                .min_by_key(|cluster| cluster.source_range.start)
                .map(logical_start_visual_offset)
                .unwrap_or(self.line.visual_range.start);
        }
        self.clusters
            .iter()
            .max_by_key(|cluster| cluster.source_range.end)
            .map(logical_end_visual_offset)
            .unwrap_or(self.line.visual_range.end)
    }

    pub fn caret_for_visual_boundary(
        &self,
        visual_boundary: usize,
        bias: UiTextVisualBoundaryBias,
        fallback: usize,
    ) -> UiTextCaret {
        match bias {
            UiTextVisualBoundaryBias::LeadingCurrent => self
                .clusters
                .get(visual_boundary)
                .map(|cluster| UiTextCaret {
                    offset: leading_source_offset(cluster),
                    affinity: UiTextCaretAffinity::Downstream,
                })
                .unwrap_or(UiTextCaret {
                    offset: fallback,
                    affinity: UiTextCaretAffinity::Downstream,
                }),
            UiTextVisualBoundaryBias::TrailingPrevious => visual_boundary
                .checked_sub(1)
                .and_then(|index| self.clusters.get(index))
                .map(|cluster| UiTextCaret {
                    offset: trailing_source_offset(cluster),
                    affinity: UiTextCaretAffinity::Upstream,
                })
                .unwrap_or(UiTextCaret {
                    offset: fallback,
                    affinity: UiTextCaretAffinity::Upstream,
                }),
        }
    }

    pub fn visual_spans_for_source_range(&self, range: UiTextRange) -> Vec<UiTextVisualSpan> {
        let mut spans = Vec::<UiTextVisualSpan>::new();
        for cluster in self.clusters.iter().filter(|cluster| {
            range.start < cluster.source_range.end && cluster.source_range.start < range.end
        }) {
            if let Some(last) = spans.last_mut() {
                if last.visual_range.end == cluster.visual_range.start {
                    last.visual_range.end = cluster.visual_range.end;
                    continue;
                }
            }
            spans.push(UiTextVisualSpan {
                visual_range: cluster.visual_range,
            });
        }
        spans
    }

    pub fn advance_to_visual_offset(&self, visual_offset: usize) -> f32 {
        let visual_index = self
            .clusters
            .iter()
            .take_while(|cluster| cluster.visual_range.end <= visual_offset)
            .count();
        if self.line.glyph_advances.len() == self.clusters.len() {
            return self
                .line
                .glyph_advances
                .iter()
                .take(visual_index)
                .map(|advance| sanitized_advance(*advance))
                .sum();
        }

        let cluster_count = self.clusters.len().max(1) as f32;
        sanitized_advance(self.line.measured_width) * visual_index as f32 / cluster_count
    }

    fn source_edge_visual_offset(
        &self,
        cluster: &UiTextVisualSourceCluster,
        logical_start: bool,
    ) -> usize {
        if cluster.source_isomorphic {
            return if logical_start {
                logical_start_visual_offset(cluster)
            } else {
                logical_end_visual_offset(cluster)
            };
        }

        let matching = self
            .clusters
            .iter()
            .filter(|candidate| candidate.source_range == cluster.source_range);
        let visual_start = matching
            .clone()
            .map(|candidate| candidate.visual_range.start)
            .min()
            .unwrap_or(cluster.visual_range.start);
        let visual_end = matching
            .map(|candidate| candidate.visual_range.end)
            .max()
            .unwrap_or(cluster.visual_range.end);
        if matches!(cluster.direction, UiTextDirection::RightToLeft) == logical_start {
            visual_end
        } else {
            visual_start
        }
    }
}

fn visual_source_clusters(line: &UiResolvedTextLine) -> Vec<UiTextVisualSourceCluster> {
    let mut clusters = Vec::new();
    for run in &line.runs {
        append_run_clusters(&mut clusters, run);
    }
    clusters
}

fn append_run_clusters(clusters: &mut Vec<UiTextVisualSourceCluster>, run: &UiResolvedTextRun) {
    let source_len = run.source_range.end.saturating_sub(run.source_range.start);
    let isomorphic = source_len == run.text.len();
    for (start, grapheme) in run.text.grapheme_indices(true) {
        let end = start + grapheme.len();
        clusters.push(UiTextVisualSourceCluster {
            source_range: if isomorphic {
                UiTextRange {
                    start: run.source_range.start + start,
                    end: run.source_range.start + end,
                }
            } else {
                run.source_range
            },
            visual_range: UiTextRange {
                start: run.visual_range.start + start,
                end: run.visual_range.start + end,
            },
            direction: run.direction,
            source_isomorphic: isomorphic,
        });
    }
}

fn logical_start_visual_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.visual_range.end
    } else {
        cluster.visual_range.start
    }
}

fn logical_end_visual_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.visual_range.start
    } else {
        cluster.visual_range.end
    }
}

fn leading_source_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.end
    } else {
        cluster.source_range.start
    }
}

fn trailing_source_offset(cluster: &UiTextVisualSourceCluster) -> usize {
    if matches!(cluster.direction, UiTextDirection::RightToLeft) {
        cluster.source_range.start
    } else {
        cluster.source_range.end
    }
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}
