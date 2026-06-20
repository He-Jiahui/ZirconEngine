use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_cursor_icon(
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
            (3.0, 2.0, 1.5, 10.0),
            (4.5, 4.0, 2.0, 1.4),
            (6.0, 6.0, 2.0, 1.4),
            (7.5, 8.0, 2.0, 1.4),
            (8.0, 10.0, 1.4, 3.0),
            (9.5, 12.0, 2.0, 1.4),
        ],
    );
}
