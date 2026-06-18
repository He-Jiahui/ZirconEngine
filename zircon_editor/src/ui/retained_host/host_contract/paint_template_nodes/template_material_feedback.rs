use std::f32::consts::PI;

use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::material_primitives::component_variant_contains;
use super::render_commands::HostPaintCommand;
use super::template_style::template_corner_radius;
use super::template_style_color::resolved_style_color;
use super::visual_assets::raster_size_from_frame;

const MATERIAL_PROGRESS_TRACK: [u8; 4] = [42, 52, 60, 255];
const MUI_BACKDROP_SCRIM: [u8; 4] = [0, 0, 0, 128];

pub(super) fn push_material_feedback_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if is_material_backdrop_node(node) {
        push_material_backdrop_commands(commands, node, rect, clip, order, opacity);
        return true;
    }
    if is_material_progress_node(node) {
        push_material_progress_commands(commands, node, rect, clip, order, opacity);
        return true;
    }
    false
}

fn push_material_backdrop_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if !node.popup_open
        && node.surface_variant.as_str() != "backdrop"
        && !component_variant_contains(node, "open")
    {
        return;
    }
    if component_variant_contains(node, "invisible") {
        return;
    }
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(
            resolved_style_color(node.button_style.element.background_color.as_ref())
                .unwrap_or(MUI_BACKDROP_SCRIM),
        ),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn push_material_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if progress_is_circular(node) {
        push_circular_progress_command(commands, node, rect, clip, order, opacity);
    } else {
        push_linear_progress_commands(commands, node, rect, clip, order, opacity);
    }
}

fn push_linear_progress_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let radius = template_corner_radius(node)
        .max((rect.height * 0.5).min(2.0))
        .max(0.0);
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(progress_track_color(node)),
        None,
        0.0,
        radius,
        opacity,
    ));

    let fill = progress_fill_color(node);
    if progress_is_indeterminate(node) {
        for (x_factor, width_factor) in [(0.12, 0.36), (0.62, 0.24)] {
            let bar = FrameRect {
                x: rect.x + rect.width * x_factor,
                y: rect.y,
                width: (rect.width * width_factor).max(1.0),
                height: rect.height,
            };
            commands.push(HostPaintCommand::quad(
                bar,
                Some(clip.clone()),
                order + 1,
                Some(fill),
                None,
                0.0,
                radius,
                opacity,
            ));
        }
        return;
    }

    let width = rect.width * progress_percent(node);
    if width <= 0.0 {
        return;
    }
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: width.max(1.0),
            height: rect.height,
        },
        Some(clip.clone()),
        order + 1,
        Some(fill),
        None,
        0.0,
        radius,
        opacity,
    ));
}

fn push_circular_progress_command(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let image_rect = circular_progress_rect(rect);
    let Some((width, height)) = raster_size_from_frame(image_rect.width, image_rect.height) else {
        return;
    };
    let size = width.min(height);
    if size == 0 {
        return;
    }
    let rgba = circular_progress_pixels(
        size,
        if progress_is_indeterminate(node) {
            0.58
        } else {
            progress_percent(node)
        },
        progress_track_color(node),
        progress_fill_color(node),
    );
    commands.push(HostPaintCommand::image_pixels(
        image_rect,
        Some(clip.clone()),
        order,
        format!(
            "mui-circular-progress:{size}:{:.3}:{}:{}",
            progress_percent(node),
            progress_track_color(node)[0],
            progress_fill_color(node)[0]
        ),
        size,
        size,
        rgba,
        None,
        opacity,
    ));
}

fn is_material_progress_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "progress" | "progress-bar" | "linear-progress" | "circular-progress" | "spinner"
    ) || matches!(
        node.role.as_str(),
        "Progress" | "ProgressBar" | "LinearProgress" | "CircularProgress" | "Spinner"
    )
}

fn is_material_backdrop_node(node: &TemplatePaneNodeData) -> bool {
    node.component_role.as_str() == "backdrop"
        || node.role.as_str() == "Backdrop"
        || node.surface_variant.as_str() == "backdrop"
}

fn progress_is_circular(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "circular-progress" | "spinner"
    ) || matches!(node.role.as_str(), "CircularProgress" | "Spinner")
        || component_variant_contains(node, "circular")
}

fn progress_is_indeterminate(node: &TemplatePaneNodeData) -> bool {
    matches!(node.component_role.as_str(), "spinner")
        || component_variant_contains(node, "indeterminate")
}

fn progress_percent(node: &TemplatePaneNodeData) -> f32 {
    if node.value_percent.is_finite() {
        node.value_percent.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn progress_track_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref())
        .unwrap_or(MATERIAL_PROGRESS_TRACK)
}

fn progress_fill_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.text_disabled;
    }
    resolved_style_color(node.button_style.element.foreground_color.as_ref())
        .or_else(|| material_tone_color(node))
        .unwrap_or(PALETTE.accent)
}

fn material_tone_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    let tone = [node.validation_level.as_str(), node.text_tone.as_str()]
        .into_iter()
        .find(|value| !value.trim().is_empty())
        .unwrap_or("");
    match tone {
        "warning" => Some(PALETTE.warning),
        "error" | "danger" => Some(PALETTE.error),
        "success" => Some(PALETTE.success),
        "info" => Some(PALETTE.info),
        "accent" | "primary" => Some(PALETTE.accent),
        _ => None,
    }
}

fn circular_progress_rect(rect: &FrameRect) -> FrameRect {
    let size = rect.width.min(rect.height).max(1.0);
    FrameRect {
        x: rect.x + (rect.width - size) * 0.5,
        y: rect.y + (rect.height - size) * 0.5,
        width: size,
        height: size,
    }
}

fn circular_progress_pixels(size: u32, percent: f32, track: [u8; 4], fill: [u8; 4]) -> Vec<u8> {
    let mut rgba = vec![0; size as usize * size as usize * 4];
    let center = size as f32 * 0.5;
    let radius = (size as f32 * 0.5 - 0.5).max(1.0);
    let thickness = (size as f32 * 0.16).clamp(3.0, 6.0);
    let inner = (radius - thickness).max(0.0);
    let percent = percent.clamp(0.0, 1.0);
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 + 0.5 - center;
            let dy = y as f32 + 0.5 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance < inner || distance > radius {
                continue;
            }
            let angle = dy.atan2(dx);
            let turn = ((angle + PI * 0.5).rem_euclid(PI * 2.0)) / (PI * 2.0);
            let color = if turn <= percent { fill } else { track };
            let offset = ((y as usize * size as usize) + x as usize) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
    rgba
}
