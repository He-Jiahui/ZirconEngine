use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchEffect, UiFocusEffectReason},
    event_ui::UiNodeId,
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
};

use super::super::super::surface::UiSurface;
use super::super::{
    require_valid_input_owner, UiSurfaceInputEffectError, UiSurfaceInputEffectResult,
};

pub(super) fn apply_focus_pointer_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> UiSurfaceInputEffectResult<Option<UiNodeId>> {
    match effect {
        UiDispatchEffect::SetFocus { target, reason } => {
            let (change_reason, visible) = focus_effect_reasons(*reason);
            surface
                .focus_node_with_reason(*target, change_reason, visible)
                .map_err(|source| UiSurfaceInputEffectError::FocusRejected { source })?;
            Ok(Some(*target))
        }
        UiDispatchEffect::ClearFocus { target, reason } => {
            if surface.focus.focused != Some(*target) {
                return Err(UiSurfaceInputEffectError::FocusOwnerMismatch);
            }
            surface.clear_focus_with_reason(clear_focus_effect_reason(*reason));
            Ok(Some(*target))
        }
        UiDispatchEffect::CapturePointer {
            target, pointer_id, ..
        } => {
            require_valid_input_owner(surface, *target)?;
            if let Some(previous) = surface.focus.captured.filter(|owner| owner != target) {
                surface.input.clear_high_precision_for(previous);
            }
            surface.focus.captured = Some(*target);
            surface
                .input
                .set_pointer_capture_for_id(*pointer_id, *target);
            Ok(Some(*target))
        }
        UiDispatchEffect::ReleasePointerCapture {
            target, pointer_id, ..
        } => {
            if !pointer_capture_release_matches(surface, *pointer_id, *target) {
                return Err(UiSurfaceInputEffectError::PointerCaptureOwnerMismatch);
            }
            if surface
                .input
                .clear_pointer_capture_id_for_owner(*pointer_id, *target)
            {
                if surface.focus.captured == Some(*target)
                    && !surface.input.has_pointer_capture_for_owner(*target)
                {
                    surface.focus.captured = surface.input.activate_any_pointer_capture();
                }
                Ok(None)
            } else {
                Err(UiSurfaceInputEffectError::PointerCaptureOwnerMismatch)
            }
        }
        UiDispatchEffect::LockPointer { target, policy } => {
            require_valid_input_owner(surface, *target)?;
            surface.input.pointer_lock_owner = Some(*target);
            surface.input.pointer_lock_policy = Some(*policy);
            Ok(Some(*target))
        }
        UiDispatchEffect::UnlockPointer { target, .. } => {
            if surface.input.pointer_lock_owner == Some(*target) {
                surface.input.pointer_lock_owner = None;
                surface.input.pointer_lock_policy = None;
                Ok(Some(*target))
            } else {
                Err(UiSurfaceInputEffectError::PointerLockOwnerMismatch)
            }
        }
        UiDispatchEffect::UseHighPrecisionPointer { target, enabled } => {
            require_valid_input_owner(surface, *target)?;
            if *enabled {
                if surface.focus.captured != Some(*target)
                    || !surface.input.has_pointer_capture_for_owner(*target)
                {
                    return Err(UiSurfaceInputEffectError::HighPrecisionRequiresPointerCapture);
                }
                surface.input.high_precision_owner = Some(*target);
            } else if surface.input.high_precision_owner == Some(*target) {
                surface.input.high_precision_owner = None;
            } else {
                return Err(UiSurfaceInputEffectError::HighPrecisionOwnerMismatch);
            }
            Ok(Some(*target))
        }
        _ => Err(UiSurfaceInputEffectError::UnexpectedEffect {
            expected: "focus or pointer ownership",
        }),
    }
}

fn pointer_capture_release_matches(
    surface: &UiSurface,
    pointer_id: zircon_runtime_interface::ui::dispatch::UiPointerId,
    target: UiNodeId,
) -> bool {
    surface.input.pointer_capture_owner(pointer_id) == Some(target)
}

fn focus_effect_reasons(reason: UiFocusEffectReason) -> (UiFocusChangeReason, UiFocusVisible) {
    match reason {
        UiFocusEffectReason::Input => (
            UiFocusChangeReason::Input,
            UiFocusVisible::hidden(UiFocusVisibleReason::PointerInteraction),
        ),
        UiFocusEffectReason::Navigation => (
            UiFocusChangeReason::Navigation,
            UiFocusVisible::visible(UiFocusVisibleReason::KeyboardNavigation),
        ),
        UiFocusEffectReason::Programmatic | UiFocusEffectReason::Dismissal => (
            UiFocusChangeReason::Programmatic,
            UiFocusVisible::hidden(UiFocusVisibleReason::Programmatic),
        ),
    }
}

fn clear_focus_effect_reason(reason: UiFocusEffectReason) -> UiFocusChangeReason {
    match reason {
        UiFocusEffectReason::Input => UiFocusChangeReason::Input,
        UiFocusEffectReason::Navigation => UiFocusChangeReason::Navigation,
        UiFocusEffectReason::Programmatic | UiFocusEffectReason::Dismissal => {
            UiFocusChangeReason::Clear
        }
    }
}
