use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(super) fn push_plus_icon(
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
        &[(7.2, 3.0, 1.6, 10.0), (3.0, 7.2, 10.0, 1.6)],
    );
}

pub(super) fn push_trash_icon(
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
            (4.0, 5.0, 8.0, 1.2),
            (5.0, 3.0, 6.0, 1.2),
            (5.0, 6.0, 1.2, 7.0),
            (10.0, 6.0, 1.2, 7.0),
            (6.0, 12.0, 4.0, 1.2),
        ],
    );
}

pub(super) fn push_filter_icon(
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
            (3.0, 3.0, 10.0, 1.4),
            (5.0, 6.0, 6.0, 1.4),
            (7.0, 8.0, 2.0, 5.0),
        ],
    );
}
