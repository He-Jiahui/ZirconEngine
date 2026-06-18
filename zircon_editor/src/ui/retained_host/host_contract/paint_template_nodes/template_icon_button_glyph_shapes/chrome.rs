use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(super) fn push_play_icon(
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
            (4.0, 3.0, 2.0, 10.0),
            (6.0, 4.0, 2.0, 8.0),
            (8.0, 5.0, 2.0, 6.0),
            (10.0, 6.0, 2.0, 4.0),
        ],
    );
}

pub(super) fn push_chevron_down_icon(
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

pub(super) fn push_grid_icon(
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

pub(super) fn push_sun_icon(
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

pub(super) fn push_more_icon(
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
