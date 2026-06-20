use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_grid_icon(
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
            (3.0, 3.0, 4.0, 4.0),
            (9.0, 3.0, 4.0, 4.0),
            (3.0, 9.0, 4.0, 4.0),
            (9.0, 9.0, 4.0, 4.0),
        ],
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_sun_icon(
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
            (6.0, 6.0, 4.0, 4.0),
            (7.2, 2.0, 1.6, 2.4),
            (7.2, 11.6, 1.6, 2.4),
            (2.0, 7.2, 2.4, 1.6),
            (11.6, 7.2, 2.4, 1.6),
        ],
    );
}
