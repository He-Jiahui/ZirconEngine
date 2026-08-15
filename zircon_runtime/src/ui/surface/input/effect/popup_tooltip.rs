use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiPopupEffectKind, UiTooltipEffectKind, UiTransientDismissalTarget,
    },
    event_ui::UiNodeId,
};

use super::super::super::surface::UiSurface;
use super::super::{
    require_valid_input_owner, UiSurfaceInputEffectError, UiSurfaceInputEffectResult,
};

pub(super) fn apply_popup_tooltip_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
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
        UiDispatchEffect::DismissTransientUi { target, .. } => {
            apply_transient_dismissal_effect(surface, *target)
        }
        _ => Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "popup or tooltip",
        }),
    }
}

fn apply_popup_effect(
    surface: &mut UiSurface,
    kind: UiPopupEffectKind,
    popup_id: &str,
    owner: Option<UiNodeId>,
    anchor: Option<zircon_runtime_interface::ui::layout::UiPoint>,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    if let Some(owner) = owner {
        require_valid_input_owner(surface, owner)?;
    }
    let fallback_route_owner = owner.or_else(|| surface.input.popup_owner(popup_id));
    let declarative_popup_node = surface
        .unique_popup_state_for_id(popup_id)
        .map(|(node_id, _, _)| node_id);
    match kind {
        UiPopupEffectKind::Open => {
            if !surface
                .set_declarative_popup_open_by_id(popup_id, true)
                .map_err(|source| UiSurfaceInputEffectError::PopupStateRejected { source })?
            {
                surface
                    .input
                    .open_popup(popup_id.to_string(), fallback_route_owner, anchor);
            }
        }
        UiPopupEffectKind::Close => {
            return surface
                .dismiss_popup_by_id(popup_id)
                .map_err(|source| UiSurfaceInputEffectError::PopupStateRejected { source });
        }
        UiPopupEffectKind::Toggle => {
            let declared_open = declarative_popup_node
                .and_then(|node_id| surface.popup_state_for_node(node_id))
                .map(|(_, open)| open);
            match declared_open {
                Some(open) => {
                    let _ = surface
                        .set_declarative_popup_open_by_id(popup_id, !open)
                        .map_err(|source| UiSurfaceInputEffectError::PopupStateRejected {
                            source,
                        })?;
                }
                None => {
                    surface
                        .input
                        .toggle_popup(popup_id.to_string(), fallback_route_owner, anchor)
                }
            }
        }
    }
    Ok(match declarative_popup_node {
        Some(node_id) => surface.popup_route_owner_for_node(node_id),
        None => fallback_route_owner,
    })
}

fn apply_transient_dismissal_effect(
    surface: &mut UiSurface,
    target: UiTransientDismissalTarget,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    surface
        .dismiss_transient_ui(target)
        .map_err(|source| UiSurfaceInputEffectError::PopupStateRejected { source })
}

fn apply_tooltip_effect(
    surface: &mut UiSurface,
    kind: UiTooltipEffectKind,
    tooltip_id: &str,
    owner: Option<UiNodeId>,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
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
