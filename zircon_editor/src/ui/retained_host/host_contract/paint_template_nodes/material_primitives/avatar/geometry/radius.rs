use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};

use super::super::super::component_variant_contains;
use super::metrics::AVATAR_ROUNDED_RADIUS;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn avatar_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    if component_variant_contains(node, "square") {
        return 0.0;
    }
    if component_variant_contains(node, "rounded") {
        let configured = node
            .corner_radius
            .max(node.button_style.element.corner_radius)
            .max(0.0);
        return if configured > 0.0 {
            configured
        } else {
            AVATAR_ROUNDED_RADIUS
        };
    }
    rect.width.min(rect.height) * 0.5
}
