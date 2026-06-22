use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_style::is_button_disabled;
use super::super::state::MATERIAL_STATE_LAYER_OPACITY_PRESS;
use super::geometry::{ripple_clip, ripple_radius, ripple_rect};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_ripple_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity_multiplier: f32,
) {
    if !node.ripple_enabled || is_button_disabled(node) || (!node.pressed && !node.enter_pressed) {
        return;
    }
    commands.push(HostPaintCommand::quad(
        ripple_rect(node, rect),
        ripple_clip(node, clip, rect),
        order,
        Some(color),
        None,
        0.0,
        ripple_radius(rect),
        MATERIAL_STATE_LAYER_OPACITY_PRESS * opacity_multiplier,
    ));
}
