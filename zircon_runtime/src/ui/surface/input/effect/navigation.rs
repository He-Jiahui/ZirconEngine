use zircon_runtime_interface::ui::{
    dispatch::UiDispatchEffect,
    event_ui::UiNodeId,
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
};

use super::super::super::surface::UiSurface;
use super::super::{UiSurfaceInputEffectError, UiSurfaceInputEffectResult};

pub(super) fn apply_navigation_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    match effect {
        UiDispatchEffect::RequestNavigation { kind, .. } => {
            let route = surface
                .route_navigation_event(*kind)
                .map_err(|source| UiSurfaceInputEffectError::NavigationRouteRejected { source })?;
            let target = surface
                .next_navigation_target(route.target, *kind)
                .map_err(|source| UiSurfaceInputEffectError::NavigationTargetRejected { source })?;
            if let Some(target) = target {
                surface
                    .focus_node_with_reason(
                        target,
                        UiFocusChangeReason::Navigation,
                        UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
                    )
                    .map_err(
                        |source| UiSurfaceInputEffectError::NavigationFocusRejected { source },
                    )?;
                Ok(Some(target))
            } else {
                Ok(route.target)
            }
        }
        _ => Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "navigation",
        }),
    }
}
