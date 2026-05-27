use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::theme::PALETTE;
use super::{component_variant_contains, resolved_style_color};

const MUI_FIELD_FILLED_BACKGROUND: [u8; 4] = [255, 255, 255, 23];
const MUI_FIELD_FILLED_HOVER_BACKGROUND: [u8; 4] = [255, 255, 255, 31];
const MUI_FIELD_STANDARD_UNDERLINE: f32 = 1.0;
const MUI_FIELD_ACTIVE_UNDERLINE: f32 = 2.0;
const MUI_FIELD_OUTLINED_RADIUS: f32 = 4.0;
const MUI_FIELD_FILLED_RADIUS: f32 = 4.0;

pub(super) fn push_text_field_surface_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_text_field_node(node) {
        return false;
    }

    let field_rect = pixel_aligned_rect(rect);
    if field_rect.width <= 0.0 || field_rect.height <= 0.0 {
        return true;
    }

    match text_field_variant(node) {
        TextFieldVariant::Filled => {
            push_filled_field(commands, node, &field_rect, clip, order, opacity)
        }
        TextFieldVariant::Standard => {
            push_standard_field(commands, node, &field_rect, clip, order, opacity)
        }
        TextFieldVariant::Outlined => {
            push_outlined_field(commands, node, &field_rect, clip, order, opacity)
        }
    }
    true
}

fn is_text_field_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "input-field" | "number-field" | "text-field" | "mui-text-field" | "MuiTextField"
    ) || matches!(
        node.role.as_str(),
        "InputField" | "TextField" | "MuiTextField"
    )
}

fn push_outlined_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        resolved_style_color(node.button_style.element.background_color.as_ref()),
        Some(field_stroke_color(node)),
        field_stroke_width(node),
        configured_radius(node, MUI_FIELD_OUTLINED_RADIUS),
        opacity,
    ));
}

fn push_filled_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(field_fill_color(node)),
        None,
        0.0,
        configured_radius(node, MUI_FIELD_FILLED_RADIUS),
        opacity,
    ));
    push_underline(commands, node, rect, clip, order + 1, opacity);
}

fn push_standard_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    push_underline(commands, node, rect, clip, order, opacity);
}

fn push_underline(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let thickness = field_stroke_width(node)
        .max(MUI_FIELD_STANDARD_UNDERLINE)
        .min(rect.height.max(1.0));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height - thickness,
            width: rect.width,
            height: thickness,
        },
        Some(clip.clone()),
        order,
        Some(field_stroke_color(node)),
        None,
        0.0,
        0.0,
        opacity,
    ));
}

fn text_field_variant(node: &TemplatePaneNodeData) -> TextFieldVariant {
    if component_variant_contains(node, "filled") {
        TextFieldVariant::Filled
    } else if component_variant_contains(node, "standard") {
        TextFieldVariant::Standard
    } else {
        TextFieldVariant::Outlined
    }
}

#[derive(Clone, Copy)]
enum TextFieldVariant {
    Outlined,
    Filled,
    Standard,
}

fn field_stroke_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.border_disabled;
    }
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || component_variant_contains(node, "error")
    {
        return PALETTE.error;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.border_color.as_ref()) {
        return color;
    }
    if node.focused || component_variant_contains(node, "focused") {
        return PALETTE.focus_ring;
    }
    PALETTE.border
}

fn field_stroke_width(node: &TemplatePaneNodeData) -> f32 {
    let configured = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if node.focused
        || component_variant_contains(node, "focused")
        || matches!(node.validation_level.as_str(), "error" | "danger")
        || component_variant_contains(node, "error")
    {
        configured.max(MUI_FIELD_ACTIVE_UNDERLINE)
    } else {
        configured.max(MUI_FIELD_STANDARD_UNDERLINE)
    }
}

fn field_fill_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if node.disabled {
        return PALETTE.surface_disabled;
    }
    resolved_style_color(node.button_style.element.background_color.as_ref()).unwrap_or_else(|| {
        if node.hovered {
            MUI_FIELD_FILLED_HOVER_BACKGROUND
        } else {
            MUI_FIELD_FILLED_BACKGROUND
        }
    })
}

fn configured_radius(node: &TemplatePaneNodeData, fallback: f32) -> f32 {
    let configured = node
        .corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0);
    if configured > 0.0 {
        configured
    } else {
        fallback
    }
}

fn pixel_aligned_rect(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x.round(),
        y: rect.y.round(),
        width: rect.width.round().max(0.0),
        height: rect.height.round().max(0.0),
    }
}
