use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;

pub(super) const BUTTON_ICON_SIZE: f32 = 14.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ButtonGlyph {
    None,
    Plus,
    Trash,
    ChevronDown,
}

pub(super) fn button_glyph_for_key(key: &str) -> ButtonGlyph {
    if key.contains("delete") || key.contains("trash") || key.contains("danger") {
        ButtonGlyph::Trash
    } else if key.contains("dropdown") || key.contains("drop-down") || key.contains("menu") {
        ButtonGlyph::ChevronDown
    } else if key.contains("icon") || key.contains("add") || key.contains("plus") {
        ButtonGlyph::Plus
    } else {
        ButtonGlyph::None
    }
}

pub(super) fn push_button_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    glyph: ButtonGlyph,
    color: [u8; 4],
    opacity: f32,
) {
    match glyph {
        ButtonGlyph::Plus => push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[(6.0, 2.0, 2.0, 10.0), (2.0, 6.0, 10.0, 2.0)],
        ),
        ButtonGlyph::Trash => push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[
                (3.0, 4.0, 8.0, 1.2),
                (4.0, 2.0, 6.0, 1.2),
                (4.0, 5.0, 1.2, 7.0),
                (9.0, 5.0, 1.2, 7.0),
                (5.0, 12.0, 4.0, 1.2),
            ],
        ),
        ButtonGlyph::ChevronDown => push_segments(
            commands,
            rect,
            clip,
            order,
            color,
            opacity,
            &[
                (3.0, 5.0, 2.0, 2.0),
                (5.0, 7.0, 4.0, 2.0),
                (9.0, 5.0, 2.0, 2.0),
            ],
        ),
        ButtonGlyph::None => {}
    }
}

fn push_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[(f32, f32, f32, f32)],
) {
    for (x, y, width, height) in segments {
        commands.push(HostPaintCommand::quad(
            scaled_rect(origin, *x, *y, *width, *height),
            Some(clip.clone()),
            order,
            Some(color),
            None,
            0.0,
            1.0,
            opacity,
        ));
    }
}

fn scaled_rect(origin: &FrameRect, x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    let scale_x = origin.width / BUTTON_ICON_SIZE;
    let scale_y = origin.height / BUTTON_ICON_SIZE;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}
