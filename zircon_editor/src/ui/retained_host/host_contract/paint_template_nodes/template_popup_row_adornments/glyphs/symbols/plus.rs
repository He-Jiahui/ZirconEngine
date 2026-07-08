use super::super::super::super::super::data::FrameRect;
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::segments::{popup_adornment_segment, push_segments};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_plus_adornment(
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
            popup_adornment_segment(6, 3, 2, 8),
            popup_adornment_segment(3, 6, 8, 2),
        ],
    );
}
