use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::visual_assets::{
    raster_size_from_frame, template_image_pixels, HostPaintImagePixels,
};
use super::first_non_empty;
use geometry::{avatar_corner_radius, avatar_frame, avatar_text_frame, centered_child_rect};
use style::{
    avatar_background_color, avatar_border_color, avatar_border_width, avatar_foreground_color,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

mod geometry;
mod style;

const AVATAR_FALLBACK_SCALE: f32 = 0.75;

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
    let (frame, font_size, line_height) = avatar_text_frame(node, rect, &label);
    commands.push(HostPaintCommand::text(
        frame,
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
