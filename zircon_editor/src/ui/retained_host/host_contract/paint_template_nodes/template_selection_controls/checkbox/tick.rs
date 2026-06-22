use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;

pub(super) fn push_checkbox_tick(
    commands: &mut Vec<HostPaintCommand>,
    mark: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let color = PALETTE.shell_background;
    for tick in checkbox_tick_segments(mark) {
        commands.push(HostPaintCommand::quad(
            tick,
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

fn checkbox_tick_segments(mark: &FrameRect) -> [FrameRect; 3] {
    [
        FrameRect {
            x: mark.x + 3.0,
            y: mark.y + 7.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: mark.x + 5.0,
            y: mark.y + 9.0,
            width: 3.0,
            height: 3.0,
        },
        FrameRect {
            x: mark.x + 8.0,
            y: mark.y + 4.0,
            width: 3.0,
            height: 8.0,
        },
    ]
}
