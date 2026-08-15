use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_theme::{
    current_host_metrics, HostControlMetrics,
};

use super::super::super::component_variant_contains;
use super::metrics::avatar_bounded_extent;

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
    if component_variant_contains(node, "square") {
        return 0.0;
    }
    let half_extent =
        avatar_bounded_extent(rect.width).min(avatar_bounded_extent(rect.height)) * 0.5;
    if component_variant_contains(node, "rounded") {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::retained_host::host_contract::paint_theme::METRICS;

    #[test]
    fn rounded_avatar_radius_tracks_host_control_density_and_frame_bounds() {
        let mut node = TemplatePaneNodeData::default();
        node.component_variant = "rounded".to_owned();
        let mut compact = METRICS;
        compact.radius_control = 3.0;
        let wide = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
        };
        let narrow = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 8.0,
        };

        assert_eq!(avatar_corner_radius_from_host(&node, &wide, compact), 3.0);
        assert_eq!(avatar_corner_radius_from_host(&node, &narrow, compact), 1.0);
    }
}
