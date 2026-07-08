use super::super::state::{button_interaction_state, is_button_disabled};
use super::super::surface_roles::{
    is_asset_preview_surface, is_asset_thumbnail_card_surface,
    is_asset_thumbnail_name_area_surface, is_content_panel_surface,
};
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::host_contract::paint_template_nodes::template_style_color::{
    resolved_style_color, typed_button_tone_color,
};
use crate::ui::retained_host::host_contract::paint_theme::PALETTE;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn border_color(
    node: &TemplatePaneNodeData,
) -> [u8; 4] {
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
    if asset_thumbnail_card_uses_selected_border(node) {
        return PALETTE.accent;
    }
    if asset_thumbnail_name_area_uses_muted_interaction_border(node)
        || asset_preview_uses_muted_interaction_border(node)
        || content_panel_uses_muted_border(node)
    {
        return PALETTE.border;
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

fn asset_thumbnail_card_uses_selected_border(node: &TemplatePaneNodeData) -> bool {
    is_asset_thumbnail_card_surface(node) && (node.selected || node.checked)
}

fn asset_preview_uses_muted_interaction_border(node: &TemplatePaneNodeData) -> bool {
    is_asset_preview_surface(node)
        && (node.selected
            || node.checked
            || matches!(
                button_interaction_state(node),
                ButtonInteractionState::Hover
                    | ButtonInteractionState::Pressed
                    | ButtonInteractionState::Focused
            ))
}

fn asset_thumbnail_name_area_uses_muted_interaction_border(node: &TemplatePaneNodeData) -> bool {
    is_asset_thumbnail_name_area_surface(node)
        && (node.selected
            || node.checked
            || matches!(
                button_interaction_state(node),
                ButtonInteractionState::Hover
                    | ButtonInteractionState::Pressed
                    | ButtonInteractionState::Focused
            ))
}

fn content_panel_uses_muted_border(node: &TemplatePaneNodeData) -> bool {
    is_content_panel_surface(node)
        && (node.selected
            || node.checked
            || matches!(
                button_interaction_state(node),
                ButtonInteractionState::Hover
                    | ButtonInteractionState::Pressed
                    | ButtonInteractionState::Focused
            ))
}
