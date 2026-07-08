use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_save_icon(
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
            seg(60, 50, 192, 22),
            seg(60, 60, 22, 200),
            seg(240, 60, 22, 200),
            seg(80, 240, 160, 22),
            seg(102, 76, 92, 52),
            seg(212, 76, 22, 64),
            seg(104, 172, 112, 22),
            seg(104, 204, 112, 22),
        ],
    );
}
