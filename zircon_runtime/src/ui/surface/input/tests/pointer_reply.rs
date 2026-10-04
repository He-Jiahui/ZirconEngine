use super::*;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchAppliedEffect, UiDispatchHostRequest, UiDispatchHostRequestKind,
        UiDispatchRejectedEffect, UiInputEvent, UiInputEventMetadata, UiInputSequence,
        UiInputTimestamp, UiPointerEvent, UiPointerInputEvent, UiPopupEffectKind,
    },
    layout::UiPoint,
    surface::{UiPointerButton, UiPointerEventKind},
    widget::{UiWidgetEvent, UiWidgetEventSource},
};

#[test]
fn merge_pointer_text_result_preserves_text_effect_statuses_when_rebasing_indexes() {
    let pointer_effect = release_pointer_effect(UiNodeId::new(1), UiPointerId::new(1));
    let mut result = UiInputDispatchResult::new(
        pointer_event(),
        UiDispatchReply::handled().with_effect(pointer_effect.clone()),
    );
    result.applied_effects.push(UiDispatchAppliedEffect {
        effect_index: 0,
        effect: pointer_effect.clone(),
    });

    let text_applied = focus_effect(UiNodeId::new(2));
    let text_rejected = popup_effect("stale-popup");
    let mut text_result = UiInputDispatchResult::new(
        pointer_event(),
        UiDispatchReply::handled().with_effects([text_applied.clone(), text_rejected.clone()]),
    );
    text_result.applied_effects.push(UiDispatchAppliedEffect {
        effect_index: 0,
        effect: text_applied.clone(),
    });
    text_result.rejected_effects.push(UiDispatchRejectedEffect {
        effect_index: 1,
        effect: text_rejected.clone(),
        reason: "invalid text popup owner".to_string(),
    });
    text_result.host_requests.push(UiDispatchHostRequest {
        effect_index: 1,
        request: UiDispatchHostRequestKind::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "stale-popup".to_string(),
            anchor: Some(UiPoint::new(10.0, 4.0)),
        },
        reason: "text popup".to_string(),
    });
    text_result.widget_events.push(UiWidgetEvent::Activate {
        target: UiNodeId::new(2),
        source: UiWidgetEventSource::Pointer,
        action_id: Some("text.activate".to_string()),
    });

    merge_pointer_text_result(&mut result, text_result);

    assert_eq!(
        result.reply.effects,
        vec![
            pointer_effect.clone(),
            text_applied.clone(),
            text_rejected.clone()
        ]
    );
    assert_eq!(
        result.applied_effects,
        vec![
            UiDispatchAppliedEffect {
                effect_index: 0,
                effect: pointer_effect,
            },
            UiDispatchAppliedEffect {
                effect_index: 1,
                effect: text_applied,
            },
        ]
    );
    assert_eq!(result.rejected_effects.len(), 1);
    assert_eq!(result.rejected_effects[0].effect_index, 2);
    assert_eq!(result.rejected_effects[0].effect, text_rejected);
    assert_eq!(
        result.rejected_effects[0].reason,
        "invalid text popup owner"
    );
    assert_eq!(result.host_requests.len(), 1);
    assert_eq!(result.host_requests[0].effect_index, 2);
    assert_eq!(result.widget_events.len(), 1);
    assert_eq!(
        result.widget_events[0],
        UiWidgetEvent::Activate {
            target: UiNodeId::new(2),
            source: UiWidgetEventSource::Pointer,
            action_id: Some("text.activate".to_string()),
        }
    );
}

fn pointer_event() -> UiInputEvent {
    UiInputEvent::Pointer(UiPointerInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(1),
            UiInputSequence::new(1),
        ),
        event: UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(0.0, 0.0))
            .with_button(UiPointerButton::Secondary),
        precise_scroll: None,
    })
}

fn release_pointer_effect(target: UiNodeId, pointer_id: UiPointerId) -> UiDispatchEffect {
    UiDispatchEffect::ReleasePointerCapture {
        target,
        pointer_id,
        reason: UiPointerCaptureReason::Cancel,
    }
}

fn focus_effect(target: UiNodeId) -> UiDispatchEffect {
    UiDispatchEffect::SetFocus {
        target,
        reason: UiFocusEffectReason::Input,
    }
}

fn popup_effect(popup_id: &str) -> UiDispatchEffect {
    UiDispatchEffect::Popup {
        kind: UiPopupEffectKind::Open,
        popup_id: popup_id.to_string(),
        owner: Some(UiNodeId::new(2)),
        anchor: Some(UiPoint::new(10.0, 4.0)),
    }
}
