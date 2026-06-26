use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::super::template_style_color::is_primary_contained_button;
use super::super::super::state::button_interaction_state;
use super::super::super::surface_roles::is_asset_preview_surface;
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn interaction_surface_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if is_asset_preview_surface(node) {
        return asset_preview_interaction_surface_color(button_interaction_state(node));
    }

    match button_interaction_state(node) {
        ButtonInteractionState::Pressed => Some(PALETTE.surface_pressed),
        ButtonInteractionState::Focused => Some(PALETTE.surface_selected),
        ButtonInteractionState::Hover => Some(if is_primary_contained_button(node) {
            PALETTE.accent_soft
        } else {
            PALETTE.surface_hover
        }),
        ButtonInteractionState::Disabled => Some(PALETTE.surface_disabled),
        ButtonInteractionState::Loading | ButtonInteractionState::Normal => None,
    }
}

fn asset_preview_interaction_surface_color(state: ButtonInteractionState) -> Option<[u8; 4]> {
    match state {
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused => {
            Some(PALETTE.surface_pressed)
        }
        ButtonInteractionState::Hover => Some(PALETTE.surface_hover),
        ButtonInteractionState::Disabled => Some(PALETTE.surface_disabled),
        ButtonInteractionState::Loading | ButtonInteractionState::Normal => None,
    }
}
