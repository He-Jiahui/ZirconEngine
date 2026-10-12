use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style::surface_color;

use super::super::primitives::{color_with_alpha_factor, push_expanded_layer, push_inset_layer};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_soft_shadow(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = surface_color(node);
    push_expanded_layer(
        commands,
        rect,
        clip,
        order,
        color_with_alpha_factor(color, 0.44),
        8.0,
        5.0,
        rect.height * 0.42,
        opacity,
    );
    push_inset_layer(
        commands,
        rect,
        clip,
        order + 1,
        color_with_alpha_factor(color, 0.68),
        6.0,
        7.0,
        rect.height * 0.36,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.22,
            y: rect.y + rect.height * 0.40,
            width: (rect.width * 0.56).max(1.0),
            height: (rect.height * 0.28).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(color_with_alpha_factor(color, 0.86)),
        None,
        0.0,
        (rect.height * 0.14).max(5.0),
        opacity,
    ));
}
