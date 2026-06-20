use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::geometry::local_rect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_down_chevron(
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
            local_rect(rect, 3.0, 4.0, 2.0, 2.0),
            local_rect(rect, 5.0, 6.0, 2.0, 2.0),
            local_rect(rect, 7.0, 4.0, 2.0, 2.0),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[FrameRect],
) {
    for segment in segments {
        commands.push(HostPaintCommand::quad(
            segment.clone(),
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
