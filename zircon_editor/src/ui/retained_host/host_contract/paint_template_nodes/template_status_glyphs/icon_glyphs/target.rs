use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::{centered_canvas_rect, local_rect};
use super::super::segments::push_segments;

pub(super) fn push_target_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        None,
        Some(color),
        1.0,
        rect.height * 0.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        centered_canvas_rect(rect, 4.0),
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        2.0,
        opacity,
    ));
    push_segments(
        commands,
        clip,
        order + 1,
        color,
        opacity,
        &[
            local_rect(rect, 7.0, 0.0, 2.0, 4.0),
            local_rect(rect, 7.0, 12.0, 2.0, 4.0),
            local_rect(rect, 0.0, 7.0, 4.0, 2.0),
            local_rect(rect, 12.0, 7.0, 4.0, 2.0),
        ],
    );
}
