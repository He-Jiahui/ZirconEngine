use zircon_runtime_interface::ui::{dispatch::UiDispatchEffect, event_ui::UiNodeId};

use super::super::super::surface::UiSurface;
use super::super::{
    require_valid_input_owner, UiSurfaceInputEffectError, UiSurfaceInputEffectResult,
};

pub(super) fn apply_link_activation_effect(
    surface: &UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    let UiDispatchEffect::RequestLinkActivation { target, .. } = effect else {
        return Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "rich link activation",
        });
    };
    require_valid_input_owner(surface, *target)?;
    Ok(Some(*target))
}

#[cfg(test)]
#[path = "tests/link.rs"]
mod tests;
