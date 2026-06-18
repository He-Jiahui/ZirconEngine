use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::super::template_icon_button_glyph_segments::push_icon_button_glyph_segments as push_segments;

pub(super) fn push_menu_icon(
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
            (2.5, 4.0, 11.0, 1.5),
            (2.5, 7.5, 11.0, 1.5),
            (2.5, 11.0, 11.0, 1.5),
        ],
    );
}

pub(super) fn push_file_icon(
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
            (4.0, 2.0, 6.0, 1.2),
            (3.0, 3.0, 1.2, 10.0),
            (12.0, 5.0, 1.2, 8.0),
            (4.0, 12.0, 8.0, 1.2),
            (10.0, 3.0, 1.2, 3.0),
            (10.0, 5.0, 3.0, 1.2),
        ],
    );
}

pub(super) fn push_folder_icon(
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
            (2.0, 5.0, 4.0, 1.2),
            (5.0, 4.0, 4.0, 1.2),
            (2.0, 6.0, 12.0, 1.2),
            (2.0, 7.0, 1.2, 5.0),
            (13.0, 7.0, 1.2, 5.0),
            (3.0, 12.0, 10.0, 1.2),
        ],
    );
}

pub(super) fn push_save_icon(
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
            (3.0, 2.5, 10.0, 1.2),
            (3.0, 3.0, 1.2, 10.0),
            (12.0, 3.0, 1.2, 10.0),
            (4.0, 12.0, 8.0, 1.2),
            (5.0, 3.0, 5.0, 3.0),
            (6.0, 9.0, 5.0, 1.2),
        ],
    );
}
