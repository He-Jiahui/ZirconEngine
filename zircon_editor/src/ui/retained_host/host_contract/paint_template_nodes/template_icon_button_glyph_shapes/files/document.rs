use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

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
            (4.0, 2.0, 5.8, 1.1),
            (3.0, 3.0, 1.1, 10.0),
            (12.0, 5.2, 1.1, 7.8),
            (4.0, 12.0, 8.0, 1.1),
            (9.4, 2.6, 1.1, 3.8),
            (9.8, 5.6, 3.0, 1.1),
            (6.0, 7.1, 4.4, 1.1),
            (7.65, 5.45, 1.1, 4.4),
        ],
    );
}
