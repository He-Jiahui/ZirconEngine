use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_code_icon(
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
            seg(60, 120, 40, 28),
            seg(80, 100, 28, 40),
            seg(80, 180, 28, 40),
            seg(220, 120, 40, 28),
            seg(212, 100, 28, 40),
            seg(212, 180, 28, 40),
            seg(144, 80, 24, 160),
        ],
    );
}
