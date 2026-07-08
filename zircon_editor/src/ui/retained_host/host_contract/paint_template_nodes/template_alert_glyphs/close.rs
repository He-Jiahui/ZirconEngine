use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::segments::{alert_segment as seg, push_segments};

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
            seg(4, 4, 2, 2),
            seg(6, 6, 2, 2),
            seg(8, 8, 2, 2),
            seg(10, 10, 2, 2),
            seg(10, 4, 2, 2),
            seg(8, 6, 2, 2),
            seg(6, 8, 2, 2),
            seg(4, 10, 2, 2),
        ],
    );
}
