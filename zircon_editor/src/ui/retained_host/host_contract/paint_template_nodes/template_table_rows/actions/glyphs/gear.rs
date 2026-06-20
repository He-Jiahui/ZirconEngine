use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

pub(super) fn push_table_gear(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for segment in [
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 2.0,
            width: 6.0,
            height: 1.0,
        },
        FrameRect {
            x: rect.x + 4.0,
            y: rect.y + 11.0,
            width: 6.0,
            height: 1.0,
        },
        FrameRect {
            x: rect.x + 2.0,
            y: rect.y + 4.0,
            width: 1.0,
            height: 6.0,
        },
        FrameRect {
            x: rect.x + 11.0,
            y: rect.y + 4.0,
            width: 1.0,
            height: 6.0,
        },
        FrameRect {
            x: rect.x + 6.0,
            y: rect.y + 6.0,
            width: 2.0,
            height: 2.0,
        },
    ] {
        commands.push(HostPaintCommand::quad(
            segment,
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}
