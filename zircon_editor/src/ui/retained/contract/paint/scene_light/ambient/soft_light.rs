use crate::ui::retained_host::host_contract::data::{FrameRect, TemplatePaneNodeData};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style::surface_color;

use super::super::primitives::{color_with_alpha_factor, push_expanded_layer, push_inset_layer};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_soft_light(
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
        color_with_alpha_factor(color, 0.34),
        10.0,
        6.0,
        rect.height * 0.48,
        opacity,
    );
    push_inset_layer(
        commands,
        rect,
        clip,
        order + 1,
        color_with_alpha_factor(color, 0.58),
        8.0,
        9.0,
        rect.height * 0.42,
        opacity,
    );
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + rect.width * 0.18,
            y: rect.y + rect.height * 0.36,
            width: (rect.width * 0.64).max(1.0),
            height: (rect.height * 0.24).max(1.0),
        },
        Some(clip.clone()),
        order + 2,
        Some(color_with_alpha_factor(color, 0.82)),
        None,
        0.0,
        (rect.height * 0.16).max(6.0),
        opacity,
    ));
}
