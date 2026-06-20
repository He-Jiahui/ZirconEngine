use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments;

pub(super) fn push_transform_icon(
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
            (6.0, 1.0, 2.0, 12.0),
            (1.0, 6.0, 12.0, 2.0),
            (3.0, 3.0, 2.0, 2.0),
            (9.0, 9.0, 2.0, 2.0),
        ],
    );
}
