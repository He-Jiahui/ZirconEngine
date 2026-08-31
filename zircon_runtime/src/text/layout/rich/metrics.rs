use std::sync::Arc;

use crate::core::math::Vec2;
use crate::text::{InlineBaseline, InlineObjectRef, TextStyle};

use super::finite_non_negative;

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

#[derive(Clone, Copy, Debug)]
pub(super) struct InlineBoxMetrics {
    pub(super) advance: f32,
    pub(super) size: Vec2,
    pub(super) ascent: f32,
    pub(super) descent: f32,
    pub(super) baseline: InlineBaseline,
}

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
