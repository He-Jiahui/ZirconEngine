use super::*;

#[test]
fn drag_drop_lifecycle_tracks_shared_state_and_cleans_capture_on_end() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);
    let payload = UiDragPayload::new(UiDragPayloadKind::Asset, "res://assets/materials/brick.mat");

    let begin = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Begin,
            UiNodeId::new(2),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(14.0, 18.0)),
            Some(payload.clone()),
        )),
    );

    assert!(begin.rejected_effects.is_empty());
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(2)));
    assert_pointer_capture(&surface, pointer_id, UiNodeId::new(2));
    let drag = surface.input.drag_drop.as_ref().expect("active drag");
    assert_eq!(drag.session_id, session_id);
    assert_eq!(drag.source, UiNodeId::new(2));
    assert_eq!(drag.target, UiNodeId::new(2));
    assert_eq!(drag.point, Some(UiPoint::new(14.0, 18.0)));
    assert_eq!(drag.payload.as_deref(), Some(&payload));

    let update = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            UiNodeId::new(3),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(44.0, 68.0)),
            None,
        )),
    );
    assert!(update.rejected_effects.is_empty());
    let drag = surface.input.drag_drop.as_ref().expect("updated drag");
    assert_eq!(drag.target, UiNodeId::new(3));
    assert_eq!(drag.point, Some(UiPoint::new(44.0, 68.0)));
    assert_eq!(drag.payload.as_deref(), Some(&payload));

    let accept = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Accept,
            UiNodeId::new(3),
            pointer_id,
            Some(session_id),
            None,
            None,
        )),
    );
    assert!(accept.rejected_effects.is_empty());
    assert!(
        surface
            .input
            .drag_drop
            .as_ref()
            .expect("accepted drag")
            .accepted
    );

    surface.input.high_precision_owner = Some(UiNodeId::new(2));
    let complete = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Complete,
            UiNodeId::new(3),
            pointer_id,
            Some(session_id),
            None,
            None,
        )),
    );

    assert!(complete.rejected_effects.is_empty());
    assert_eq!(surface.input.drag_drop, None);
    assert_eq!(surface.focus.captured, None);
    assert_no_pointer_capture(&surface);
    assert_eq!(surface.input.high_precision_owner, None);
}

#[test]
fn drag_drop_rejects_stale_pointer_or_session_without_clearing_active_drag() {
    let mut surface = two_button_surface();
    let pointer_id = UiPointerId::new(7);
    let session_id = UiDragSessionId::new(42);

    let begin = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Begin,
            UiNodeId::new(2),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(12.0, 16.0)),
            None,
        )),
    );
    assert!(begin.rejected_effects.is_empty());

    let stale_session = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            UiNodeId::new(3),
            pointer_id,
            Some(UiDragSessionId::new(99)),
            Some(UiPoint::new(50.0, 70.0)),
            None,
        )),
    );
    assert_eq!(stale_session.rejected_effects.len(), 1);
    assert_eq!(
        stale_session.rejected_effects[0].reason,
        "drag session owner mismatch"
    );
    let drag = surface
        .input
        .drag_drop
        .as_ref()
        .expect("drag remains active");
    assert_eq!(drag.target, UiNodeId::new(2));
    assert_eq!(drag.point, Some(UiPoint::new(12.0, 16.0)));

    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(3))
        .unwrap()
        .state_flags
        .enabled = false;
    let invalid_target = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Update,
            UiNodeId::new(3),
            pointer_id,
            Some(session_id),
            Some(UiPoint::new(50.0, 70.0)),
            None,
        )),
    );
    assert_eq!(invalid_target.rejected_effects.len(), 1);
    assert!(invalid_target.rejected_effects[0]
        .reason
        .starts_with("invalid input owner"));
    let drag = surface
        .input
        .drag_drop
        .as_ref()
        .expect("drag remains active after invalid target");
    assert_eq!(drag.target, UiNodeId::new(2));
    assert_eq!(drag.point, Some(UiPoint::new(12.0, 16.0)));

    let stale_pointer = surface.apply_dispatch_reply(
        keyboard_event(),
        UiDispatchReply::handled().with_effect(drag_effect(
            UiDragDropEffectKind::Cancel,
            UiNodeId::new(2),
            UiPointerId::new(99),
            Some(session_id),
            None,
            None,
        )),
    );
    assert_eq!(stale_pointer.rejected_effects.len(), 1);
    assert_eq!(
        stale_pointer.rejected_effects[0].reason,
        "drag pointer owner mismatch"
    );
    assert!(surface.input.drag_drop.is_some());
}
