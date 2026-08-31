use crate::core::framework::text::TextDirection;
use crate::text::shaping::{TextLayoutOutcome, TextShapeRunProvider};
use crate::text::{ShapedGlyphBreakSafety, TextStyle};
use unicode_segmentation::UnicodeSegmentation;

use super::line_break::corrected_index_advance_with_provider;
use super::{MeasuredClusterCaretPolicy, MeasuredGlyphCluster, measure_line_with_provider};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphemeAdvanceMetric {
    pub(crate) source_start: usize,
    pub(crate) source_end: usize,
    pub(crate) advance: f32,
    pub(crate) cross_extent: f32,
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BoundaryBreakSafetyCounts {
    pub(crate) safe: usize,
    pub(crate) requires_reshape: usize,
    pub(crate) unknown: usize,
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
impl BoundaryBreakSafetyCounts {
    fn record(&mut self, safety: ShapedGlyphBreakSafety) {
        let count = match safety {
            ShapedGlyphBreakSafety::Safe => &mut self.safe,
            ShapedGlyphBreakSafety::RequiresReshape => &mut self.requires_reshape,
            ShapedGlyphBreakSafety::Unknown => &mut self.unknown,
        };
        *count = count.saturating_add(1);
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GraphemeAdvanceIndex {
    metrics: Vec<GraphemeAdvanceMetric>,
    prefix_advances: Vec<f32>,
    glyph_clusters: Vec<MeasuredGlyphCluster>,
}

impl GraphemeAdvanceIndex {
    pub(crate) fn measured_with_provider<P>(
        text: &str,
        style: &TextStyle,
        provider: &mut P,
    ) -> TextLayoutOutcome<Self>
    where
        P: TextShapeRunProvider + ?Sized,
    {
        measure_line_with_provider(text, style, provider).map(|measured| {
            let cross_extent = finite_non_negative(style.font_size.max(1.0));
            let metrics = text
                .grapheme_indices(true)
                .enumerate()
                .map(|(index, (source_start, grapheme))| GraphemeAdvanceMetric {
                    source_start,
                    source_end: source_start + grapheme.len(),
                    advance: measured
                        .grapheme_advances
                        .get(index)
                        .copied()
                        .map_or(0.0, finite_non_negative),
                    cross_extent,
                })
                .collect();
            Self::from_metrics_and_clusters(metrics, measured.glyph_clusters)
        })
    }

    pub(crate) fn from_metrics(metrics: Vec<GraphemeAdvanceMetric>) -> Self {
        let glyph_clusters = metrics
            .iter()
            .map(|metric| MeasuredGlyphCluster {
                source_range: crate::text::TextRange {
                    start: metric.source_start,
                    end: metric.source_end,
                },
                advance: finite_non_negative(metric.advance),
                caret_policy: MeasuredClusterCaretPolicy::GraphemeBoundary,
                break_safety: ShapedGlyphBreakSafety::Unknown,
            })
            .collect();
        Self::from_metrics_and_clusters(metrics, glyph_clusters)
    }

    pub(super) fn from_metrics_and_clusters(
        metrics: Vec<GraphemeAdvanceMetric>,
        mut glyph_clusters: Vec<MeasuredGlyphCluster>,
    ) -> Self {
        glyph_clusters
            .sort_by_key(|cluster| (cluster.source_range.start, cluster.source_range.end));
        let mut prefix_advances = Vec::with_capacity(metrics.len().saturating_add(1));
        prefix_advances.push(0.0);
        for metric in &metrics {
            let next = prefix_advances.last().copied().unwrap_or_default() + metric.advance;
            prefix_advances.push(next);
        }
        Self {
            metrics,
            prefix_advances,
            glyph_clusters,
        }
    }

    pub(crate) fn metrics_in_range(&self, start: usize, end: usize) -> &[GraphemeAdvanceMetric] {
        let (first, after_last) = self.metric_range(start, end);
        self.metrics.get(first..after_last).unwrap_or_default()
    }

    pub(crate) fn advance(&self, start: usize, end: usize) -> f32 {
        let (first, after_last) = self.metric_range(start, end);
        let Some(first_prefix) = self.prefix_advances.get(first) else {
            return 0.0;
        };
        let Some(after_last_prefix) = self.prefix_advances.get(after_last) else {
            return 0.0;
        };
        finite_non_negative(after_last_prefix - first_prefix)
    }

    pub(crate) fn corrected_advance_with_provider<P>(
        &self,
        text: &str,
        start: usize,
        end: usize,
        style: &TextStyle,
        direction: TextDirection,
        break_suffix: Option<&str>,
        provider: &mut P,
    ) -> TextLayoutOutcome<f32>
    where
        P: TextShapeRunProvider + ?Sized,
    {
        corrected_index_advance_with_provider(
            text,
            self,
            start,
            end,
            style,
            direction,
            break_suffix,
            provider,
        )
    }

    pub(crate) fn advances_and_max_cross(
        &self,
        start: usize,
        end: usize,
        minimum_cross_extent: f32,
    ) -> (Vec<f32>, f32) {
        let mut cross_extent = finite_non_negative(minimum_cross_extent);
        let advances = self
            .metrics_in_range(start, end)
            .iter()
            .map(|metric| {
                cross_extent = cross_extent.max(metric.cross_extent);
                metric.advance
            })
            .collect();
        (advances, cross_extent)
    }

    /// Returns both physical edges of an LTR atomic cluster containing `source_offset`.
    ///
    /// Legal source boundaries return `None`; callers may keep their ordinary grapheme geometry.
    /// This fallback is intentionally LTR-only. BiDi and vertical consumers require the canonical
    /// resolved glyph artifact because source order alone cannot recover physical cluster order.
    pub(crate) fn ltr_atomic_caret_span(&self, source_offset: usize) -> Option<(f32, f32)> {
        let cluster = self.atomic_cluster_containing_offset(source_offset)?;
        Some(self.cluster_advance_span(cluster.source_range))
    }

    /// Resolves a physical LTR advance to a legal source cluster edge.
    ///
    /// Atomic backend clusters replace the selected grapheme range. The boolean is true for the
    /// leading half and false for the trailing half. Exact trailing edges belong to the following
    /// legal cluster when one exists.
    pub(crate) fn ltr_caret_hit(
        &self,
        visual_advance: f32,
    ) -> Option<(crate::text::TextRange, bool)> {
        let visual_advance = finite_non_negative(visual_advance);
        let first_metric = self.metrics.first()?;
        let last_metric = self.metrics.last()?;
        if visual_advance <= 0.0 {
            return Some((
                crate::text::TextRange {
                    start: first_metric.source_start,
                    end: first_metric.source_end,
                },
                true,
            ));
        }
        if visual_advance >= self.prefix_advances.last().copied().unwrap_or_default() {
            return Some((
                crate::text::TextRange {
                    start: last_metric.source_start,
                    end: last_metric.source_end,
                },
                false,
            ));
        }
        let metric_index = self
            .prefix_advances
            .partition_point(|prefix| *prefix <= visual_advance)
            .saturating_sub(1)
            .min(self.metrics.len().saturating_sub(1));
        let metric = self.metrics.get(metric_index)?;
        let source_range = self
            .atomic_cluster_covering_range(metric.source_start, metric.source_end)
            .map(|cluster| cluster.source_range)
            .unwrap_or(crate::text::TextRange {
                start: metric.source_start,
                end: metric.source_end,
            });
        let (leading, trailing) = self.cluster_advance_span(source_range);
        Some((
            source_range,
            visual_advance <= leading + (trailing - leading) * 0.5,
        ))
    }

    pub(crate) fn grapheme_boundary_index(&self, source_offset: usize) -> usize {
        self.metrics
            .partition_point(|metric| metric.source_end <= source_offset)
    }

    /// Expands a non-empty source selection to every overlapping atomic cluster.
    pub(crate) fn coalesce_atomic_source_range(
        &self,
        mut range: crate::text::TextRange,
    ) -> crate::text::TextRange {
        if range.start >= range.end {
            return range;
        }
        for cluster in self.atomic_clusters() {
            if cluster.source_range.start >= range.end {
                break;
            }
            if range.start < cluster.source_range.end && cluster.source_range.start < range.end {
                range.start = range.start.min(cluster.source_range.start);
                range.end = range.end.max(cluster.source_range.end);
            }
        }
        range
    }

    /// Returns the shaping receipt for an exact candidate line boundary.
    ///
    /// Document endpoints are semantic boundaries. An offset inside an atomic backend cluster is
    /// never safe even when no cluster starts at that grapheme boundary. Missing provenance stays
    /// `Unknown` so the final-line owner can select an exact fallback instead of guessing.
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    pub(crate) fn break_safety_at_boundary(&self, source_offset: usize) -> ShapedGlyphBreakSafety {
        let Some(first_metric) = self.metrics.first() else {
            return ShapedGlyphBreakSafety::Unknown;
        };
        let Some(last_metric) = self.metrics.last() else {
            return ShapedGlyphBreakSafety::Unknown;
        };
        if source_offset == first_metric.source_start || source_offset == last_metric.source_end {
            return ShapedGlyphBreakSafety::Safe;
        }
        if source_offset < first_metric.source_start || source_offset > last_metric.source_end {
            return ShapedGlyphBreakSafety::Unknown;
        }

        let first_at_or_after = self
            .glyph_clusters
            .partition_point(|cluster| cluster.source_range.start < source_offset);
        if first_at_or_after > 0
            && self.glyph_clusters[first_at_or_after - 1]
                .source_range
                .start
                < source_offset
            && source_offset < self.glyph_clusters[first_at_or_after - 1].source_range.end
        {
            return ShapedGlyphBreakSafety::RequiresReshape;
        }

        let mut resolved = None;
        for cluster in self.glyph_clusters[first_at_or_after..]
            .iter()
            .take_while(|cluster| cluster.source_range.start == source_offset)
        {
            resolved = Some(most_conservative_break_safety(
                resolved,
                cluster.break_safety,
            ));
        }
        resolved.unwrap_or(ShapedGlyphBreakSafety::Unknown)
    }

    /// Aggregates an ordered candidate-boundary stream in O(boundaries + clusters).
    #[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
    pub(crate) fn break_safety_counts_at_monotonic_boundaries<I>(
        &self,
        source_offsets: I,
    ) -> BoundaryBreakSafetyCounts
    where
        I: IntoIterator<Item = usize>,
    {
        let mut counts = BoundaryBreakSafetyCounts::default();
        let Some(first_metric) = self.metrics.first() else {
            for _ in source_offsets {
                counts.record(ShapedGlyphBreakSafety::Unknown);
            }
            return counts;
        };
        let Some(last_metric) = self.metrics.last() else {
            return counts;
        };
        let document_start = first_metric.source_start;
        let document_end = last_metric.source_end;
        let mut cluster_index = 0_usize;
        let mut last_source_offset = None;

        for source_offset in source_offsets {
            if last_source_offset.is_some_and(|last| source_offset < last) {
                counts.record(ShapedGlyphBreakSafety::Unknown);
                continue;
            }
            last_source_offset = Some(source_offset);
            if source_offset == document_start || source_offset == document_end {
                counts.record(ShapedGlyphBreakSafety::Safe);
                continue;
            }
            if source_offset < document_start || source_offset > document_end {
                counts.record(ShapedGlyphBreakSafety::Unknown);
                continue;
            }

            while self
                .glyph_clusters
                .get(cluster_index)
                .is_some_and(|cluster| cluster.source_range.end <= source_offset)
            {
                cluster_index = cluster_index.saturating_add(1);
            }
            let Some(cluster) = self.glyph_clusters.get(cluster_index) else {
                counts.record(ShapedGlyphBreakSafety::Unknown);
                continue;
            };
            if cluster.source_range.start < source_offset {
                counts.record(ShapedGlyphBreakSafety::RequiresReshape);
                continue;
            }
            if cluster.source_range.start > source_offset {
                counts.record(ShapedGlyphBreakSafety::Unknown);
                continue;
            }

            let safety = self.glyph_clusters[cluster_index..]
                .iter()
                .take_while(|cluster| cluster.source_range.start == source_offset)
                .fold(None, |resolved, cluster| {
                    Some(most_conservative_break_safety(
                        resolved,
                        cluster.break_safety,
                    ))
                })
                .unwrap_or(ShapedGlyphBreakSafety::Unknown);
            counts.record(safety);
        }
        counts
    }

    /// Moves tentative glyph-wrap boundaries to the end of an atomic backend cluster.
    ///
    /// The final range may exceed its width limit; splitting a ligature without a font caret or a
    /// final-line reshape would be a stronger correctness violation than bounded overhang.
    pub(crate) fn coalesce_atomic_source_ranges(
        &self,
        ranges: Vec<(usize, usize)>,
    ) -> Vec<(usize, usize)> {
        let Some(first) = ranges.first() else {
            return ranges;
        };
        let final_end = ranges.last().map(|range| range.1).unwrap_or(first.1);
        let mut start = first.0;
        let mut coalesced = Vec::with_capacity(ranges.len());
        let mut cluster_index = 0_usize;
        for (_, tentative_end) in ranges {
            while self
                .glyph_clusters
                .get(cluster_index)
                .is_some_and(|cluster| cluster.source_range.end <= tentative_end)
            {
                cluster_index = cluster_index.saturating_add(1);
            }
            let end = self
                .glyph_clusters
                .get(cluster_index)
                .filter(|cluster| {
                    matches!(
                        cluster.caret_policy,
                        MeasuredClusterCaretPolicy::AtomicCluster
                    ) && cluster.source_range.start < tentative_end
                        && tentative_end < cluster.source_range.end
                })
                .map_or(tentative_end, |cluster| cluster.source_range.end)
                .min(final_end);
            while self
                .glyph_clusters
                .get(cluster_index)
                .is_some_and(|cluster| cluster.source_range.end <= end)
            {
                cluster_index = cluster_index.saturating_add(1);
            }
            if end > start {
                coalesced.push((start, end));
                start = end;
            }
        }
        if start < final_end {
            coalesced.push((start, final_end));
        }
        coalesced
    }

    fn atomic_clusters(&self) -> impl Iterator<Item = &MeasuredGlyphCluster> {
        self.glyph_clusters.iter().filter(|cluster| {
            matches!(
                cluster.caret_policy,
                MeasuredClusterCaretPolicy::AtomicCluster
            )
        })
    }

    fn atomic_cluster_containing_offset(
        &self,
        source_offset: usize,
    ) -> Option<&MeasuredGlyphCluster> {
        let index = self
            .glyph_clusters
            .partition_point(|cluster| cluster.source_range.end <= source_offset);
        self.glyph_clusters.get(index).filter(|cluster| {
            matches!(
                cluster.caret_policy,
                MeasuredClusterCaretPolicy::AtomicCluster
            ) && cluster.source_range.start < source_offset
                && source_offset < cluster.source_range.end
        })
    }

    fn atomic_cluster_covering_range(
        &self,
        start: usize,
        end: usize,
    ) -> Option<&MeasuredGlyphCluster> {
        let index = self
            .glyph_clusters
            .partition_point(|cluster| cluster.source_range.end <= start);
        self.glyph_clusters.get(index).filter(|cluster| {
            matches!(
                cluster.caret_policy,
                MeasuredClusterCaretPolicy::AtomicCluster
            ) && cluster.source_range.start <= start
                && end <= cluster.source_range.end
        })
    }

    fn cluster_advance_span(&self, range: crate::text::TextRange) -> (f32, f32) {
        let document_start = self
            .metrics
            .first()
            .map_or(range.start, |metric| metric.source_start);
        (
            self.advance(document_start, range.start),
            self.advance(document_start, range.end),
        )
    }

    fn metric_range(&self, start: usize, end: usize) -> (usize, usize) {
        if start >= end {
            let index = self
                .metrics
                .partition_point(|metric| metric.source_end <= start);
            return (index, index);
        }
        let first = self
            .metrics
            .partition_point(|metric| metric.source_end <= start);
        let after_last = self
            .metrics
            .partition_point(|metric| metric.source_start < end);
        (first.min(after_last), after_last)
    }
}

#[cfg(any(test, feature = "profiling", feature = "profiling-tracy"))]
const fn most_conservative_break_safety(
    current: Option<ShapedGlyphBreakSafety>,
    next: ShapedGlyphBreakSafety,
) -> ShapedGlyphBreakSafety {
    match (current, next) {
        (_, ShapedGlyphBreakSafety::RequiresReshape)
        | (Some(ShapedGlyphBreakSafety::RequiresReshape), _) => {
            ShapedGlyphBreakSafety::RequiresReshape
        }
        (_, ShapedGlyphBreakSafety::Unknown) | (Some(ShapedGlyphBreakSafety::Unknown), _) => {
            ShapedGlyphBreakSafety::Unknown
        }
        _ => ShapedGlyphBreakSafety::Safe,
    }
}

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests;
