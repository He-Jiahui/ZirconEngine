use zircon_runtime_interface::ui::{
    dispatch::UiDispatchEffect,
    event_ui::UiNodeId,
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
};

use crate::ui::tree::UiRuntimeTreeFocusExt;

use super::super::super::surface::UiSurface;

pub(super) fn apply_navigation_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> Result<Option<UiNodeId>, String> {
    match effect {
        UiDispatchEffect::RequestNavigation { kind, .. } => {
            let route = surface
                .route_navigation_event(*kind)
                .map_err(|error| format!("navigation route rejected: {error}"))?;
            let target = surface
                .tree
                .next_navigation_target(route.target, *kind)
                .map_err(|error| format!("navigation target rejected: {error}"))?;
            if let Some(target) = target {
                surface
                    .focus_node_with_reason(
                        target,
                        UiFocusChangeReason::Navigation,
                        UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
                    )
                    .map_err(|error| format!("navigation focus rejected: {error}"))?;
                Ok(Some(target))
            } else {
                Ok(route.target)
            }
        }
        _ => Err("expected navigation effect".to_string()),
    }
}
