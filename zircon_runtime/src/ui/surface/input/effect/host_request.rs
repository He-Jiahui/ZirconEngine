use zircon_runtime_interface::ui::{
    dispatch::{UiDispatchEffect, UiDispatchHostRequest, UiDispatchHostRequestKind},
    event_ui::UiNodeId,
};

pub(super) fn host_request_for_effect(
    effect_index: usize,
    effect: &UiDispatchEffect,
    target: Option<UiNodeId>,
) -> Option<UiDispatchHostRequest> {
    let request = match effect {
        UiDispatchEffect::LockPointer { target, policy } => {
            UiDispatchHostRequestKind::PointerLock {
                target: *target,
                policy: *policy,
            }
        }
        UiDispatchEffect::UnlockPointer { policy, .. } => {
            UiDispatchHostRequestKind::PointerUnlock { policy: *policy }
        }
        UiDispatchEffect::UseHighPrecisionPointer { target, enabled } => {
            UiDispatchHostRequestKind::HighPrecisionPointer {
                target: *target,
                enabled: *enabled,
            }
        }
        UiDispatchEffect::Popup {
            kind,
            popup_id,
            owner: _,
            anchor,
        } => UiDispatchHostRequestKind::Popup {
            kind: *kind,
            popup_id: popup_id.clone(),
            anchor: *anchor,
        },
        UiDispatchEffect::Tooltip {
            kind,
            tooltip_id,
            owner: _,
        } => UiDispatchHostRequestKind::Tooltip {
            kind: *kind,
            tooltip_id: tooltip_id.clone(),
        },
        UiDispatchEffect::RequestInputMethod { request } => {
            UiDispatchHostRequestKind::InputMethod(request.clone())
        }
        UiDispatchEffect::RequestClipboard { request } => {
            UiDispatchHostRequestKind::Clipboard(request.clone())
        }
        _ => return None,
    };
    Some(UiDispatchHostRequest {
        effect_index,
        request,
        reason: target
            .map(|node_id| format!("effect applied for {node_id:?}"))
            .unwrap_or_else(|| "effect applied".to_string()),
    })
}
