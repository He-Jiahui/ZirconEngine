use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

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
            (3.0, 2.5, 9.6, 1.1),
            (3.0, 3.0, 1.1, 10.0),
            (12.0, 3.0, 1.1, 10.0),
            (4.0, 12.0, 8.0, 1.1),
            (5.1, 3.8, 4.6, 2.6),
            (10.6, 3.8, 1.1, 3.2),
            (5.2, 8.6, 5.6, 1.1),
            (5.2, 10.2, 5.6, 1.1),
        ],
    );
}
