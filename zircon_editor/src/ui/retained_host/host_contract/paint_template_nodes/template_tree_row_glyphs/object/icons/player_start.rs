use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::segments::{local_rect, push_segments};

pub(in super::super) fn push_player_start_icon(
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
            local_rect(rect, 6.0, 1.0, 2.0, 3.0),
            local_rect(rect, 3.0, 4.0, 8.0, 2.0),
            local_rect(rect, 2.0, 7.0, 4.0, 4.0),
            local_rect(rect, 8.0, 7.0, 4.0, 4.0),
            local_rect(rect, 5.0, 11.0, 4.0, 2.0),
        ],
    );
}
