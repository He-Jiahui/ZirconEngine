use super::super::data::FrameRect;
use super::render_commands::HostPaintCommand;

const ICON_GRID: f32 = 16.0;

pub(super) type IconButtonGlyphSegment = (f32, f32, f32, f32);

pub(super) fn push_icon_button_glyph_segments(
    commands: &mut Vec<HostPaintCommand>,
    origin: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
    segments: &[IconButtonGlyphSegment],
) {
    for (x, y, width, height) in segments {
        commands.push(HostPaintCommand::quad(
            scaled_icon_button_glyph_rect(origin, *x, *y, *width, *height),
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

fn scaled_icon_button_glyph_rect(
    origin: &FrameRect,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> FrameRect {
    let scale_x = origin.width / ICON_GRID;
    let scale_y = origin.height / ICON_GRID;
    FrameRect {
        x: origin.x + x * scale_x,
        y: origin.y + y * scale_y,
        width: (width * scale_x).max(1.0),
        height: (height * scale_y).max(1.0),
    }
}
