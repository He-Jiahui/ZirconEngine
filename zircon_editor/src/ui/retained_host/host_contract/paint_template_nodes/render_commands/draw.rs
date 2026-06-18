use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{inset, is_visible_frame};
use super::super::super::paint_primitives::{
    draw_border_clipped, draw_rect_clipped, draw_rgba_image_clipped_with_atlas,
    draw_rgba_image_clipped_with_resource_key, draw_rounded_border_clipped,
    draw_rounded_rect_clipped,
};
use super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::paint_theme::PALETTE;
use super::super::visual_assets::HostPaintImagePixels;
use super::command::{HostPaintCommand, HostPaintCommandKind};

const FALLBACK_TEXT: [u8; 4] = PALETTE.text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_host_paint_commands(
    frame: &mut HostRgbaFrame,
    commands: &[HostPaintCommand],
) -> bool {
    let mut ordered = {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_collect_order");
        commands.iter().enumerate().collect::<Vec<_>>()
    };
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_sort");
        ordered.sort_by_key(|(index, command)| (command.z_index, *index));
    }

    let mut drew_any = false;
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_draw_ordered");
        for (_, command) in ordered {
            drew_any |= draw_host_paint_command(frame, command);
        }
    }
    drew_any
}

fn draw_host_paint_command(frame: &mut HostRgbaFrame, command: &HostPaintCommand) -> bool {
    if command.opacity <= 0.0 || !command.opacity.is_finite() || !is_visible_frame(&command.frame) {
        return false;
    }

    match command.kind {
        HostPaintCommandKind::Group => false,
        HostPaintCommandKind::Quad => {
            zircon_runtime::profile_scope!("editor", "host_painter", "paint_command_quad");
            draw_quad_command(frame, command)
        }
        HostPaintCommandKind::Text => {
            zircon_runtime::profile_scope!("editor", "host_painter", "paint_command_text");
            draw_text_command(frame, command)
        }
        HostPaintCommandKind::Image => {
            zircon_runtime::profile_scope!("editor", "host_painter", "paint_command_image");
            draw_image_command(frame, command)
        }
    }
}

fn draw_quad_command(frame: &mut HostRgbaFrame, command: &HostPaintCommand) -> bool {
    let clip = command.clip_frame.as_ref();
    let mut drew_any = false;
    if let Some(color) = command
        .background_color
        .map(|color| color_with_opacity(color, command.opacity))
    {
        if command.corner_radius > 0.0 {
            draw_rounded_rect_clipped(
                frame,
                command.frame.clone(),
                clip,
                color,
                command.corner_radius,
            );
        } else {
            draw_rect_clipped(frame, command.frame.clone(), clip, color);
        }
        drew_any = true;
    }
    if command.border_width > 0.0 {
        if let Some(color) = command
            .border_color
            .map(|color| color_with_opacity(color, command.opacity))
        {
            if command.corner_radius > 0.0 {
                draw_rounded_border_clipped(
                    frame,
                    command.frame.clone(),
                    clip,
                    color,
                    command.border_width,
                    command.corner_radius,
                );
            } else {
                draw_border_width(frame, &command.frame, clip, color, command.border_width);
            }
            drew_any = true;
        }
    }
    drew_any
}

fn draw_text_command(frame: &mut HostRgbaFrame, command: &HostPaintCommand) -> bool {
    let Some(text) = command.text.as_ref() else {
        return false;
    };
    let color = color_with_opacity(
        command.foreground_color.unwrap_or(FALLBACK_TEXT),
        command.opacity,
    );
    draw_text_with_size_and_style(
        frame,
        command.frame.clone(),
        text,
        command.clip_frame.as_ref(),
        color,
        command.font_size,
        command.line_height,
        command.text_style,
    );
    true
}

fn draw_image_command(frame: &mut HostRgbaFrame, command: &HostPaintCommand) -> bool {
    if let Some(image) = command.image_pixels.as_ref() {
        if command.opacity >= 1.0 {
            if let Some(atlas) = image.atlas.as_ref().filter(|atlas| atlas.rgba.is_some()) {
                if draw_rgba_image_clipped_with_atlas(
                    frame,
                    command.frame.clone(),
                    command.clip_frame.as_ref(),
                    image.width,
                    image.height,
                    &image.rgba,
                    atlas,
                ) {
                    return true;
                }
            }
        }
        let rgba;
        let source = if command.opacity < 1.0 {
            rgba = image_pixels_with_opacity(image, command.opacity);
            &rgba
        } else {
            image.rgba.as_slice()
        };
        if draw_rgba_image_clipped_with_resource_key(
            frame,
            command.frame.clone(),
            command.clip_frame.as_ref(),
            &image.resource_key,
            image.width,
            image.height,
            source,
        ) {
            return true;
        }
    }

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

fn draw_border_width(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    border_width: f32,
) {
    let pixel_width = border_width.ceil().max(1.0).min(8.0) as u32;
    for offset in 0..pixel_width {
        draw_border_clipped(frame, inset(rect, offset as f32), clip, color);
    }
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

fn image_pixels_with_opacity(image: &HostPaintImagePixels, opacity: f32) -> Vec<u8> {
    let opacity = opacity.clamp(0.0, 1.0);
    let mut rgba = image.rgba.clone();
    for pixel in rgba.chunks_exact_mut(4) {
        pixel[3] = ((pixel[3] as f32 * opacity).round()).clamp(0.0, 255.0) as u8;
    }
    rgba
}

fn color_with_opacity(mut color: [u8; 4], opacity: f32) -> [u8; 4] {
    let opacity = opacity.clamp(0.0, 1.0);
    color[3] = ((color[3] as f32 * opacity).round()).clamp(0.0, 255.0) as u8;
    color
}
