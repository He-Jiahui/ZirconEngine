use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::{
    icon_button_segment as seg, push_icon_button_glyph_segments as push_segments,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_folder_icon(
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
            seg(40, 100, 76, 22),
            seg(100, 80, 88, 22),
            seg(176, 100, 104, 22),
            seg(40, 122, 22, 118),
            seg(260, 122, 22, 118),
            seg(60, 240, 200, 22),
            seg(60, 152, 200, 20),
        ],
    );
}
