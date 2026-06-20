use super::super::state::{button_interaction_state, is_button_disabled};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style_color::{
    is_primary_contained_button, resolved_style_color, typed_button_variant_background,
    MUI_SNACKBAR_BG, MUI_TOOLTIP_BG,
};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::{ButtonInteractionState, ButtonVariant};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn surface_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
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

fn is_explicit_text_button(node: &TemplatePaneNodeData) -> bool {
    matches!(node.button_variant.as_str(), "default" | "text")
        || (!node.button_variant.is_empty()
            && node.button_style.variant.normalized() == ButtonVariant::Text)
}
