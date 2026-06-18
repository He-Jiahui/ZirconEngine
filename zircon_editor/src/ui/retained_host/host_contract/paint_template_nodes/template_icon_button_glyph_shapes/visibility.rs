use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(super) fn push_eye_icon(
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
            (2.5, 7.0, 2.0, 2.0),
            (4.5, 5.0, 7.0, 1.2),
            (4.5, 10.0, 7.0, 1.2),
            (11.5, 7.0, 2.0, 2.0),
            (7.0, 7.0, 2.0, 2.0),
        ],
    );
}

pub(super) fn push_eye_off_icon(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_eye_icon(commands, rect, clip, order, color, opacity);
    push_segments(
        commands,
        rect,
        clip,
        order + 1,
        color,
        opacity,
        &[(3.0, 12.0, 10.0, 1.4)],
    );
}

pub(super) fn push_lock_icon(
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
            (4.0, 7.0, 8.0, 6.0),
            (5.0, 4.0, 6.0, 1.2),
            (4.0, 5.0, 1.2, 3.0),
            (11.0, 5.0, 1.2, 3.0),
        ],
    );
}
