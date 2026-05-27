use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::render_commands::HostPaintCommand;
use super::theme::PALETTE;
use zircon_runtime_interface::ui::style::UiStyleColor;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

mod alert;
mod avatar;
mod badge;
mod chip;
mod paper;
mod skeleton;
mod text_field;
mod timeline;

const DIVIDER_THICKNESS: f32 = 1.0;
const DIVIDER_MIDDLE_HORIZONTAL_INSET: f32 = 16.0;
const DIVIDER_INSET_HORIZONTAL_INSET: f32 = 72.0;
const DIVIDER_MIDDLE_VERTICAL_INSET: f32 = 8.0;
const DIVIDER_WRAPPER_HORIZONTAL_PADDING: f32 = 9.6;
const DIVIDER_WRAPPER_VERTICAL_PADDING: f32 = 9.6;
const DIVIDER_DEFAULT_FONT_SIZE: f32 = 12.0;
const DIVIDER_MIN_FONT_SIZE: f32 = 8.0;

pub(super) fn push_material_primitive_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if alert::push_alert_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if chip::push_chip_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if avatar::push_avatar_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if badge::push_badge_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if skeleton::push_skeleton_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if paper::push_paper_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    if timeline::push_timeline_primitive_commands(commands, node, rect, clip, order, opacity) {
        return true;
    }

    match material_primitive_kind(node) {
        Some(MaterialPrimitiveKind::Divider) => {
            push_divider(commands, node, rect, clip, order, opacity);
        }
        None => return false,
    }
    true
}

pub(super) fn push_material_text_field_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    text_field::push_text_field_surface_commands(commands, node, rect, clip, order, opacity)
}

enum MaterialPrimitiveKind {
    Divider,
}

fn material_primitive_kind(node: &TemplatePaneNodeData) -> Option<MaterialPrimitiveKind> {
    matches_any_role(
        node.component_role.as_str(),
        node.role.as_str(),
        &["divider", "Divider"],
    )
    .then_some(MaterialPrimitiveKind::Divider)
    .or_else(|| {
        (node.surface_variant.as_str() == "divider").then_some(MaterialPrimitiveKind::Divider)
    })
}

fn matches_any_role(component_role: &str, role: &str, expected: &[&str]) -> bool {
    expected
        .iter()
        .any(|candidate| component_role == *candidate || role == *candidate)
}

fn push_divider(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    // MUI Divider is border/pseudo-element geometry, not a filled panel.
    // Emit explicit line segments so inset, middle, and label gaps match the web contract.
    if divider_is_vertical(node, rect) {
        push_vertical_divider(commands, node, rect, clip, order, opacity);
    } else {
        push_horizontal_divider(commands, node, rect, clip, order, opacity);
    }
}

fn push_horizontal_divider(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (line_start, line_end) = horizontal_divider_extent(node, rect);
    let line_y = pixel_aligned(rect.y + (rect.height - DIVIDER_THICKNESS).max(0.0) * 0.5);
    let label = divider_label(node);
    if label.is_empty() {
        push_horizontal_line(
            commands, line_start, line_end, line_y, clip, order, node, opacity,
        );
        return;
    }

    let label_width = estimated_horizontal_label_width(node, &label, line_end - line_start);
    let label_left = horizontal_label_left(node, line_start, line_end, label_width);
    let label_right = (label_left + label_width).min(line_end);

    push_horizontal_line(
        commands, line_start, label_left, line_y, clip, order, node, opacity,
    );
    push_horizontal_line(
        commands,
        label_right,
        line_end,
        line_y,
        clip,
        order,
        node,
        opacity,
    );
    push_horizontal_divider_label(
        commands,
        node,
        &label,
        label_left,
        label_right,
        rect,
        clip,
        order + 1,
        opacity,
    );
}

fn push_vertical_divider(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (line_top, line_bottom) = vertical_divider_extent(node, rect);
    let line_x = pixel_aligned(rect.x + (rect.width - DIVIDER_THICKNESS).max(0.0) * 0.5);
    let label = divider_label(node);
    if label.is_empty() {
        push_vertical_line(
            commands,
            line_x,
            line_top,
            line_bottom,
            clip,
            order,
            node,
            opacity,
        );
        return;
    }

    let label_height = estimated_vertical_label_height(node, rect);
    let label_top = pixel_aligned(rect.y + (rect.height - label_height).max(0.0) * 0.5);
    let label_bottom = (label_top + label_height).min(line_bottom);

    push_vertical_line(
        commands, line_x, line_top, label_top, clip, order, node, opacity,
    );
    push_vertical_line(
        commands,
        line_x,
        label_bottom,
        line_bottom,
        clip,
        order,
        node,
        opacity,
    );
    push_vertical_divider_label(
        commands,
        node,
        &label,
        label_top,
        label_bottom,
        rect,
        clip,
        order + 1,
        opacity,
    );
}

fn push_horizontal_line(
    commands: &mut Vec<HostPaintCommand>,
    left: f32,
    right: f32,
    y: f32,
    clip: &FrameRect,
    order: i32,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    let width = right - left;
    if width <= 0.5 {
        return;
    }
    push_quad(
        commands,
        FrameRect {
            x: left,
            y,
            width,
            height: DIVIDER_THICKNESS,
        },
        clip,
        order,
        divider_color(node),
        opacity,
    );
}

fn push_vertical_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    top: f32,
    bottom: f32,
    clip: &FrameRect,
    order: i32,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    let height = bottom - top;
    if height <= 0.5 {
        return;
    }
    push_quad(
        commands,
        FrameRect {
            x,
            y: top,
            width: DIVIDER_THICKNESS,
            height,
        },
        clip,
        order,
        divider_color(node),
        opacity,
    );
}

fn push_horizontal_divider_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    label: &str,
    label_left: f32,
    label_right: f32,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if label.trim().is_empty() || label_right <= label_left {
        return;
    }
    let font_size = divider_font_size(node, rect.height);
    let line_height = font_size * 1.2;
    let text_left = label_left + DIVIDER_WRAPPER_HORIZONTAL_PADDING;
    let text_right = (label_right - DIVIDER_WRAPPER_HORIZONTAL_PADDING).max(text_left);
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: text_left,
            y: rect.y + (rect.height - line_height).max(0.0) * 0.5,
            width: text_right - text_left,
            height: line_height,
        },
        Some(clip.clone()),
        order,
        label.to_string(),
        divider_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_vertical_divider_label(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    label: &str,
    label_top: f32,
    label_bottom: f32,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    if label.trim().is_empty() || label_bottom <= label_top {
        return;
    }
    let font_size = divider_font_size(node, rect.height);
    let line_height = font_size * 1.2;
    let horizontal_padding = DIVIDER_WRAPPER_HORIZONTAL_PADDING.min(rect.width * 0.25);
    let text_top = label_top + DIVIDER_WRAPPER_VERTICAL_PADDING;
    commands.push(HostPaintCommand::text(
        FrameRect {
            x: rect.x + horizontal_padding,
            y: text_top,
            width: (rect.width - horizontal_padding * 2.0).max(1.0),
            height: (label_bottom - text_top).min(line_height).max(1.0),
        },
        Some(clip.clone()),
        order,
        label.to_string(),
        divider_text_color(node),
        font_size,
        line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

fn push_quad(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn horizontal_divider_extent(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let mut start = rect.x;
    let mut end = rect.x + rect.width;
    if component_variant_contains(node, "middle") {
        let inset = DIVIDER_MIDDLE_HORIZONTAL_INSET.min(rect.width * 0.45);
        start += inset;
        end -= inset;
    } else if component_variant_contains(node, "inset") {
        let inset = DIVIDER_INSET_HORIZONTAL_INSET.min(rect.width * 0.9);
        start += inset;
    }
    if end < start {
        let center = rect.x + rect.width * 0.5;
        (center, center)
    } else {
        (pixel_aligned(start), pixel_aligned(end))
    }
}

fn vertical_divider_extent(node: &TemplatePaneNodeData, rect: &FrameRect) -> (f32, f32) {
    let mut top = rect.y;
    let mut bottom = rect.y + rect.height;
    if component_variant_contains(node, "middle") {
        let inset = DIVIDER_MIDDLE_VERTICAL_INSET.min(rect.height * 0.45);
        top += inset;
        bottom -= inset;
    }
    if bottom < top {
        let center = rect.y + rect.height * 0.5;
        (center, center)
    } else {
        (pixel_aligned(top), pixel_aligned(bottom))
    }
}

fn horizontal_label_left(
    node: &TemplatePaneNodeData,
    line_start: f32,
    line_end: f32,
    label_width: f32,
) -> f32 {
    let available = (line_end - line_start).max(0.0);
    let remaining = (available - label_width).max(0.0);
    let ratio = match divider_text_align(node) {
        DividerTextAlign::Left => 0.1,
        DividerTextAlign::Center => 0.5,
        DividerTextAlign::Right => 0.9,
    };
    pixel_aligned(line_start + remaining * ratio)
}

fn estimated_horizontal_label_width(
    node: &TemplatePaneNodeData,
    label: &str,
    available_width: f32,
) -> f32 {
    let font_size = divider_font_size(node, available_width);
    let text_width = label.chars().count() as f32 * font_size * 0.56;
    (text_width + DIVIDER_WRAPPER_HORIZONTAL_PADDING * 2.0)
        .max(DIVIDER_WRAPPER_HORIZONTAL_PADDING * 2.0)
        .min(available_width.max(0.0))
}

fn estimated_vertical_label_height(node: &TemplatePaneNodeData, rect: &FrameRect) -> f32 {
    let font_size = divider_font_size(node, rect.height);
    (font_size * 1.2 + DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .max(DIVIDER_WRAPPER_VERTICAL_PADDING * 2.0)
        .min(rect.height.max(0.0))
}

fn divider_is_vertical(node: &TemplatePaneNodeData, rect: &FrameRect) -> bool {
    component_variant_contains(node, "vertical")
        || component_variant_contains(node, "wrapperVertical")
        || (!component_variant_contains(node, "horizontal") && rect.height > rect.width * 1.4)
}

fn divider_text_align(node: &TemplatePaneNodeData) -> DividerTextAlign {
    if component_variant_contains(node, "textAlignRight")
        || component_variant_contains(node, "right")
        || matches!(node.text_align.as_str(), "right" | "end")
    {
        DividerTextAlign::Right
    } else if component_variant_contains(node, "textAlignLeft")
        || component_variant_contains(node, "left")
        || matches!(node.text_align.as_str(), "left" | "start")
    {
        DividerTextAlign::Left
    } else {
        DividerTextAlign::Center
    }
}

enum DividerTextAlign {
    Left,
    Center,
    Right,
}

fn divider_label(node: &TemplatePaneNodeData) -> String {
    first_non_empty(&[
        node.text.as_str(),
        node.value_text.as_str(),
        node.options_text.as_str(),
    ])
    .to_string()
}

fn divider_font_size(node: &TemplatePaneNodeData, available_height: f32) -> f32 {
    let requested = if node.font_size.is_finite() && node.font_size > 0.0 {
        node.font_size
    } else {
        DIVIDER_DEFAULT_FONT_SIZE
    };
    requested
        .min((available_height * 0.82).max(DIVIDER_MIN_FONT_SIZE))
        .max(DIVIDER_MIN_FONT_SIZE)
}

fn divider_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
        return PALETTE.border_disabled;
    }
    resolved_style_color(node.button_style.element.border_color.as_ref())
        .or_else(|| resolved_style_color(node.button_style.element.foreground_color.as_ref()))
        .unwrap_or(PALETTE.border)
}

fn divider_text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled || node.validation_level.as_str() == "disabled" {
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

pub(super) fn resolved_style_color(color: Option<&UiStyleColor>) -> Option<[u8; 4]> {
    match color? {
        UiStyleColor::Rgba(color) => Some(color.to_u8()),
        UiStyleColor::Transparent => Some([0, 0, 0, 0]),
        UiStyleColor::Inherit => None,
        UiStyleColor::Role(role) => material_role_color(role),
    }
}

fn material_role_color(role: &str) -> Option<[u8; 4]> {
    match role {
        "primary" | "accent" | "material.primary" | "material_color_primary" => {
            Some(PALETTE.accent)
        }
        "surface" | "material.surface" => Some(PALETTE.surface),
        "surface_inset" | "material.surface_inset" => Some(PALETTE.surface_inset),
        "surface_hover" | "material.surface_hover" => Some(PALETTE.surface_hover),
        "surface_selected" | "material.surface_selected" => Some(PALETTE.surface_selected),
        "border" | "outline" | "material.outline" => Some(PALETTE.border),
        "focus" | "focus_ring" | "material.focus_ring" => Some(PALETTE.focus_ring),
        "text" | "on_surface" | "material.text" | "material.on_surface" => Some(PALETTE.text),
        "text_muted" | "muted" | "material.text_muted" => Some(PALETTE.text_muted),
        "text_disabled" | "material.text_disabled" => Some(PALETTE.text_disabled),
        "warning" | "material.warning" => Some(PALETTE.warning),
        "error" | "danger" | "material.error" => Some(PALETTE.error),
        "success" | "material.success" => Some(PALETTE.success),
        "info" | "material.info" => Some(PALETTE.info),
        _ => None,
    }
}

pub(super) fn component_variant_contains(node: &TemplatePaneNodeData, expected: &str) -> bool {
    node.component_variant
        .as_str()
        .split(|character: char| {
            character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
        })
        .any(|part| part.eq_ignore_ascii_case(expected))
}

pub(super) fn first_non_empty<'a>(values: &[&'a str]) -> &'a str {
    values
        .iter()
        .copied()
        .find(|value| !value.is_empty())
        .unwrap_or_default()
}

fn pixel_aligned(value: f32) -> f32 {
    value.round()
}
