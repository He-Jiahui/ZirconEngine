use super::*;

#[test]
fn popup_and_tooltip_inputs_reject_stale_owner_without_mutating_shared_state() {
    let mut surface = two_button_surface();
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();
    let stale_owner = UiNodeId::new(2);
    surface
        .tree
        .nodes
        .get_mut(&stale_owner)
        .unwrap()
        .state_flags
        .enabled = false;

    let popup_open = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            popup_input_event_for_owner(
                UiPopupInputEventKind::OpenRequested,
                "menu.disabled",
                Some(stale_owner),
                Some(UiPoint::new(8.0, 12.0)),
            ),
        )
        .unwrap();

    assert_eq!(popup_open.rejected_effects.len(), 1);
    assert!(popup_open.rejected_effects[0]
        .reason
        .starts_with("invalid input owner"));
    assert!(popup_open.host_requests.is_empty());
    assert!(surface.input.popup_stack.is_empty());

    let tooltip_arm = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            tooltip_input_event_for_owner(
                UiTooltipTimerInputEventKind::Armed,
                "disabled.tooltip",
                Some(stale_owner),
            ),
        )
        .unwrap();

    assert_eq!(tooltip_arm.rejected_effects.len(), 1);
    assert!(tooltip_arm.rejected_effects[0]
        .reason
        .starts_with("invalid input owner"));
    assert!(tooltip_arm.host_requests.is_empty());
    assert_eq!(surface.input.tooltip, None);
}

#[test]
fn shared_input_dispatch_applies_drag_drop_popup_and_tooltip_events_through_effects() {
    let mut surface = two_button_surface();
    let pointer_dispatcher = UiPointerDispatcher::default();
    let navigation_dispatcher = UiNavigationDispatcher::default();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);
    let payload = UiDragPayload::new(UiDragPayloadKind::Asset, "res://assets/materials/brick.mat");

    let begin = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::Begin,
                Some(session_id),
                UiPoint::new(20.0, 20.0),
                Some(payload.clone()),
            ),
        )
        .unwrap();

    assert!(begin.rejected_effects.is_empty());
    assert_eq!(begin.applied_effects.len(), 1);
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_pointer_capture(&surface, pointer_id, UiNodeId::new(2));
    let drag = surface.input.drag_drop.as_ref().expect("active drag");
    assert_eq!(drag.session_id, session_id);
    assert_eq!(drag.source, UiNodeId::new(2));
    assert_eq!(drag.target, UiNodeId::new(2));
    assert_eq!(drag.point, Some(UiPoint::new(20.0, 20.0)));
    assert_eq!(drag.payload.as_deref(), Some(&payload));

    let over = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::Over,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
                None,
            ),
        )
        .unwrap();
    assert!(over.rejected_effects.is_empty());
    let drag = surface.input.drag_drop.as_ref().expect("updated drag");
    assert_eq!(drag.target, UiNodeId::new(3));
    assert_eq!(drag.point, Some(UiPoint::new(20.0, 60.0)));
    assert_eq!(drag.payload.as_deref(), Some(&payload));

    let drop = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::Drop,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
                None,
            ),
        )
        .unwrap();
    assert!(drop.rejected_effects.is_empty());
    assert!(
        surface
            .input
            .drag_drop
            .as_ref()
            .expect("accepted drag")
            .accepted
    );

    let end = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            drag_drop_input_event(
                UiDragDropInputEventKind::End,
                Some(session_id),
                UiPoint::new(20.0, 60.0),
                None,
            ),
        )
        .unwrap();
    assert!(end.rejected_effects.is_empty());
    assert_eq!(surface.input.drag_drop, None);
    assert_eq!(surface.focus.captured, None);
    assert_no_pointer_capture(&surface);

    let popup_open = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            popup_input_event(
                UiPopupInputEventKind::OpenRequested,
                "menu.file",
                Some(UiPoint::new(8.0, 12.0)),
            ),
        )
        .unwrap();
    assert!(popup_open.rejected_effects.is_empty());
    assert_eq!(popup_open.host_requests.len(), 1);
    assert_eq!(surface.input.popup_stack.len(), 1);
    assert_eq!(surface.input.popup_stack[0].popup_id, "menu.file");

    surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            popup_input_event(UiPopupInputEventKind::Dismissed, "menu.file", None),
        )
        .unwrap();
    assert!(surface.input.popup_stack.is_empty());

    surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            tooltip_input_event(UiTooltipTimerInputEventKind::Armed, "asset.tooltip"),
        )
        .unwrap();
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| tooltip.visible),
        Some(false)
    );

    let tooltip_shown = surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            tooltip_input_event(UiTooltipTimerInputEventKind::Elapsed, "asset.tooltip"),
        )
        .unwrap();
    assert!(tooltip_shown.rejected_effects.is_empty());
    assert_eq!(
        surface
            .input
            .tooltip
            .as_ref()
            .map(|tooltip| tooltip.visible),
        Some(true)
    );

    surface
        .dispatch_input_event(
            &pointer_dispatcher,
            &navigation_dispatcher,
            tooltip_input_event(UiTooltipTimerInputEventKind::Canceled, "asset.tooltip"),
        )
        .unwrap();
    assert_eq!(surface.input.tooltip, None);
}
