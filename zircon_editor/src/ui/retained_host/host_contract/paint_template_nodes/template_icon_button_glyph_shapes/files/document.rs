use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_file_icon(
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
            seg(80, 40, 116, 22),
            seg(60, 60, 22, 200),
            seg(240, 104, 22, 156),
            seg(80, 240, 160, 22),
            seg(188, 52, 22, 76),
            seg(196, 112, 60, 22),
            seg(120, 142, 88, 22),
            seg(153, 109, 22, 88),
        ],
    );
}
