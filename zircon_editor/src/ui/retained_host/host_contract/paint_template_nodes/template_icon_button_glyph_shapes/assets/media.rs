use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_image_icon(
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
            (3.0, 3.0, 10.0, 1.2),
            (3.0, 4.0, 1.2, 9.0),
            (12.0, 4.0, 1.2, 9.0),
            (4.0, 12.0, 8.0, 1.2),
            (5.0, 10.0, 3.0, 1.2),
            (7.0, 8.0, 3.0, 1.2),
            (10.0, 6.0, 1.6, 1.6),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_audio_icon(
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
            (3.0, 6.0, 3.0, 4.0),
            (6.0, 4.0, 2.0, 8.0),
            (9.0, 5.0, 1.2, 2.0),
            (11.0, 4.0, 1.2, 4.0),
            (9.0, 9.0, 1.2, 2.0),
            (11.0, 8.0, 1.2, 4.0),
        ],
    );
}
