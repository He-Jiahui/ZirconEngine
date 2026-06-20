use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::material_primitives::component_variant_contains;

const MUI_PAPER_DEFAULT_RADIUS: f32 = 4.0;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn paper_corner_radius(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    if component_variant_contains(node, "square") {
        return 0.0;
    }
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    let radius = if configured > 0.0 {
        configured
    } else {
        MUI_PAPER_DEFAULT_RADIUS
    };
    radius.min(rect.width.min(rect.height) * 0.5)
}
