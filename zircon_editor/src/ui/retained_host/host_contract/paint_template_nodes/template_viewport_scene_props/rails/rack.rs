use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::palette::{RACK_HORIZONTAL, RACK_VERTICAL};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_rack_detail(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let mut x = rect.x + 8.0;
    while x < rect.x + rect.width {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x,
                y: rect.y,
                width: 2.0,
                height: rect.height,
            },
            Some(clip.clone()),
            order,
            Some(RACK_VERTICAL),
            None,
            0.0,
            0.0,
            opacity,
        ));
        x += 28.0;
    }
    let mut y = rect.y + 3.0;
    while y < rect.y + rect.height {
        commands.push(HostPaintCommand::quad(
            FrameRect {
                x: rect.x,
                y,
                width: rect.width,
                height: 2.0,
            },
            Some(clip.clone()),
            order + 1,
            Some(RACK_HORIZONTAL),
            None,
            0.0,
            0.0,
            opacity,
        ));
        y += 42.0;
    }
}
