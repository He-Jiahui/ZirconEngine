use crate::core::framework::text::TextDirection;
use crate::text::shaping::TextShapeRunProvider;
use crate::text::TextStyle;
use unicode_segmentation::UnicodeSegmentation;

use super::line_break::corrected_index_advance_with_provider;
use super::measured_grapheme_widths_with_provider;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphemeAdvanceMetric {
    pub(crate) source_start: usize,
    pub(crate) source_end: usize,
    pub(crate) advance: f32,
    pub(crate) cross_extent: f32,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GraphemeAdvanceIndex {
    metrics: Vec<GraphemeAdvanceMetric>,
    prefix_advances: Vec<f32>,
}

impl GraphemeAdvanceIndex {
    pub(crate) fn measured_with_provider<P>(text: &str, style: &TextStyle, provider: &mut P) -> Self
    where
        P: TextShapeRunProvider + ?Sized,
    {
        let advances = measured_grapheme_widths_with_provider(text, style, provider);
        let cross_extent = finite_non_negative(style.font_size.max(1.0));
        let metrics = text
            .grapheme_indices(true)
            .enumerate()
            .map(|(index, (source_start, grapheme))| GraphemeAdvanceMetric {
                source_start,
                source_end: source_start + grapheme.len(),
                advance: advances
                    .get(index)
                    .copied()
                    .map_or(0.0, finite_non_negative),
                cross_extent,
            })
            .collect();
        Self::from_metrics(metrics)
    }

    pub(crate) fn from_metrics(metrics: Vec<GraphemeAdvanceMetric>) -> Self {
        let mut prefix_advances = Vec::with_capacity(metrics.len().saturating_add(1));
        prefix_advances.push(0.0);
        for metric in &metrics {
            let next = prefix_advances.last().copied().unwrap_or_default() + metric.advance;
            prefix_advances.push(next);
        }
        Self {
            metrics,
            prefix_advances,
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
    ) -> f32
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

fn finite_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::core::framework::text::TextDirection;
    use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
    use crate::text::{ShapedGlyphRun, TextRange};

    use super::*;

    #[test]
    fn measured_advance_index_shapes_the_complete_source_once() {
        let mut provider = CountingShapeRunProvider::default();

        let index = GraphemeAdvanceIndex::measured_with_provider(
            "abcd",
            &TextStyle::default(),
            &mut provider,
        );

        assert_eq!(provider.shape_calls, 1);
        assert_eq!(index.metrics_in_range(0, 4).len(), 4);
        assert!((index.advance(0, 2) + index.advance(2, 4) - index.advance(0, 4)).abs() < 0.01);
    }

    #[derive(Default)]
    struct CountingShapeRunProvider {
        direct: DirectTextShapeRunProvider,
        shape_calls: usize,
    }

    impl TextShapeRunProvider for CountingShapeRunProvider {
        fn shape_horizontal_line_with_kerning(
            &mut self,
            text: &str,
            style: &TextStyle,
            direction: TextDirection,
            source_range: TextRange,
            include_kerning: bool,
        ) -> Arc<ShapedGlyphRun> {
            self.shape_calls = self.shape_calls.saturating_add(1);
            self.direct.shape_horizontal_line_with_kerning(
                text,
                style,
                direction,
                source_range,
                include_kerning,
            )
        }
    }
}
