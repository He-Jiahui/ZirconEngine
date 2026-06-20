use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::segments;

pub(super) fn push_mesh_icon(
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
            (2.0, 2.0, 10.0, 2.0),
            (2.0, 6.0, 10.0, 2.0),
            (2.0, 10.0, 10.0, 2.0),
            (2.0, 2.0, 2.0, 10.0),
            (6.0, 2.0, 2.0, 10.0),
            (10.0, 2.0, 2.0, 10.0),
        ],
    );
}
