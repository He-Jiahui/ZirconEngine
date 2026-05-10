use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiBrushPayload, UiPaintElement, UiPaintPayload, UiRenderCommand, UiRenderResourceKey,
        UiRenderResourceKind, UiResolvedStyle, UiTextAlign, UiTextPaint, UiTextPaintDecorationKind,
        UiTextRunPaintStyle, UiVisualAssetRef,
    },
};

use super::super::data::FrameRect;
use super::frame::HostRgbaFrame;
use super::geometry::{inset, is_visible_frame};
use super::primitives::{draw_border_clipped, draw_rect_clipped, draw_rgba_image_clipped};
use super::text::draw_text_with_size_and_style;
use super::theme::PALETTE;
use super::visual_assets::{
    load_visual_asset_pixels_for_size, raster_size_from_frame, HostPaintImagePixels,
};

const FALLBACK_PANEL: [u8; 4] = PALETTE.surface;
const FALLBACK_TEXT: [u8; 4] = PALETTE.text;
const FALLBACK_IMAGE_BORDER: [u8; 4] = PALETTE.focus_ring;

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
    text_style: UiTextRunPaintStyle,
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
            text_style: UiTextRunPaintStyle::default(),
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
        text_style: UiTextRunPaintStyle,
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
            text_style,
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
            text_style: UiTextRunPaintStyle::default(),
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
            text_style: UiTextRunPaintStyle::default(),
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
            text_style: UiTextRunPaintStyle::default(),
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
    let command_clip = command
        .clip_frame
        .map(frame_from_ui)
        .or_else(|| parent_clip.cloned());

    for element in command.to_paint_elements(0) {
        push_runtime_paint_element(output, command, &element, command_clip.clone());
    }
}

fn push_runtime_paint_element(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    element: &UiPaintElement,
    clip_frame: Option<FrameRect>,
) {
    let frame = frame_from_ui(element.geometry.render_bounds);
    if !is_visible_frame(&frame) || element.effects.opacity <= 0.0 {
        return;
    }

    match &element.payload {
        UiPaintPayload::Empty => output.push(HostPaintCommand::group(
            frame,
            clip_frame,
            element.z_index,
            element.effects.opacity,
        )),
        UiPaintPayload::Brush { brushes } => {
            if let Some(image_brush) = brushes.fill.as_ref().and_then(image_brush_resource) {
                push_image_resource_command(
                    output,
                    image_brush,
                    frame,
                    clip_frame,
                    element.z_index,
                    element.effects.opacity,
                );
            } else {
                let background_color = brushes.fill.as_ref().and_then(brush_fill_color);
                let (border_color, border_width) = brushes
                    .border
                    .as_ref()
                    .and_then(brush_border)
                    .unwrap_or((None, 0.0));
                output.push(HostPaintCommand::quad(
                    frame,
                    clip_frame,
                    element.z_index,
                    background_color,
                    border_color,
                    border_width,
                    element.effects.opacity,
                ));
            }
        }
        UiPaintPayload::Text { text } => {
            push_text_paint_commands(output, command, text, frame, clip_frame, element.z_index)
        }
    }
}

fn push_text_paint_commands(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    text: &UiTextPaint,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
) {
    let color = parse_style_color(text.color.as_deref())
        .unwrap_or_else(|| runtime_foreground_color(&command.style));
    push_text_decorations(
        output,
        text,
        clip_frame.clone(),
        z_index,
        command.opacity,
        true,
    );
    if !text.runs.is_empty() {
        for run in &text.runs {
            let run_color = parse_style_color(run.color.as_deref()).unwrap_or(color);
            output.push(HostPaintCommand::text(
                frame_from_ui(run.frame),
                clip_frame.clone(),
                z_index,
                run.text.clone(),
                run_color,
                run.font_size.max(1.0),
                run.line_height.max(run.font_size).max(1.0),
                run.style,
                command.opacity,
            ));
        }
        push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
        return;
    }

    if let Some(shaped) = text.shaped.as_ref() {
        for line in &shaped.lines {
            output.push(HostPaintCommand::text(
                frame_from_ui(line.frame),
                clip_frame.clone(),
                z_index,
                line.text.clone(),
                color,
                text.font_size.max(1.0),
                text.line_height.max(text.font_size).max(1.0),
                UiTextRunPaintStyle::default(),
                command.opacity,
            ));
        }
        push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
        return;
    }

    let text_x = aligned_text_x(&frame, &text.source_text, &command.style);
    output.push(HostPaintCommand::text(
        FrameRect {
            x: text_x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        },
        clip_frame.clone(),
        z_index,
        text.source_text.clone(),
        color,
        text.font_size.max(1.0),
        text.line_height.max(text.font_size).max(1.0),
        UiTextRunPaintStyle::default(),
        command.opacity,
    ));
    push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
}

fn push_text_decorations(
    output: &mut Vec<HostPaintCommand>,
    text: &UiTextPaint,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
    before_text: bool,
) {
    for decoration in &text.decorations {
        let decoration_before_text =
            matches!(decoration.kind, UiTextPaintDecorationKind::Selection);
        if decoration_before_text != before_text {
            continue;
        }
        let color =
            parse_style_color(Some(decoration.color.as_str())).unwrap_or(match decoration.kind {
                UiTextPaintDecorationKind::Selection => [77, 137, 255, 102],
                UiTextPaintDecorationKind::CompositionUnderline => [77, 137, 255, 255],
                UiTextPaintDecorationKind::Caret => [232, 238, 247, 255],
                UiTextPaintDecorationKind::Outline => [232, 238, 247, 255],
            });
        let decoration_z = if decoration_before_text {
            z_index - 1
        } else {
            z_index + 1
        };
        output.push(HostPaintCommand::quad(
            frame_from_ui(decoration.frame),
            clip_frame.clone(),
            decoration_z,
            Some(color),
            None,
            0.0,
            opacity,
        ));
    }
}

fn push_image_resource_command(
    output: &mut Vec<HostPaintCommand>,
    resource: &UiRenderResourceKey,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
) {
    if let Some(asset) = visual_asset_from_resource(resource) {
        if let Some((target_width, target_height)) =
            raster_size_from_frame(frame.width, frame.height)
        {
            if let Some(image) =
                load_visual_asset_pixels_for_size(&asset, target_width, target_height)
            {
                output.push(HostPaintCommand::image_pixels(
                    frame,
                    clip_frame,
                    z_index,
                    image.width,
                    image.height,
                    image.rgba,
                    opacity,
                ));
                return;
            }
        }
    }

    output.push(HostPaintCommand::image(
        frame,
        clip_frame,
        z_index,
        resource_image_key(resource),
        opacity,
    ));
}

fn brush_fill_color(brush: &UiBrushPayload) -> Option<[u8; 4]> {
    match brush {
        UiBrushPayload::Solid(payload) => parse_style_color(Some(&payload.color)),
        UiBrushPayload::Rounded(payload) => parse_style_color(Some(&payload.color)),
        UiBrushPayload::Material(payload) => parse_style_color(payload.fallback_color.as_deref()),
        UiBrushPayload::Gradient(payload) => payload
            .stops
            .first()
            .and_then(|stop| parse_style_color(Some(&stop.color))),
        _ => None,
    }
}

fn brush_border(brush: &UiBrushPayload) -> Option<(Option<[u8; 4]>, f32)> {
    match brush {
        UiBrushPayload::Border(payload) => {
            Some((parse_style_color(Some(&payload.color)), payload.width))
        }
        _ => None,
    }
}

fn image_brush_resource(brush: &UiBrushPayload) -> Option<&UiRenderResourceKey> {
    match brush {
        UiBrushPayload::Image(payload) | UiBrushPayload::Box(payload) => Some(&payload.resource),
        UiBrushPayload::Vector(payload) => Some(&payload.resource),
        _ => None,
    }
}

fn visual_asset_from_resource(resource: &UiRenderResourceKey) -> Option<UiVisualAssetRef> {
    match resource.kind {
        UiRenderResourceKind::Icon => Some(UiVisualAssetRef::Icon(resource.id.clone())),
        UiRenderResourceKind::Image | UiRenderResourceKind::Vector => {
            Some(UiVisualAssetRef::Image(resource.id.clone()))
        }
        _ => None,
    }
}

fn resource_image_key(resource: &UiRenderResourceKey) -> String {
    match resource.kind {
        UiRenderResourceKind::Icon => format!("icon:{}", resource.id),
        UiRenderResourceKind::Image | UiRenderResourceKind::Vector => {
            format!("image:{}", resource.id)
        }
        UiRenderResourceKind::Material => format!("material:{}", resource.id),
        UiRenderResourceKind::Font => format!("font:{}", resource.id),
        UiRenderResourceKind::Texture => format!("texture:{}", resource.id),
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
