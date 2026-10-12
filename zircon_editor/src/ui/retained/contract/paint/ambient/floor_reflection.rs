use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style::surface_color;

use super::super::primitives::{color_with_alpha_factor, push_expanded_layer};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_floor_reflection(
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
        color_with_alpha_factor(color, 0.30),
        16.0,
        2.0,
        rect.height * 0.44,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.30,
            y: rect.y + rect.height * 0.12,
            width: (rect.width * 0.40).max(1.0),
            height: (rect.height * 0.76).max(1.0),
        },
        Some(clip.clone()),
        order + 1,
        Some(color_with_alpha_factor(color, 0.76)),
        None,
        0.0,
        rect.height * 0.36,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.42,
            y: rect.y + rect.height * 0.06,
            width: (rect.width * 0.16).max(1.0),
            height: (rect.height * 0.88).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(color_with_alpha_factor(color, 0.94)),
        None,
        0.0,
        rect.height * 0.30,
        opacity,
    ));
}
