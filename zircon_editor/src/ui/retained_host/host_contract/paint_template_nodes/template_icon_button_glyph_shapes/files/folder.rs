use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

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
            (2.0, 5.0, 4.0, 1.2),
            (5.0, 4.0, 4.0, 1.2),
            (2.0, 6.0, 12.0, 1.2),
            (2.0, 7.0, 1.2, 5.0),
            (13.0, 7.0, 1.2, 5.0),
            (3.0, 12.0, 10.0, 1.2),
        ],
    );
}
