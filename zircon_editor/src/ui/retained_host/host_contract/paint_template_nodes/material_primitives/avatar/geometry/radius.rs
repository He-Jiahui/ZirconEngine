use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

use super::metrics::avatar_bounded_extent;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
// variant 将形状约束为圆形、主题圆角或方形；图像遮罩和根框边框须采用同一结果。
enum AvatarShapeVariant {
    #[default]
    Circular,
    Rounded,
    Square,
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    avatar_corner_radius_from_host(node, rect, current_host_metrics())
}

fn avatar_corner_radius_from_host(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    metrics: HostControlMetrics,
) -> f32 {
    let shape = avatar_shape_variant(&node.component_variant);
    if shape == AvatarShapeVariant::Square {
        return 0.0;
    }
    let half_extent =
        avatar_bounded_extent(rect.width).min(avatar_bounded_extent(rect.height)) * 0.5;
    if shape == AvatarShapeVariant::Rounded {
        let configured = node
            .corner_radius
            .max(node.button_style.element.corner_radius)
            .max(0.0);
        let radius = if configured.is_finite() && configured > 0.0 {
            configured
        } else {
            avatar_bounded_extent(metrics.radius_control)
        };
        return radius.min(half_extent);
    }
    half_extent
}

fn avatar_shape_variant(component_variant: &str) -> AvatarShapeVariant {
    let mut shape = AvatarShapeVariant::Circular;
    for part in component_variant.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    }) {
        if part.eq_ignore_ascii_case("square") {
            return AvatarShapeVariant::Square;
        }
        if part.eq_ignore_ascii_case("rounded") {
            shape = AvatarShapeVariant::Rounded;
        }
    }
    shape
}

#[cfg(test)]
#[path = "radius/tests/single_scan_variant_tests.rs"]
mod single_scan_variant_tests;

#[cfg(test)]
#[path = "tests/radius.rs"]
mod tests;
