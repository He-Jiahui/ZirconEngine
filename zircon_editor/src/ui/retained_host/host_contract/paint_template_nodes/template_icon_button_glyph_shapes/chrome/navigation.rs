use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chevron_down_icon(
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
            (4.0, 6.0, 2.0, 2.0),
            (6.0, 8.0, 4.0, 2.0),
            (10.0, 6.0, 2.0, 2.0),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_more_icon(
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
            (7.0, 3.0, 2.0, 2.0),
            (7.0, 7.0, 2.0, 2.0),
            (7.0, 11.0, 2.0, 2.0),
        ],
    );
}
