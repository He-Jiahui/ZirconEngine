use super::super::super::data::TemplatePaneNodeData;
use super::super::template_style::is_mui_overlay_surface_node;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draws_surface(
    node: &TemplatePaneNodeData,
) -> bool {
    if is_frame_only_node(node) {
        return false;
    }
    matches!(node.role.as_str(), "Panel" | "Button" | "Mount")
        || is_mui_overlay_surface_node(node)
        || !node.surface_variant.is_empty()
        || !node.button_variant.is_empty()
        || node.button_style.element.background_color.is_some()
        || node.button_style.element.border_color.is_some()
        || node.button_style.element.border_width > 0.0
        || node.button_style.element.corner_radius > 0.0
        || node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.hovered
        || node.pressed
        || node.focused
        || node.state_layer_enabled
        || node.ripple_enabled
        || node.disabled
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_frame_only_node(
    node: &TemplatePaneNodeData,
) -> bool {
    node.surface_variant
        .split_whitespace()
        .any(|part| matches!(part, "frame_only" | "frame-only" | "frameOnly"))
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draws_border(
    node: &TemplatePaneNodeData,
) -> bool {
    node.button_style.element.border_width > 0.0
        || node.button_style.element.border_color.is_some()
        || node.border_width > 0.0
        || node.corner_radius > 0.0
        || node.selected
        || node.checked
        || node.focused
        || node.hovered
        || node.pressed
        || node.drop_hovered
        || node.active_drag_target
        || matches!(node.role.as_str(), "Button" | "Mount")
}
