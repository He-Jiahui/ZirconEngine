use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchEffect, UiFocusEffectReason},
    event_ui::UiNodeId,
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
};

use super::super::super::surface::UiSurface;
use super::super::require_valid_input_owner;

pub(super) fn apply_focus_pointer_effect(
    surface: &mut UiSurface,
    effect: &UiDispatchEffect,
) -> Result<Option<UiNodeId>, String> {
    match effect {
        UiDispatchEffect::SetFocus { target, reason } => {
            let (change_reason, visible) = focus_effect_reasons(*reason);
            surface
                .focus_node_with_reason(*target, change_reason, visible)
                .map_err(|error| format!("focus rejected: {error}"))?;
            Ok(Some(*target))
        }
        UiDispatchEffect::ClearFocus { target, reason } => {
            if surface.focus.focused != Some(*target) {
                return Err("focus owner mismatch".to_string());
            }
            surface.clear_focus_with_reason(clear_focus_effect_reason(*reason));
            if surface.input.input_method_owner == Some(*target) {
                surface.input.clear_input_method();
            }
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
                return Err("pointer capture belongs to a different or unknown pointer".to_string());
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
                Err("pointer capture belongs to a different or unknown pointer".to_string())
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
                Err("pointer lock owner mismatch".to_string())
            }
        }
        UiDispatchEffect::UseHighPrecisionPointer { target, enabled } => {
            require_valid_input_owner(surface, *target)?;
            if *enabled {
                if surface.focus.captured != Some(*target)
                    || !surface
                        .input
                        .has_legacy_or_indexed_pointer_capture_for_owner(*target)
                {
                    return Err("high precision requires pointer capture".to_string());
                }
                surface.input.high_precision_owner = Some(*target);
            } else if surface.input.high_precision_owner == Some(*target) {
                surface.input.high_precision_owner = None;
            } else {
                return Err("high precision owner mismatch".to_string());
            }
            Ok(Some(*target))
        }
        _ => Err("expected focus or pointer ownership effect".to_string()),
    }
}

fn pointer_capture_release_matches(
    surface: &UiSurface,
    pointer_id: zircon_runtime_interface::ui::dispatch::UiPointerId,
    target: UiNodeId,
) -> bool {
    match surface.input.pointer_capture_owner(pointer_id) {
        Some(owner) => owner == target,
        None => {
            surface.input.pointer_captures.is_empty()
                && surface.input.captured_pointer_id == Some(pointer_id)
                && surface.focus.captured == Some(target)
        }
    }
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
            UiFocusVisible::visible(UiFocusVisibleReason::Programmatic),
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
