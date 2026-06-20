use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;
use super::super::template_style_color::resolved_style_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn matches_any_role(
    component_role: &str,
    role: &str,
    expected: &[&str],
) -> bool {
    expected
        .iter()
        .any(|candidate| *candidate == component_role || *candidate == role)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_quad(
    commands: &mut Vec<HostPaintCommand>,
    rect: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    border_width: f32,
    radius: f32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect,
        Some(clip.clone()),
        order,
        Some(color),
        (border_width > 0.0).then_some(PALETTE.focus_ring),
        border_width,
        radius,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn node_radius(
    node: &TemplatePaneNodeData,
) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn node_background(
    node: &TemplatePaneNodeData,
) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.background_color.as_ref())
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn component_variant_contains(
    node: &TemplatePaneNodeData,
    expected: &str,
) -> bool {
    node.component_variant
        .split_whitespace()
        .any(|part| part == expected)
}
