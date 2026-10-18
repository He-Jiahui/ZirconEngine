//! 从已成形的 Plain 横排行及其度量侧表推导行框指标。
//!
//! 调用方先按同一行提取侧表；本模块只检查连续 glyph 范围与 ascent/descent 包络。
//! 无法采用策略时由调用方使用既有指标回退，几何投影另行处理；这里不写回行或 glyph。
//! 返回值拥有标量指标，输入借用只在本次调用内使用。

use crate::text::{HorizontalGlyphMetricSpan, HorizontalLineRawMetrics, ShapedHardLine, TextStyle};

use super::TextLineMetrics;

/// Plain 横排行采用的行框策略值。
///
/// # 生命周期
/// 只保存复制后的宽度、基线和行高；不持有输入行、字体或侧表的借用，也不调整 glyph 偏移。
///
/// Resolves the Plain horizontal line box from direct-shaping metric provenance.
///
/// Every fallback face in the current Plain path shares one alphabetic baseline. The artifact
/// renderers consume that baseline directly, then add each glyph's shaping offset. Therefore a
/// per-face block-top offset must not be copied into `TextGlyph::offset[1]`: doing so would apply
/// the ascent correction twice. Rich text and inline objects use separate block origins and join
/// a later policy adapter instead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct HorizontalPlainLinePolicy {
    /// 解析成功返回时，宽度沿用输入行，基线与行高由组合包络计算；三者均通过本模块的有限性检查。
    pub(crate) metrics: TextLineMetrics,
}

/// 在已有横排行及同一行度量侧表上尝试计算 Plain 行框。
///
/// # 调用时机
/// 由 `HorizontalLineFragmentGeometry::from_shaped` 提取首行侧表后调用；成功后仍由调用方独立投影几何。
///
/// # 参数
/// - `style`：提供请求的行高和字号；这里只检查计算后的请求行高，不验证全部样式字段。
/// - `line`：借用已成形的行，读取 glyph 数量和既有宽度，不改变其内容。
/// - `raw_metrics`：按值传入的组合 ascent、descent 和行间隙。
/// - `spans`：借用该行按 glyph 索引排列的非空侧表，需从零连续覆盖全部 glyph。
///
/// # 返回
/// `Some` 拥有推导的指标；包络不匹配或受检计算结果非有限时返回 `None`。
/// `None` 由调用方回退；有首行时沿用其既有指标，不在此重新测量。
///
/// # 约束与生命周期
/// 只检查范围及最大 ascent/descent 与组合值精确相等，不复核侧表的行号、字体身份或行间隙来源，
/// 也不验证全部 glyph 几何。调用方负责传入相互对应的侧表，输入借用不进入返回值。
/// 原始度量类型可由反序列化构造，不能假定所有实例都经构造器验证。
///
/// # 参考
/// `HorizontalLineFragmentGeometry::from_shaped` 负责失败回退；`spans_match_line_envelope` 负责局部包络检查。
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
    // 计算后的行高若高于内容高度，差值等分到上下留白。
    let baseline = (line_height - content_height).max(0.0) * 0.5 + raw_metrics.ascent();
    (line_height.is_finite() && baseline.is_finite()).then_some(HorizontalPlainLinePolicy {
        metrics: TextLineMetrics {
            width: line.measured_width,
            baseline,
            line_height,
        },
    })
}

/// 检查 glyph 索引区间能否为组合 ascent/descent 提供完整包络。
///
/// # 参数与返回
/// `line` 提供 glyph 总数，`raw_metrics` 提供待比较的组合值；行与 `spans` 仅在调用期间借用。
/// 空行、空侧表、范围断裂、重叠、越界或最大值不匹配返回 `None`，完全匹配返回 `Some(())`。
///
/// # 约束
/// 比较使用浮点精确相等；范围是 glyph 下标，而非源文本字节区间。
/// 此检查不读取侧表行号、字体身份或行间隙，不替代调用方的来源配对。
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
