use super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::paint_theme::PALETTE;
use super::style_selector::resolved_state_for_node;
use super::template_style_color::{
    is_primary_contained_button, resolved_style_color, typed_button_tone_color,
    typed_button_variant_background, MUI_ON_DARK, MUI_SNACKBAR_BG, MUI_TOOLTIP_BG,
};
use zircon_runtime_interface::ui::style::{ButtonInteractionState, ButtonVariant};

const MATERIAL_ELEVATION_SHADOW_OFFSET: f32 = 2.0;

pub(super) fn template_border_width(node: &TemplatePaneNodeData) -> f32 {
    let width = node
        .border_width
        .max(node.button_style.element.border_width)
        .max(0.0);
    if matches!(
        button_interaction_state(node),
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused
    ) || node.selected
        || node.checked
    {
        width.max(2.0)
    } else {
        width
    }
}

pub(super) fn template_corner_radius(node: &TemplatePaneNodeData) -> f32 {
    node.corner_radius
        .max(node.button_style.element.corner_radius)
        .max(0.0)
}

pub(super) fn draws_elevation_shadow(node: &TemplatePaneNodeData) -> bool {
    node.elevation > 0.0 && !is_button_disabled(node)
}

pub(super) fn elevation_shadow_rect(rect: &FrameRect, elevation: f32) -> FrameRect {
    let offset = elevation.max(1.0) * MATERIAL_ELEVATION_SHADOW_OFFSET;
    FrameRect {
        x: rect.x + offset,
        y: rect.y + offset,
        width: rect.width,
        height: rect.height,
    }
}

pub(super) fn surface_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.surface_disabled;
    }
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || matches!(node.surface_variant.as_str(), "danger" | "error")
    {
        return PALETTE.error_container;
    }
    if node.validation_level.as_str() == "warning" {
        return PALETTE.warning_container;
    }
    if node.validation_level.as_str() == "success" || node.surface_variant.as_str() == "success" {
        return PALETTE.success_container;
    }
    if node.validation_level.as_str() == "info" || node.surface_variant.as_str() == "info" {
        return PALETTE.info_container;
    }
    match button_interaction_state(node) {
        ButtonInteractionState::Pressed => return PALETTE.surface_pressed,
        ButtonInteractionState::Focused => return PALETTE.surface_selected,
        ButtonInteractionState::Hover => {
            return if is_primary_contained_button(node) {
                PALETTE.accent_soft
            } else {
                PALETTE.surface_hover
            };
        }
        ButtonInteractionState::Disabled => return PALETTE.surface_disabled,
        ButtonInteractionState::Loading | ButtonInteractionState::Normal => {}
    }
    if let Some(color) = resolved_style_color(node.button_style.element.background_color.as_ref()) {
        return color;
    }
    if let Some(color) = typed_button_variant_background(node) {
        return color;
    }
    match node.surface_variant.as_str() {
        "tooltip" => return MUI_TOOLTIP_BG,
        "snackbar" => return MUI_SNACKBAR_BG,
        "paper" | "paper-outlined" | "dialog" | "popover" => return PALETTE.popup,
        _ => {}
    }
    if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
    {
        return PALETTE.accent;
    }
    match node.surface_variant.as_str() {
        "inset" | "scroll-body" | "asset-tree-row" | "reference-row" => PALETTE.surface_inset,
        "popup" | "elevated" => PALETTE.popup,
        "panel" | "asset-preview" | "asset-preview-visual" => PALETTE.surface,
        "shell" => PALETTE.shell_background,
        _ => match node.role.as_str() {
            "Button" if node.surface_variant.is_empty() && is_explicit_text_button(node) => {
                [0, 0, 0, 0]
            }
            "Button" if node.surface_variant.is_empty() => PALETTE.surface_hover,
            _ => PALETTE.surface,
        },
    }
}

pub(super) fn border_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.border_disabled;
    }
    if matches!(node.validation_level.as_str(), "error" | "danger")
        || matches!(node.surface_variant.as_str(), "danger" | "error")
    {
        return PALETTE.error;
    }
    if node.validation_level.as_str() == "warning" {
        return PALETTE.warning;
    }
    if node.validation_level.as_str() == "success" || node.surface_variant.as_str() == "success" {
        return PALETTE.success;
    }
    if node.validation_level.as_str() == "info" || node.surface_variant.as_str() == "info" {
        return PALETTE.info;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.border_color.as_ref()) {
        return color;
    }
    if matches!(
        button_interaction_state(node),
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused
    ) || node.selected
        || node.checked
    {
        PALETTE.focus_ring
    } else if let Some(color) = typed_button_tone_color(node) {
        color
    } else if matches!(node.button_variant.as_str(), "primary" | "filled")
        || matches!(node.surface_variant.as_str(), "accent" | "primary")
        || matches!(
            button_interaction_state(node),
            ButtonInteractionState::Hover
        )
    {
        PALETTE.focus_ring
    } else {
        PALETTE.border
    }
}

pub(super) fn text_color(node: &TemplatePaneNodeData) -> [u8; 4] {
    if is_button_disabled(node) {
        return PALETTE.text_disabled;
    }
    if let Some(color) = resolved_style_color(node.button_style.element.foreground_color.as_ref()) {
        return color;
    }
    if is_primary_contained_button(node)
        && matches!(
            button_interaction_state(node),
            ButtonInteractionState::Normal | ButtonInteractionState::Hover
        )
    {
        return [8, 20, 22, 255];
    }
    match node.text_tone.as_str() {
        "inverse" | "on-dark" | "tooltip" | "snackbar" => MUI_ON_DARK,
        "muted" | "subtle" => PALETTE.text_muted,
        "accent" | "primary" | "default" => PALETTE.focus_ring,
        "warning" => PALETTE.warning,
        "error" | "danger" => PALETTE.error,
        "success" => PALETTE.success,
        "info" => PALETTE.info,
        _ => PALETTE.text,
    }
}

pub(super) fn is_mui_overlay_surface_node(node: &TemplatePaneNodeData) -> bool {
    matches!(
        node.component_role.as_str(),
        "paper"
            | "dialog"
            | "alert-dialog"
            | "popover"
            | "menu"
            | "tooltip"
            | "snackbar"
            | "drawer"
    )
}

pub(super) fn is_button_disabled(node: &TemplatePaneNodeData) -> bool {
    node.disabled
        || node.button_style.disabled
        || matches!(
            node.button_style.interaction_state,
            ButtonInteractionState::Disabled
        )
}

fn button_interaction_state(node: &TemplatePaneNodeData) -> ButtonInteractionState {
    resolved_state_for_node(node).button_interaction_state()
}

fn is_explicit_text_button(node: &TemplatePaneNodeData) -> bool {
    matches!(node.button_variant.as_str(), "default" | "text")
        || (!node.button_variant.is_empty()
            && node.button_style.variant.normalized() == ButtonVariant::Text)
}

#[cfg(test)]
#[path = "template_style_tests.rs"]
mod tests;
