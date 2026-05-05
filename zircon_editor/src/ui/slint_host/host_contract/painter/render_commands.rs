use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiRenderCommand, UiRenderCommandKind, UiResolvedStyle, UiResolvedTextLayout, UiTextAlign,
        UiVisualAssetRef,
    },
};

use super::super::data::FrameRect;
use super::frame::HostRgbaFrame;
use super::geometry::{inset, is_visible_frame};
use super::primitives::{draw_border_clipped, draw_rect_clipped, draw_rgba_image_clipped};
use super::text::draw_text_with_size;
use super::visual_assets::{load_visual_asset_pixels, HostPaintImagePixels};

const FALLBACK_PANEL: [u8; 4] = [32, 37, 46, 255];
const FALLBACK_TEXT: [u8; 4] = [210, 220, 235, 255];
const FALLBACK_IMAGE_BORDER: [u8; 4] = [92, 156, 255, 255];

#[derive(Clone, Copy)]
enum HostPaintCommandKind {
    Group,
    Quad,
    Text,
    Image,
}

#[derive(Clone)]
pub(super) struct HostPaintCommand {
    kind: HostPaintCommandKind,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    background_color: Option<[u8; 4]>,
    foreground_color: Option<[u8; 4]>,
    border_color: Option<[u8; 4]>,
    border_width: f32,
    text: Option<String>,
    font_size: f32,
    line_height: f32,
    image_key: Option<String>,
    image_pixels: Option<HostPaintImagePixels>,
    opacity: f32,
}

impl HostPaintCommand {
    pub(super) fn quad(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        background_color: Option<[u8; 4]>,
        border_color: Option<[u8; 4]>,
        border_width: f32,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Quad,
            frame,
            clip_frame,
            z_index,
            background_color,
            foreground_color: None,
            border_color,
            border_width,
            text: None,
            font_size: 12.0,
            line_height: 14.0,
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }

    pub(super) fn text(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        text: String,
        foreground_color: [u8; 4],
        font_size: f32,
        line_height: f32,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Text,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: Some(foreground_color),
            border_color: None,
            border_width: 0.0,
            text: Some(text),
            font_size,
            line_height,
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }

    fn group(frame: FrameRect, clip_frame: Option<FrameRect>, z_index: i32, opacity: f32) -> Self {
        Self {
            kind: HostPaintCommandKind::Group,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: None,
            border_color: None,
            border_width: 0.0,
            text: None,
            font_size: 12.0,
            line_height: 14.0,
            image_key: None,
            image_pixels: None,
            opacity,
        }
    }

    fn image(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        image_key: String,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Image,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: None,
            border_color: Some(FALLBACK_IMAGE_BORDER),
            border_width: 1.0,
            text: None,
            font_size: 12.0,
            line_height: 14.0,
            image_key: Some(image_key),
            image_pixels: None,
            opacity,
        }
    }

    pub(super) fn image_pixels(
        frame: FrameRect,
        clip_frame: Option<FrameRect>,
        z_index: i32,
        image_width: u32,
        image_height: u32,
        rgba: Vec<u8>,
        opacity: f32,
    ) -> Self {
        Self {
            kind: HostPaintCommandKind::Image,
            frame,
            clip_frame,
            z_index,
            background_color: None,
            foreground_color: None,
            border_color: None,
            border_width: 0.0,
            text: None,
            font_size: 12.0,
            line_height: 14.0,
            image_key: None,
            image_pixels: Some(HostPaintImagePixels {
                width: image_width,
                height: image_height,
                rgba,
            }),
            opacity,
        }
    }
}

pub(super) fn draw_host_paint_commands(
    frame: &mut HostRgbaFrame,
    commands: &[HostPaintCommand],
) -> bool {
    let mut ordered = commands.iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(index, command)| (command.z_index, *index));

    let mut drew_any = false;
    for (_, command) in ordered {
        drew_any |= draw_host_paint_command(frame, command);
    }
    drew_any
}

pub(super) fn draw_runtime_render_commands(
    frame: &mut HostRgbaFrame,
    commands: &[UiRenderCommand],
    clip_frame: Option<&FrameRect>,
) -> bool {
    let mut host_commands = Vec::new();
    for command in commands {
        push_runtime_command(&mut host_commands, command, clip_frame);
    }
    draw_host_paint_commands(frame, &host_commands)
}

#[cfg(test)]
pub(crate) fn paint_runtime_render_commands_for_test(
    width: u32,
    height: u32,
    commands: &[UiRenderCommand],
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    draw_runtime_render_commands(&mut frame, commands, None);
    frame.into_bytes()
}

fn draw_host_paint_command(frame: &mut HostRgbaFrame, command: &HostPaintCommand) -> bool {
    if command.opacity <= 0.0 || !command.opacity.is_finite() || !is_visible_frame(&command.frame) {
        return false;
    }

    match command.kind {
        HostPaintCommandKind::Group => false,
        HostPaintCommandKind::Quad => draw_quad_command(frame, command),
        HostPaintCommandKind::Text => draw_text_command(frame, command),
        HostPaintCommandKind::Image => draw_image_command(frame, command),
    }
}

fn draw_quad_command(frame: &mut HostRgbaFrame, command: &HostPaintCommand) -> bool {
    let clip = command.clip_frame.as_ref();
    let mut drew_any = false;
    if let Some(color) = command
        .background_color
        .map(|color| color_with_opacity(color, command.opacity))
    {
        draw_rect_clipped(frame, command.frame.clone(), clip, color);
        drew_any = true;
    }
    if command.border_width > 0.0 {
        if let Some(color) = command
            .border_color
            .map(|color| color_with_opacity(color, command.opacity))
        {
            draw_border_width(frame, &command.frame, clip, color, command.border_width);
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
    draw_text_with_size(
        frame,
        command.frame.clone(),
        text,
        command.clip_frame.as_ref(),
        color,
        command.font_size,
        command.line_height,
    );
    true
}

fn draw_image_command(frame: &mut HostRgbaFrame, command: &HostPaintCommand) -> bool {
    if let Some(image) = command.image_pixels.as_ref() {
        let rgba;
        let source = if command.opacity < 1.0 {
            rgba = image_pixels_with_opacity(image, command.opacity);
            &rgba
        } else {
            image.rgba.as_slice()
        };
        if draw_rgba_image_clipped(
            frame,
            command.frame.clone(),
            command.clip_frame.as_ref(),
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

fn push_runtime_command(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    parent_clip: Option<&FrameRect>,
) {
    let frame = frame_from_ui(command.frame);
    if !is_visible_frame(&frame) || command.opacity <= 0.0 {
        return;
    }

    let command_clip = command
        .clip_frame
        .map(frame_from_ui)
        .or_else(|| parent_clip.cloned());
    match command.kind {
        UiRenderCommandKind::Group => {
            output.push(HostPaintCommand::group(
                frame,
                command_clip,
                command.z_index,
                command.opacity,
            ));
        }
        UiRenderCommandKind::Quad => {
            output.push(HostPaintCommand::quad(
                frame,
                command_clip,
                command.z_index,
                runtime_background_color(&command.style),
                runtime_border_color(&command.style),
                command.style.border_width,
                command.opacity,
            ));
        }
        UiRenderCommandKind::Text => {
            push_runtime_text_command(output, command, frame, command_clip);
        }
        UiRenderCommandKind::Image => {
            let image_key = image_key(command.image.as_ref());
            if let Some(image) = command.image.as_ref().and_then(load_visual_asset_pixels) {
                output.push(HostPaintCommand::image_pixels(
                    frame,
                    command_clip,
                    command.z_index,
                    image.width,
                    image.height,
                    image.rgba,
                    command.opacity,
                ));
            } else {
                output.push(HostPaintCommand::image(
                    frame,
                    command_clip,
                    command.z_index,
                    image_key,
                    command.opacity,
                ));
            }
        }
    }
}

fn push_runtime_text_command(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
) {
    let color = runtime_foreground_color(&command.style);
    if let Some(text_layout) = command.text_layout.as_ref() {
        push_text_layout_commands(
            output,
            text_layout,
            clip_frame,
            command.z_index,
            color,
            command.opacity,
        );
        return;
    }

    let text = command.text.clone().unwrap_or_default();
    let text_x = aligned_text_x(&frame, &text, &command.style);
    output.push(HostPaintCommand::text(
        FrameRect {
            x: text_x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        },
        clip_frame,
        command.z_index,
        text,
        color,
        command.style.font_size.max(1.0),
        command
            .style
            .line_height
            .max(command.style.font_size)
            .max(1.0),
        command.opacity,
    ));
}

fn push_text_layout_commands(
    output: &mut Vec<HostPaintCommand>,
    text_layout: &UiResolvedTextLayout,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    color: [u8; 4],
    opacity: f32,
) {
    for line in &text_layout.lines {
        output.push(HostPaintCommand::text(
            frame_from_ui(line.frame),
            clip_frame.clone(),
            z_index,
            line.text.clone(),
            color,
            12.0,
            line.frame.height.max(14.0),
            opacity,
        ));
    }
}

fn frame_from_ui(frame: UiFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn runtime_background_color(style: &UiResolvedStyle) -> Option<[u8; 4]> {
    parse_style_color(style.background_color.as_deref()).or(Some(FALLBACK_PANEL))
}

fn runtime_border_color(style: &UiResolvedStyle) -> Option<[u8; 4]> {
    parse_style_color(style.border_color.as_deref())
}

fn runtime_foreground_color(style: &UiResolvedStyle) -> [u8; 4] {
    parse_style_color(style.foreground_color.as_deref()).unwrap_or(FALLBACK_TEXT)
}

fn aligned_text_x(frame: &FrameRect, text: &str, style: &UiResolvedStyle) -> f32 {
    let estimated_width = text.chars().count() as f32 * (style.font_size.max(1.0) * 0.5);
    match style.text_align {
        UiTextAlign::Left => frame.x,
        UiTextAlign::Center => frame.x + (frame.width - estimated_width).max(0.0) * 0.5,
        UiTextAlign::Right => frame.x + (frame.width - estimated_width).max(0.0),
    }
}

fn image_key(image: Option<&UiVisualAssetRef>) -> String {
    match image {
        Some(UiVisualAssetRef::Icon(icon)) => format!("icon:{icon}"),
        Some(UiVisualAssetRef::Image(image)) => format!("image:{image}"),
        None => "image".to_string(),
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

pub(super) fn parse_style_color(value: Option<&str>) -> Option<[u8; 4]> {
    let value = value?.trim();
    let hex = value.strip_prefix('#')?;
    match hex.len() {
        3 => Some([
            parse_nibble(hex.as_bytes()[0])? * 17,
            parse_nibble(hex.as_bytes()[1])? * 17,
            parse_nibble(hex.as_bytes()[2])? * 17,
            255,
        ]),
        4 => Some([
            parse_nibble(hex.as_bytes()[0])? * 17,
            parse_nibble(hex.as_bytes()[1])? * 17,
            parse_nibble(hex.as_bytes()[2])? * 17,
            parse_nibble(hex.as_bytes()[3])? * 17,
        ]),
        6 => Some([
            parse_hex_pair(hex, 0)?,
            parse_hex_pair(hex, 2)?,
            parse_hex_pair(hex, 4)?,
            255,
        ]),
        8 => Some([
            parse_hex_pair(hex, 0)?,
            parse_hex_pair(hex, 2)?,
            parse_hex_pair(hex, 4)?,
            parse_hex_pair(hex, 6)?,
        ]),
        _ => None,
    }
}

fn parse_hex_pair(hex: &str, offset: usize) -> Option<u8> {
    let bytes = hex.as_bytes();
    Some(parse_nibble(bytes[offset])? * 16 + parse_nibble(bytes[offset + 1])?)
}

fn parse_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
