use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

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
            (3.0, 6.0, 2.0, 1.4),
            (4.0, 5.0, 1.4, 2.0),
            (4.0, 9.0, 1.4, 2.0),
            (11.0, 6.0, 2.0, 1.4),
            (10.6, 5.0, 1.4, 2.0),
            (10.6, 9.0, 1.4, 2.0),
            (7.2, 4.0, 1.2, 8.0),
        ],
    );
}
