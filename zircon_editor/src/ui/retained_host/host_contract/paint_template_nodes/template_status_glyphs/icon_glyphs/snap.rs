use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::local_rect;
use super::super::segments::push_segments;

pub(super) fn push_snap_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 4.0, 3.0, 8.0),
            local_rect(rect, 10.0, 4.0, 3.0, 8.0),
            local_rect(rect, 3.0, 11.0, 10.0, 3.0),
            local_rect(rect, 4.0, 2.0, 2.0, 3.0),
            local_rect(rect, 10.0, 2.0, 2.0, 3.0),
        ],
    );
}
