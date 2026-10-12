use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::cap::push_axis_cap;
use super::colors::{axis_color, axis_glow};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_axis_line(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = axis_color(node);
    let glow_rect = if rect.width >= rect.height {
        FrameRect {
            x: rect.x - 1.0,
            y: rect.y - 2.0,
            width: rect.width + 2.0,
            height: rect.height + 4.0,
        }
    } else {
        FrameRect {
            x: rect.x - 2.0,
            y: rect.y - 1.0,
            width: rect.width + 4.0,
            height: rect.height + 2.0,
        }
    };
    commands.push(HostPaintCommand::quad(
        glow_rect,
        Some(clip.clone()),
        order,
        Some(axis_glow(color)),
        None,
        0.0,
        4.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        2.0,
        opacity,
    ));
    push_axis_cap(commands, rect, clip, order + 2, color, opacity);
}
