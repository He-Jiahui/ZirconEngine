use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::segments::push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_close_mark(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        rect,
        clip,
        order,
        color,
        opacity,
        &[
            (4.0, 4.0, 2.0, 2.0),
            (6.0, 6.0, 2.0, 2.0),
            (8.0, 8.0, 2.0, 2.0),
            (10.0, 10.0, 2.0, 2.0),
            (10.0, 4.0, 2.0, 2.0),
            (8.0, 6.0, 2.0, 2.0),
            (6.0, 8.0, 2.0, 2.0),
            (4.0, 10.0, 2.0, 2.0),
        ],
    );
}
