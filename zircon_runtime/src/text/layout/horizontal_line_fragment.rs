use std::sync::Arc;

use crate::text::shaping::TextShapingOutcome;
use crate::text::{ShapedGlyphRun, TextStyle};

use super::{
    MeasuredGlyphCluster, TextLineMetrics, measured_grapheme_geometry_from_shaped,
    resolve_horizontal_plain_line_policy, text_line_metrics_from_shaped,
};

/// Shared immutable geometry derived from one horizontal final-line shape request.
///
/// Physical and logical-virtual fragments differ in source/anchor semantics, but both require
/// identical metrics and grapheme advances from their retained shaped run. Composite baseline
/// policy and glyph-origin adjustment deliberately remain a later owner: this container does
/// not mutate cached glyph data or publish a new UI contract.
#[derive(Clone, Debug)]
pub(crate) struct HorizontalLineFragmentGeometry {
    shaped: Arc<ShapedGlyphRun>,
    metrics: TextLineMetrics,
    grapheme_advances: Vec<f32>,
    glyph_clusters: Vec<MeasuredGlyphCluster>,
}

impl HorizontalLineFragmentGeometry {
    pub(crate) fn from_shaped(
        shaped: Arc<ShapedGlyphRun>,
        text: &str,
        style: &TextStyle,
    ) -> TextShapingOutcome<Self> {
        let metrics = shaped
            .lines
            .first()
            .and_then(|line| {
                shaped
                    .horizontal_line_raw_metrics_at(0)
                    .zip(shaped.horizontal_glyph_metric_spans_for_line(0))
                    .and_then(|(raw_metrics, spans)| {
                        resolve_horizontal_plain_line_policy(style, line, raw_metrics, spans)
                    })
            })
            .map_or_else(
                || text_line_metrics_from_shaped(&shaped, style),
                |policy| policy.metrics,
            );
        let geometry = match measured_grapheme_geometry_from_shaped(&shaped, text) {
            Ok(geometry) => geometry,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        TextShapingOutcome::Ready(Self {
            metrics,
            grapheme_advances: geometry.advances,
            glyph_clusters: geometry.glyph_clusters,
            shaped,
        })
    }

    pub(crate) fn shaped(&self) -> &Arc<ShapedGlyphRun> {
        &self.shaped
    }

    pub(crate) const fn metrics(&self) -> TextLineMetrics {
        self.metrics
    }

    pub(crate) fn grapheme_advances(&self) -> &[f32] {
        &self.grapheme_advances
    }

    pub(crate) fn glyph_clusters(&self) -> &[MeasuredGlyphCluster] {
        &self.glyph_clusters
    }
}
