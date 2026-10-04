use std::sync::Arc;

use crate::core::math::Vec2;
use crate::text::{InlineBaseline, InlineObjectRef, TextStyle};

use super::finite_non_negative;

/// 将富文本 run 的局部覆写投影到一次布局使用的中性样式。
/// 未覆写的字体与行距保留基样式，测量、整形和最终物化须使用同一投影以免宽度漂移。
pub(crate) fn resolve_rich_run_style(
    base: &TextStyle,
    override_style: &crate::text::StyleOverride,
) -> TextStyle {
    let mut style = base.clone();
    if let Some(weight) = override_style.weight {
        style.font_weight = TextStyle::normalized_font_weight(weight);
    }
    if let Some(italic) = override_style.italic {
        style.italic = italic;
    }
    if let Some(features) = override_style.features.as_ref() {
        style.features = Arc::from(features.as_slice());
    }
    if let Some(font_size) = override_style
        .font_size
        .filter(|size| size.is_finite() && *size > 0.0)
    {
        let line_height_scale = base.line_height / base.font_size.max(1.0);
        style.font_size = font_size;
        style.line_height = font_size * line_height_scale;
    }
    if let Some(family) = override_style
        .family
        .as_ref()
        .filter(|family| !family.is_empty())
    {
        style.font_family = Some(family.as_str().to_string());
    }
    style
}

/// 内联对象参与基线和折行时的几何约定；同一尺寸用于前期宽度估计和最终行物化。
#[derive(Clone, Copy, Debug)]
pub(super) struct InlineBoxMetrics {
    pub(super) advance: f32,
    pub(super) size: Vec2,
    pub(super) ascent: f32,
    pub(super) descent: f32,
    pub(super) baseline: InlineBaseline,
}

/// 把图片、图标或控件槽转为行内盒模型，并按文本 ascent/descent 计算基线占用。
/// 输入尺寸可能来自可编辑标记，先收敛非有限或负尺寸，再供 RichAdvanceIndex 与物化共用。
pub(super) fn inline_box_metrics(
    inline: &InlineObjectRef,
    text_ascent: f32,
    text_descent: f32,
) -> InlineBoxMetrics {
    let (size, baseline) = match inline {
        InlineObjectRef::Image { size, baseline, .. }
        | InlineObjectRef::Icon { size, baseline, .. } => (*size, *baseline),
        InlineObjectRef::Widget { size, .. } => (*size, InlineBaseline::Baseline),
    };
    let size = Vec2::new(finite_non_negative(size.x), finite_non_negative(size.y));
    let (ascent, descent) = match baseline {
        InlineBaseline::Baseline => (size.y, 0.0),
        InlineBaseline::Center => (size.y * 0.5, size.y * 0.5),
        InlineBaseline::Top => (text_ascent, (size.y - text_ascent).max(0.0)),
        InlineBaseline::Bottom => ((size.y - text_descent).max(0.0), text_descent),
    };
    InlineBoxMetrics {
        advance: size.x,
        size,
        ascent,
        descent,
        baseline,
    }
}

pub(super) fn inline_origin_y(metrics: InlineBoxMetrics, baseline: f32, line_height: f32) -> f32 {
    match metrics.baseline {
        InlineBaseline::Baseline => baseline - metrics.size.y,
        InlineBaseline::Center => (line_height - metrics.size.y) * 0.5,
        InlineBaseline::Top => 0.0,
        InlineBaseline::Bottom => line_height - metrics.size.y,
    }
}
