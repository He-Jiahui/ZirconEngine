use super::super::super::{
    UiResolvedTextLine, UiTextCaret, UiTextCaretAffinity, UiTextDirection, UiTextRange,
};
use super::cluster::{
    leading_source_offset, logical_end_visual_offset, logical_start_visual_offset,
    trailing_source_offset, visual_source_clusters, UiTextVisualSourceCluster,
};
use super::{UiTextVisualBoundaryBias, UiTextVisualSpan};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    OnceLock,
};

#[cfg(test)]
#[path = "line/tests/performance_tests.rs"]
mod performance_tests;

const EXACT_ADVANCE_PREFIX_LINEAR_QUERY_LIMIT: u8 = 2;
const SOURCE_EDGE_LINEAR_QUERY_LIMIT: u8 = 2;

struct UiTextExactAdvanceCache {
    query_count: AtomicU8,
    prefixes: OnceLock<Vec<f32>>,
}

impl UiTextExactAdvanceCache {
    fn new() -> Self {
        Self {
            query_count: AtomicU8::new(0),
            prefixes: OnceLock::new(),
        }
    }
}

struct UiTextSourceEdgeCache {
    query_count: AtomicU8,
    bounds: OnceLock<HashMap<(usize, usize), (usize, usize)>>,
}

impl UiTextSourceEdgeCache {
    fn new() -> Self {
        Self {
            query_count: AtomicU8::new(0),
            bounds: OnceLock::new(),
        }
    }
}

/// One resolved line's authoritative source-byte to visual-cluster map.
///
/// The map is built from the post-UAX#9 resolved runs, so runtime hit testing,
/// editable paint and IME geometry do not reconstruct direction or cluster
/// ownership independently.
pub struct UiTextLineSourceMap<'a> {
    line: &'a UiResolvedTextLine,
    clusters: Vec<UiTextVisualSourceCluster>,
    source_ranges_are_disjoint_and_ordered: bool,
    source_edge_cache: Option<UiTextSourceEdgeCache>,
    exact_advance_cache: Option<UiTextExactAdvanceCache>,
}

impl<'a> UiTextLineSourceMap<'a> {
    pub fn new(line: &'a UiResolvedTextLine) -> Self {
        let clusters = visual_source_clusters(line);
        let source_ranges_are_disjoint_and_ordered = clusters
            .windows(2)
            .all(|pair| pair[0].source_range.end <= pair[1].source_range.start);
        let exact_advance_cache = (line.glyph_advances.len() == clusters.len()
            && line
                .glyph_advances
                .iter()
                .all(|advance| advance.is_finite() && *advance >= 0.0))
        .then(UiTextExactAdvanceCache::new);
        let source_edge_cache =
            (!source_ranges_are_disjoint_and_ordered).then(UiTextSourceEdgeCache::new);
        Self {
            line,
            clusters,
            source_ranges_are_disjoint_and_ordered,
            source_edge_cache,
            exact_advance_cache,
        }
    }

    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }

    pub fn visual_offset_for_caret(&self, caret: &UiTextCaret) -> usize {
        if self.clusters.is_empty() {
            return self.line.visual_range.start;
        }

        if self.source_ranges_are_disjoint_and_ordered {
            return self.visual_offset_for_ordered_caret(caret);
        }

        self.visual_offset_for_caret_linear(caret)
    }

    fn visual_offset_for_ordered_caret(&self, caret: &UiTextCaret) -> usize {
        let exact = match caret.affinity {
            UiTextCaretAffinity::Upstream => {
                let end = self
                    .clusters
                    .partition_point(|cluster| cluster.source_range.end <= caret.offset);
                end.checked_sub(1)
                    .and_then(|index| self.clusters.get(index))
                    .filter(|cluster| cluster.source_range.end == caret.offset)
                    .map(|cluster| unique_source_edge_visual_offset(cluster, false))
            }
            UiTextCaretAffinity::Downstream => {
                let start = self
                    .clusters
                    .partition_point(|cluster| cluster.source_range.start < caret.offset);
                self.clusters
                    .get(start)
                    .filter(|cluster| cluster.source_range.start == caret.offset)
                    .map(|cluster| unique_source_edge_visual_offset(cluster, true))
            }
        };
        if let Some(offset) = exact {
            return offset;
        }

        let end = self
            .clusters
            .partition_point(|cluster| cluster.source_range.start < caret.offset);
        if let Some(cluster) = end
            .checked_sub(1)
            .and_then(|index| self.clusters.get(index))
            .filter(|cluster| caret.offset < cluster.source_range.end)
        {
            return match caret.affinity {
                UiTextCaretAffinity::Upstream => unique_source_edge_visual_offset(cluster, true),
                UiTextCaretAffinity::Downstream => unique_source_edge_visual_offset(cluster, false),
            };
        }

        if caret.offset <= self.line.source_range.start {
            return self
                .clusters
                .first()
                .map(logical_start_visual_offset)
                .unwrap_or(self.line.visual_range.start);
        }
        self.clusters
            .last()
            .map(logical_end_visual_offset)
            .unwrap_or(self.line.visual_range.end)
    }

    fn visual_offset_for_caret_linear(&self, caret: &UiTextCaret) -> usize {
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
        if self.source_ranges_are_disjoint_and_ordered {
            return visual_spans_from_clusters(self.source_clusters_for_range(range).iter());
        }
        self.visual_spans_for_source_range_linear(range)
    }

    fn source_clusters_for_range(&self, range: UiTextRange) -> &[UiTextVisualSourceCluster] {
        let start = self
            .clusters
            .partition_point(|cluster| cluster.source_range.end <= range.start);
        let end = self
            .clusters
            .partition_point(|cluster| cluster.source_range.start < range.end);
        &self.clusters[start.min(end)..end]
    }

    fn visual_spans_for_source_range_linear(&self, range: UiTextRange) -> Vec<UiTextVisualSpan> {
        visual_spans_from_clusters(self.clusters.iter().filter(|cluster| {
            range.start < cluster.source_range.end && cluster.source_range.start < range.end
        }))
    }

    pub fn advance_to_visual_offset(&self, visual_offset: usize) -> f32 {
        if let Some(cache) = &self.exact_advance_cache {
            // Visual ranges follow the visual text's UTF-8 byte order, so the
            // completed cluster count is monotonic and can use binary search.
            let completed = self
                .clusters
                .partition_point(|cluster| cluster.visual_range.end <= visual_offset);
            if let Some(advance) = cache
                .prefixes
                .get()
                .and_then(|prefixes| prefixes.get(completed))
            {
                return *advance;
            }

            // Caret and IME geometry commonly construct a temporary map for one
            // lookup. Preserve their old zero-allocation, bounded scan before
            // paying for a prefix table shared by repeated selection/decor queries.
            if cache.query_count.fetch_add(1, Ordering::Relaxed)
                < EXACT_ADVANCE_PREFIX_LINEAR_QUERY_LIMIT
            {
                return self.exact_advance_to_visual_index(completed);
            }

            let prefixes = cache.prefixes.get_or_init(|| self.exact_advance_prefixes());
            if let Some(advance) = prefixes.get(completed) {
                return *advance;
            }
        }

        if visual_offset >= self.line.visual_range.end {
            sanitized_advance(self.line.measured_width)
        } else {
            0.0
        }
    }

    fn exact_advance_to_visual_index(&self, visual_index: usize) -> f32 {
        self.line
            .glyph_advances
            .iter()
            .take(visual_index)
            .map(|advance| sanitized_advance(*advance))
            .sum()
    }

    fn exact_advance_prefixes(&self) -> Vec<f32> {
        let mut prefixes = Vec::with_capacity(self.clusters.len().saturating_add(1));
        let mut total = 0.0;
        prefixes.push(total);
        for advance in &self.line.glyph_advances {
            total += sanitized_advance(*advance);
            prefixes.push(total);
        }
        prefixes
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

        let (visual_start, visual_end) = self.source_edge_visual_bounds(cluster);
        if matches!(cluster.direction, UiTextDirection::RightToLeft) == logical_start {
            visual_end
        } else {
            visual_start
        }
    }

    fn source_edge_visual_bounds(&self, cluster: &UiTextVisualSourceCluster) -> (usize, usize) {
        if let Some(cache) = &self.source_edge_cache {
            let key = (cluster.source_range.start, cluster.source_range.end);
            if let Some(bounds) = cache.bounds.get().and_then(|bounds| bounds.get(&key)) {
                return *bounds;
            }
            if cache.query_count.fetch_add(1, Ordering::Relaxed) >= SOURCE_EDGE_LINEAR_QUERY_LIMIT {
                if let Some(bounds) = cache
                    .bounds
                    .get_or_init(|| self.source_edge_bounds_by_range())
                    .get(&key)
                {
                    return *bounds;
                }
            }
        }
        self.source_edge_visual_bounds_linear(cluster)
    }

    fn source_edge_bounds_by_range(&self) -> HashMap<(usize, usize), (usize, usize)> {
        let mut bounds_by_range = HashMap::with_capacity(self.clusters.len());
        for cluster in self
            .clusters
            .iter()
            .filter(|cluster| !cluster.source_isomorphic)
        {
            let key = (cluster.source_range.start, cluster.source_range.end);
            bounds_by_range
                .entry(key)
                .and_modify(|bounds: &mut (usize, usize)| {
                    bounds.0 = bounds.0.min(cluster.visual_range.start);
                    bounds.1 = bounds.1.max(cluster.visual_range.end);
                })
                .or_insert((cluster.visual_range.start, cluster.visual_range.end));
        }
        bounds_by_range
    }

    fn source_edge_visual_bounds_linear(
        &self,
        cluster: &UiTextVisualSourceCluster,
    ) -> (usize, usize) {
        self.clusters
            .iter()
            .filter(|candidate| candidate.source_range == cluster.source_range)
            .fold(
                (cluster.visual_range.start, cluster.visual_range.end),
                |bounds, candidate| {
                    (
                        bounds.0.min(candidate.visual_range.start),
                        bounds.1.max(candidate.visual_range.end),
                    )
                },
            )
    }
}

fn visual_spans_from_clusters<'a>(
    clusters: impl Iterator<Item = &'a UiTextVisualSourceCluster>,
) -> Vec<UiTextVisualSpan> {
    let mut spans = Vec::<UiTextVisualSpan>::new();
    for cluster in clusters {
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

fn unique_source_edge_visual_offset(
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
    if matches!(cluster.direction, UiTextDirection::RightToLeft) == logical_start {
        cluster.visual_range.end
    } else {
        cluster.visual_range.start
    }
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}
