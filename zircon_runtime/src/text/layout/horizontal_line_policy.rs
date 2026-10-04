use crate::text::{HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, ShapedHardLine, TextStyle};

use super::TextLineMetrics;

/// Resolves the Plain horizontal line box from direct-shaping metric provenance.
///
/// Every fallback face in the current Plain path shares one alphabetic baseline. The artifact
/// renderers consume that baseline directly, then add each glyph's shaping offset. Therefore a
/// per-face block-top offset must not be copied into `TextGlyph::offset[1]`: doing so would apply
/// the ascent correction twice. Rich text and inline objects use separate block origins and join
/// a later policy adapter instead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HorizontalPlainLinePolicy {
    pub(crate) metrics: TextLineMetrics,
}

pub(crate) fn resolve_horizontal_plain_line_policy(
    style: &TextStyle,
    line: &ShapedHardLine,
    raw_metrics: HorizontalLineRawMetrics,
    spans: &[HorizontalGlyphMetricSpan],
) -> Option<HorizontalPlainLinePolicy> {
    spans_match_line_envelope(line, raw_metrics, spans)?;
    let requested_line_height = style.line_height.max(style.font_size.max(1.0));
    if !requested_line_height.is_finite()
        || !line.measured_width.is_finite()
        || line.measured_width < 0.0
    {
        return None;
    }
    let content_height = raw_metrics.ascent() + raw_metrics.descent();
    let natural_line_height = content_height + raw_metrics.line_spacing_gap();
    if !content_height.is_finite() || !natural_line_height.is_finite() {
        return None;
    }
    let line_height = requested_line_height.max(natural_line_height);
    let baseline = (line_height - content_height).max(0.0) * 0.5 + raw_metrics.ascent();
    (line_height.is_finite() && baseline.is_finite()).then_some(HorizontalPlainLinePolicy {
        metrics: TextLineMetrics {
            width: line.measured_width,
            baseline,
            line_height,
        },
    })
}

fn spans_match_line_envelope(
    line: &ShapedHardLine,
    raw_metrics: HorizontalLineRawMetrics,
    spans: &[HorizontalGlyphMetricSpan],
) -> Option<()> {
    // 连续 spans 与完整行包络一致时才使用组合字体基线；缺少此证据时让调用层使用原行指标。
    if line.glyphs.is_empty() || spans.is_empty() {
        return None;
    }
    let mut expected_start = 0_usize;
    let mut max_ascent = 0.0_f32;
    let mut max_descent = 0.0_f32;
    for span in spans {
        if span.glyph_start != expected_start
            || span.glyph_start >= span.glyph_end
            || span.glyph_end > line.glyphs.len()
        {
            return None;
        }
        expected_start = span.glyph_end;
        max_ascent = max_ascent.max(span.metrics.ascent());
        max_descent = max_descent.max(span.metrics.descent());
    }
    (expected_start == line.glyphs.len()
        && max_ascent == raw_metrics.ascent()
        && max_descent == raw_metrics.descent())
    .then_some(())
}

#[cfg(test)]
#[path = "tests/horizontal_line_policy.rs"]
mod tests;
