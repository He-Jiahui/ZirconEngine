use zircon_runtime_interface::ui::{dispatch::UiDispatchEffect, event_ui::UiNodeId};

pub(super) fn effect_target(effect: &UiDispatchEffect) -> Option<UiNodeId> {
    match effect {
        UiDispatchEffect::SetFocus { target, .. }
        | UiDispatchEffect::ClearFocus { target, .. }
        | UiDispatchEffect::CapturePointer { target, .. }
        | UiDispatchEffect::ReleasePointerCapture { target, .. }
        | UiDispatchEffect::LockPointer { target, .. }
        | UiDispatchEffect::UnlockPointer { target, .. }
        | UiDispatchEffect::UseHighPrecisionPointer { target, .. }
        | UiDispatchEffect::DragDrop { target, .. }
        | UiDispatchEffect::DirtyRedraw { target, .. }
        | UiDispatchEffect::EmitComponentEvent { target, .. } => Some(*target),
        UiDispatchEffect::RequestInputMethod { request } => Some(request.owner),
        UiDispatchEffect::RequestClipboard { request } => Some(request.owner),
        UiDispatchEffect::Popup { owner, .. } | UiDispatchEffect::Tooltip { owner, .. } => *owner,
        UiDispatchEffect::DismissTransientUi { .. }
        | UiDispatchEffect::RequestNavigation { .. } => None,
    }
}
