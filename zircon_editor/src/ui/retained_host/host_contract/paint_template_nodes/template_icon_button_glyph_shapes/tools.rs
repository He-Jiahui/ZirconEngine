use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(super) fn push_cursor_icon(
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

pub(super) fn push_move_icon(
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
            (7.2, 2.0, 1.6, 12.0),
            (2.0, 7.2, 12.0, 1.6),
            (6.0, 3.0, 4.0, 1.2),
            (6.0, 12.0, 4.0, 1.2),
            (3.0, 6.0, 1.2, 4.0),
            (12.0, 6.0, 1.2, 4.0),
        ],
    );
}

pub(super) fn push_rotate_icon(
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
            (4.0, 3.0, 6.0, 1.3),
            (3.0, 4.0, 1.3, 5.0),
            (4.0, 10.5, 7.0, 1.3),
            (11.0, 7.0, 1.3, 4.5),
            (9.0, 2.0, 3.5, 1.3),
            (11.0, 2.0, 1.3, 3.5),
        ],
    );
}

pub(super) fn push_scale_icon(
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
            (3.0, 3.0, 5.0, 1.3),
            (3.0, 3.0, 1.3, 5.0),
            (8.0, 8.0, 5.0, 1.3),
            (12.0, 8.0, 1.3, 5.0),
            (4.0, 11.0, 8.0, 1.3),
            (10.0, 5.0, 1.3, 7.0),
        ],
    );
}

pub(super) fn push_snap_icon(
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
            (3.0, 3.0, 1.4, 7.0),
            (11.5, 3.0, 1.4, 7.0),
            (4.0, 10.0, 3.0, 1.4),
            (9.0, 10.0, 3.0, 1.4),
            (6.8, 11.0, 2.4, 2.0),
        ],
    );
}
