use crate::ui::surface::UiSurface;
use zircon_runtime_interface::ui::{
    dispatch::{
        UiDispatchEffect, UiDispatchHostRequestKind, UiDispatchReply, UiInputEvent,
        UiInputEventMetadata, UiInputSequence, UiInputTimestamp, UiKeyboardInputEvent,
        UiKeyboardInputState, UiPopupEffectKind, UiTooltipEffectKind,
    },
    event_ui::UiTreeId,
    layout::UiPoint,
};

#[test]
fn popup_effects_update_shared_popup_stack_and_host_requests() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.popup.state"));
    let open = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Open,
            popup_id: "menu.file".to_string(),
            anchor: Some(UiPoint::new(10.0, 20.0)),
        }),
    );

    assert_eq!(surface.input.popup_stack.len(), 1);
    assert_eq!(surface.input.popup_stack[0].popup_id, "menu.file");
    assert_eq!(
        surface.input.popup_stack[0].anchor,
        Some(UiPoint::new(10.0, 20.0))
    );
    assert!(matches!(
        open.host_requests[0].request,
        UiDispatchHostRequestKind::Popup { .. }
    ));

    surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Popup {
            kind: UiPopupEffectKind::Toggle,
            popup_id: "menu.file".to_string(),
            anchor: Some(UiPoint::new(10.0, 20.0)),
        }),
    );

    assert!(surface.input.popup_stack.is_empty());
}

#[test]
fn tooltip_effects_track_pending_visible_and_canceled_state() {
    let mut surface = UiSurface::new(UiTreeId::new("runtime.ui.tooltip.state"));

    surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Arm,
            tooltip_id: "status.hint".to_string(),
        }),
    );
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| tooltip.visible),
        Some(false)
    );

    let shown = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Show,
            tooltip_id: "status.hint".to_string(),
        }),
    );
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| tooltip.visible),
        Some(true)
    );
    assert!(matches!(
        shown.host_requests[0].request,
        UiDispatchHostRequestKind::Tooltip { .. }
    ));

    surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(UiDispatchEffect::Tooltip {
            kind: UiTooltipEffectKind::Cancel,
            tooltip_id: "status.hint".to_string(),
        }),
    );
    assert_eq!(surface.input.tooltip, None);
}

fn keyboard_event() -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata: UiInputEventMetadata::new(
            UiInputTimestamp::from_micros(10),
            UiInputSequence::new(1),
        ),
        state: UiKeyboardInputState::Pressed,
        key_code: 65,
        scan_code: Some(30),
        physical_key: "KeyA".to_string(),
        logical_key: "A".to_string(),
        text: Some("a".to_string()),
    })
}
