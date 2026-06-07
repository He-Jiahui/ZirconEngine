use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchEffect, UiPopupEffectKind, UiTooltipEffectKind},
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::super::require_valid_input_owner;

pub(super) fn apply_popup_tooltip_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> Result<Option<UiNodeId>, String> {
    match effect {
        UiDispatchEffect::Popup {
            kind,
            popup_id,
            owner,
            anchor,
        } => apply_popup_effect(surface, *kind, popup_id, *owner, *anchor),
        UiDispatchEffect::Tooltip {
            kind,
            tooltip_id,
            owner,
        } => apply_tooltip_effect(surface, *kind, tooltip_id, *owner),
        _ => Err("expected popup or tooltip effect".to_string()),
    }
}

fn apply_popup_effect(
    surface: &mut UiSurface,
    kind: UiPopupEffectKind,
    popup_id: &str,
    owner: Option<UiNodeId>,
    anchor: Option<zircon_runtime_interface::ui::layout::UiPoint>,
) -> Result<Option<UiNodeId>, String> {
    if let Some(owner) = owner {
        require_valid_input_owner(surface, owner)?;
    }
    let route_owner = owner.or_else(|| surface.input.popup_owner(popup_id));
    match kind {
        UiPopupEffectKind::Open => {
            surface
                .input
                .open_popup(popup_id.to_string(), route_owner, anchor);
        }
        UiPopupEffectKind::Close => {
            surface.input.close_popup(popup_id);
        }
        UiPopupEffectKind::Toggle => {
            surface
                .input
                .toggle_popup(popup_id.to_string(), route_owner, anchor);
        }
    }
    Ok(route_owner)
}

fn apply_tooltip_effect(
    surface: &mut UiSurface,
    kind: UiTooltipEffectKind,
    tooltip_id: &str,
    owner: Option<UiNodeId>,
) -> Result<Option<UiNodeId>, String> {
    if let Some(owner) = owner {
        require_valid_input_owner(surface, owner)?;
    }
    let route_owner = owner.or_else(|| surface.input.tooltip_owner(tooltip_id));
    match kind {
        UiTooltipEffectKind::Arm => {
            surface
                .input
                .arm_tooltip(tooltip_id.to_string(), route_owner);
        }
        UiTooltipEffectKind::Show => {
            surface
                .input
                .show_tooltip(tooltip_id.to_string(), route_owner);
        }
        UiTooltipEffectKind::Hide | UiTooltipEffectKind::Cancel => {
            surface.input.clear_tooltip(tooltip_id);
        }
    }
    Ok(route_owner)
}
