use super::super::super::super::super::data::TemplatePaneNodeData;
use super::super::super::super::super::paint_theme::PALETTE;
use super::super::super::super::template_style_color::is_primary_contained_button;
use super::super::super::state::button_interaction_state;
use super::super::super::surface_roles::{
    is_asset_preview_surface, is_asset_thumbnail_name_area_surface, is_content_panel_surface,
};
use zircon_runtime_interface::ui::style::ButtonInteractionState;

pub(super) fn interaction_surface_color(node: &TemplatePaneNodeData) -> Option<[u8; 4]> {
    if is_asset_thumbnail_name_area_surface(node) {
        return asset_thumbnail_name_area_interaction_surface_color(
            node,
            button_interaction_state(node),
        );
    }
    if is_asset_preview_surface(node) {
        return asset_preview_interaction_surface_color(button_interaction_state(node));
    }
    if is_content_panel_surface(node) {
        return content_panel_interaction_surface_color(button_interaction_state(node));
    }

    match button_interaction_state(node) {
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused => {
            Some(PALETTE.surface_pressed)
        }
        ButtonInteractionState::Hover => Some(if is_primary_contained_button(node) {
            PALETTE.accent_soft
        } else {
            PALETTE.surface_hover
        }),
        ButtonInteractionState::Disabled => Some(PALETTE.surface_disabled),
        ButtonInteractionState::Loading | ButtonInteractionState::Normal => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_generic_interaction_surface_uses_pressed_not_selected_surface() {
        let node = TemplatePaneNodeData {
            focused: true,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(
            interaction_surface_color(&node),
            Some(PALETTE.surface_pressed)
        );
        assert_ne!(
            interaction_surface_color(&node),
            Some(PALETTE.surface_selected)
        );
    }

    #[test]
    fn pressed_generic_interaction_surface_still_uses_pressed_surface() {
        let node = TemplatePaneNodeData {
            pressed: true,
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(
            interaction_surface_color(&node),
            Some(PALETTE.surface_pressed)
        );
    }

    #[test]
    fn selected_asset_thumbnail_name_area_still_uses_selected_surface() {
        let node = TemplatePaneNodeData {
            selected: true,
            surface_variant: "asset-thumbnail-name-area".into(),
            ..TemplatePaneNodeData::default()
        };

        assert_eq!(
            interaction_surface_color(&node),
            Some(PALETTE.surface_selected)
        );
    }
}

fn content_panel_interaction_surface_color(state: ButtonInteractionState) -> Option<[u8; 4]> {
    match state {
        ButtonInteractionState::Disabled => Some(PALETTE.surface_disabled),
        ButtonInteractionState::Loading
        | ButtonInteractionState::Pressed
        | ButtonInteractionState::Focused
        | ButtonInteractionState::Hover
        | ButtonInteractionState::Normal => None,
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

fn asset_thumbnail_name_area_interaction_surface_color(
    node: &TemplatePaneNodeData,
    state: ButtonInteractionState,
) -> Option<[u8; 4]> {
    match state {
        ButtonInteractionState::Disabled => Some(PALETTE.surface_disabled),
        _ if node.selected || node.checked => Some(PALETTE.surface_selected),
        ButtonInteractionState::Pressed | ButtonInteractionState::Focused => {
            Some(PALETTE.surface_pressed)
        }
        ButtonInteractionState::Hover => Some(PALETTE.surface_hover),
        ButtonInteractionState::Loading | ButtonInteractionState::Normal => None,
    }
}
