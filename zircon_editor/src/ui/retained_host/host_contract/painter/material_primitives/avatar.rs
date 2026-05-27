use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, HostPaintImagePixels,
};
use super::{component_variant_contains, first_non_empty, resolved_style_color};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const AVATAR_DEFAULT_EDGE: f32 = 40.0;
const AVATAR_ROUNDED_RADIUS: f32 = 4.0;
const AVATAR_FALLBACK_SCALE: f32 = 0.75;
const AVATAR_TEXT_FONT_RATIO: f32 = 0.5;
const AVATAR_MIN_FONT_SIZE: f32 = 8.0;
const AVATAR_TEXT_WIDTH_RATIO: f32 = 0.58;
const MUI_GREY_600: [u8; 4] = [117, 117, 117, 255];

pub(super) fn push_avatar_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_avatar_node(node) {
        return false;
    }

    let avatar_rect = avatar_frame(rect);
    if avatar_rect.width <= 0.0 || avatar_rect.height <= 0.0 {
        return true;
    }

    let corner_radius = avatar_corner_radius(node, &avatar_rect);
    let avatar_image = avatar_image_pixels(node, &avatar_rect, corner_radius);
    let background = avatar_background_color(node, avatar_image.is_none());
    let foreground = avatar_foreground_color(node);
    commands.push(HostPaintCommand::quad(
        avatar_rect.clone(),
        Some(clip.clone()),
        order,
        Some(background),
        None,
        0.0,
        corner_radius,
        opacity,
    ));

    if let Some(image) = avatar_image {
        push_avatar_image(
            commands,
            image,
            avatar_rect.clone(),
            clip,
            order + 1,
            opacity,
        );
    } else if !avatar_label(node).is_empty() {
        push_avatar_text(
            commands,
            node,
            &avatar_rect,
            clip,
            order + 1,
            foreground,
            opacity,
        );
    } else if let Some(icon) = avatar_icon_pixels(node, &avatar_rect, foreground) {
        let icon_rect = centered_child_rect(&avatar_rect, AVATAR_FALLBACK_SCALE);
        push_avatar_image(commands, icon, icon_rect, clip, order + 1, opacity);
    } else {
        push_avatar_fallback_glyph(commands, &avatar_rect, clip, order + 1, foreground, opacity);
    }

    if let Some(border_color) = avatar_border_color(node) {
        commands.push(HostPaintCommand::quad(
            avatar_rect,
            Some(clip.clone()),
            order + 2,
            None,
            Some(border_color),
            avatar_border_width(node),
            corner_radius,
            opacity,
        ));
    }

    true
}

fn is_avatar_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "avatar" | "Avatar" | "mui-avatar" | "MuiAvatar"
    ) || matches!(node.role.as_str(), "Avatar" | "MuiAvatar")
}

fn avatar_frame(rect: &FrameRect) -> FrameRect {
    let size = rect.width.min(rect.height).min(AVATAR_DEFAULT_EDGE).round();
    let size = size.max(1.0);
    FrameRect {
        x: rect.x.round(),
        y: (rect.y + (rect.height - size).max(0.0) * 0.5).round(),
        width: size,
        height: size,
    }
}

fn avatar_corner_radius(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    if component_variant_contains(node, "square") {
        return 0.0;
    }
    if component_variant_contains(node, "rounded") {
        let configured = node
            .corner_radius
            .max(node.button_style.element.corner_radius)
            .max(0.0);
        return if configured > 0.0 {
            configured
        } else {
            AVATAR_ROUNDED_RADIUS
        };
    }
    rect.width.min(rect.height) * 0.5
}

fn avatar_background_color(node: &TemplatePaneNodeData, color_default: bool) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if color_default || component_variant_contains(node, "colorDefault") {
            MUI_GREY_600
        } else {
            PALETTE.surface_selected
        }
    })
}

fn avatar_foreground_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref()).unwrap_or_else(|| {
        match first_non_empty(&[node.text_tone.as_str(), node.validation_level.as_str()]) {
            "primary" | "accent" => PALETTE.accent,
            "muted" | "secondary" => PALETTE.text_muted,
            "warning" => PALETTE.warning,
            "error" | "danger" => PALETTE.error,
            "success" => PALETTE.success,
            "info" => PALETTE.info,
            _ => PALETTE.text,
        }
    })
}

fn avatar_border_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    resolved_style_color(node.button_style.element.border_color.as_ref()).or_else(|| {
        (node.border_width > 0.0 || node.button_style.element.border_width > 0.0)
            .then_some(PALETTE.border)
    })
}

fn avatar_border_width(node: &TemplatePaneNodeData) -> f32 {
    node.button_style
        .element
        .border_width
        .max(node.border_width)
        .max(1.0)
}

fn avatar_image_pixels(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    corner_radius: f32,
) -> Option<HostPaintImagePixels> {
    if !node.has_preview_image && node.media_source.is_empty() {
        return None;
    }
    let (target_width, target_height) = raster_size_from_frame(rect.width, rect.height)?;
    let mut image = template_image_pixels(
        &node.preview_image,
        node.media_source.as_str(),
        "",
        target_width,
        target_height,
        None,
        true,
    )?;
    apply_rounded_alpha_mask(&mut image, corner_radius, rect);
    Some(image)
}

fn avatar_icon_pixels(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    foreground: [u8; 4],
) -> Option<HostPaintImagePixels> {
    if node.icon_name.is_empty() {
        return None;
    }
    let icon_rect = centered_child_rect(rect, AVATAR_FALLBACK_SCALE);
    let (target_width, target_height) = raster_size_from_frame(icon_rect.width, icon_rect.height)?;
    template_image_pixels(
        &node.preview_image,
        "",
        node.icon_name.as_str(),
        target_width,
        target_height,
        Some(foreground),
        false,
    )
}

fn push_avatar_image(
    commands: &mut Vec<HostPaintCommand>,
    image: HostPaintImagePixels,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::image_pixels(
        frame,
        Some(clip.clone()),
        order,
        image.resource_key,
        image.width,
        image.height,
        image.rgba,
        image.atlas,
        opacity,
    ));
}

fn push_avatar_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let label = avatar_label(node);
    if label.is_empty() {
        return;
    }
    let font_size = avatar_font_size(node, rect);
    let line_height = font_size;
    let text_width = (label.chars().count() as f32 * font_size * AVATAR_TEXT_WIDTH_RATIO)
        .min(rect.width)
        .max(1.0);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + (rect.width - text_width).max(0.0) * 0.5,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_width,
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label,
        color,
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_avatar_fallback_glyph(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    let head_size = (rect.width.min(rect.height) * 0.24).max(2.0);
    let head = FrameRect {
        x: rect.x + (rect.width - head_size) * 0.5,
        y: rect.y + rect.height * 0.24,
        width: head_size,
        height: head_size,
    };
    commands.push(HostPaintCommand::quad(
        head.clone(),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        head_size * 0.5,
        opacity,
    ));

    let body_width = rect.width * 0.52;
    let body_height = rect.height * 0.22;
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x + (rect.width - body_width) * 0.5,
            y: rect.y + rect.height * 0.55,
            width: body_width,
            height: body_height,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        body_height * 0.5,
        opacity,
    ));
}

fn avatar_label(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.options_text.as_str(),
    ])
    .trim()
    .to_string()
}

fn avatar_font_size(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        rect.width.min(rect.height) * AVATAR_TEXT_FONT_RATIO
    };
    requested.max(AVATAR_MIN_FONT_SIZE).min(rect.height)
}

fn centered_child_rect(rect: &FrameRect, scale: f32) -> FrameRect {
    let size = (rect.width.min(rect.height) * scale.clamp(0.0, 1.0)).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size) * 0.5,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
    }
}

fn apply_rounded_alpha_mask(
    image: &mut HostPaintImagePixels,
    corner_radius: f32,
    rect: &FrameRect,
) {
    if corner_radius <= 0.0 || image.width == 0 || image.height == 0 {
        return;
    }
    let display_edge = rect.width.min(rect.height).max(1.0);
    let mask_edge = image.width.min(image.height) as f32;
    let mask_radius = (corner_radius / display_edge * mask_edge).clamp(0.0, mask_edge * 0.5);
    if mask_radius <= 0.0 {
        return;
    }

    let width = image.width;
    let height = image.height;
    for y in 0..height {
        for x in 0..width {
            if rounded_mask_contains_pixel(x, y, width, height, mask_radius) {
                continue;
            }
            let offset = ((y as usize * width as usize) + x as usize) * 4 + 3;
            image.rgba[offset] = 0;
        }
    }
    image.resource_key = format!(
        "mui-avatar-mask:{}x{}:{:.3}:{}",
        image.width, image.height, mask_radius, image.resource_key
    );
}

fn rounded_mask_contains_pixel(x: u32, y: u32, width: u32, height: u32, radius: f32) -> bool {
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let right = width as f32;
    let bottom = height as f32;
    let radius = radius.min(right.min(bottom) * 0.5).max(0.0);
    if radius <= 0.0 {
        return px >= 0.0 && px < right && py >= 0.0 && py < bottom;
    }
    let center_x = clamp_to_ordered_range(px, radius, right - radius);
    let center_y = clamp_to_ordered_range(py, radius, bottom - radius);
    let dx = px - center_x;
    let dy = py - center_y;
    dx * dx + dy * dy <= radius * radius
}

fn clamp_to_ordered_range(value: f32, min: f32, max: f32) -> f32 {
    if min <= max {
        value.clamp(min, max)
    } else {
        (min + max) * 0.5
    }
}
