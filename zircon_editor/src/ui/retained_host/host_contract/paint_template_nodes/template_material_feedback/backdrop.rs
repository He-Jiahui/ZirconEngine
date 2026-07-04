use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::material_primitives::component_variant_contains;
use super::super::render_commands::HostPaintCommand;
use super::super::template_style_color::resolved_style_color;
use super::palette::material_feedback_palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_material_backdrop_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !node.popup_open
        && node.surface_variant.as_str() != "backdrop"
        && !component_variant_contains(node, "open")
    {
        return;
    }
    if component_variant_contains(node, "invisible") {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(
            resolved_style_color(node.button_style.element.background_color.as_ref())
                .unwrap_or_else(|| material_feedback_palette().backdrop_scrim),
        ),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_material_backdrop_node(
    node: &TemplatePaneNodeData,
) -> bool {
    node.component_role.as_str() == "backdrop"
        || node.role.as_str() == "Backdrop"
        || node.surface_variant.as_str() == "backdrop"
}
