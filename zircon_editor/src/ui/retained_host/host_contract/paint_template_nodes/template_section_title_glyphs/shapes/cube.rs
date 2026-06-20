use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments;

pub(super) fn push_cube_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    segments::push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (5.0, 1.0, 4.0, 2.0),
            (3.0, 3.0, 2.0, 7.0),
            (9.0, 3.0, 2.0, 7.0),
            (5.0, 11.0, 4.0, 2.0),
            (1.0, 5.0, 2.0, 4.0),
            (11.0, 5.0, 2.0, 4.0),
        ],
    );
}
