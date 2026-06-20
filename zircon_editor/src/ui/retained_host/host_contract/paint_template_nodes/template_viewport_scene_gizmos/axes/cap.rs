use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;

pub(super) fn push_axis_cap(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let cap_size = rect.height.max(rect.width.min(5.0)).max(3.0);
    let cap = if rect.width >= rect.height {
        FrameRect {
            x: rect.x + rect.width - cap_size,
            y: rect.y + (rect.height - cap_size) * 0.5,
            width: cap_size,
            height: cap_size,
        }
    } else {
        FrameRect {
            x: rect.x + (rect.width - cap_size) * 0.5,
            y: rect.y,
            width: cap_size,
            height: cap_size,
        }
    };
    commands.push(HostPaintCommand::quad(
        cap,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        2.0,
        opacity,
    ));
}
