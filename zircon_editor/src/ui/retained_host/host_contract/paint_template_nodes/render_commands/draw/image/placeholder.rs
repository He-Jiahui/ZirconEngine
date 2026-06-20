use super::super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::super::paint_geometry::inset;
use super::super::super::super::super::paint_primitives::draw_rect_clipped;
use super::super::super::command::HostPaintCommand;
use super::super::{border::draw_border_width, color::color_with_opacity};

pub(super) fn draw_image_placeholder(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    let image_key = command.image_key.as_deref().unwrap_or("image");
    let color = color_with_opacity(image_placeholder_color(image_key), command.opacity);
    let clip = command.clip_frame.as_ref();
    draw_rect_clipped(frame, command.frame.clone(), clip, color);
    let inner = inset(&command.frame, 5.0);
    draw_rect_clipped(
        frame,
        inner,
        clip,
        color_with_opacity([255, 255, 255, 42], command.opacity),
    );
    if let Some(border) = command.border_color {
        draw_border_width(
            frame,
            &command.frame,
            clip,
            color_with_opacity(border, command.opacity),
            command.border_width.max(1.0),
        );
    }
    true
}

fn image_placeholder_color(key: &str) -> [u8; 4] {
    let seed = key.bytes().fold(0_u32, |sum, byte| {
        sum.wrapping_mul(31).wrapping_add(byte as u32)
    });
    [
        48 + (seed & 0x3f) as u8,
        70 + ((seed >> 6) & 0x5f) as u8,
        96 + ((seed >> 13) & 0x5f) as u8,
        255,
    ]
}
