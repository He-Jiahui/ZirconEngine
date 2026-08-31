use super::*;
use zircon_runtime_interface::ui::dispatch::UiInputDiagnosticsMode;

#[test]
fn input_manager_summary_mode_keeps_pointer_behavior_receipt_without_full_trace() {
    let mut summary_surface = route_matrix_surface();
    let mut full_surface = route_matrix_surface();
    let mut summary_manager = UiInputManager::summary();
    let mut full_manager = UiInputManager::default();
    let event = touch_pointer_event_at(
        UiPointerId::new(21),
        UiPointerEventKind::Move,
        UiPoint::new(20.0, 20.0),
        10,
    );

    let summary = summary_manager
        .dispatch_input_event(&mut summary_surface, event.clone())
        .unwrap();
    let full = full_manager
        .dispatch_input_event(&mut full_surface, event)
        .unwrap();

    assert_eq!(
        summary_manager.diagnostics_mode(),
        UiInputDiagnosticsMode::Summary
    );
    assert_eq!(summary.pointer_routing, full.pointer_routing);
    assert!(summary.pointer_routing.is_some());
    assert!(summary.diagnostics.route_trace.preview_tunnel.is_empty());
    assert!(summary.diagnostics.route_trace.bubble_path.is_empty());
    assert!(summary.diagnostics.route_steps.is_empty());
    assert_eq!(
        summary.diagnostics.route_policy,
        full.diagnostics.route_policy
    );
    assert_eq!(summary.reply, full.reply);
    assert!(!full.diagnostics.route_trace.bubble_path.is_empty());
}

#[test]
fn captured_pointer_keeps_physical_hover_path_separate_from_dispatch_path() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::summary();
    let pointer_id = UiPointerId::new(22);
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
            UiPointerDispatchEffect::capture()
        });
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(2), UiPointerEventKind::Move, |_| {
            UiPointerDispatchEffect::handled()
        });

    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                pointer_id,
                UiPointerEventKind::Down,
                UiPoint::new(20.0, 20.0),
                10,
            ),
        )
        .unwrap();
    let moved = manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                pointer_id,
                UiPointerEventKind::Move,
                UiPoint::new(120.0, 20.0),
                20,
            ),
        )
        .unwrap();

    let routing = moved.pointer_routing.as_ref().unwrap();
    assert_eq!(routing.route_target, Some(UiNodeId::new(2)));
    assert_eq!(routing.capture_target, Some(UiNodeId::new(2)));
    assert_eq!(
        routing.physical_root_to_leaf(),
        &[UiNodeId::new(1), UiNodeId::new(3)]
    );
    assert_eq!(
        routing.dispatch_root_to_leaf(),
        &[UiNodeId::new(1), UiNodeId::new(2)]
    );

    let pointer = manager.active_pointers().entry(pointer_id).unwrap();
    assert_eq!(pointer.hovered, vec![UiNodeId::new(3), UiNodeId::new(1)]);
    assert_eq!(pointer.capture_target, Some(UiNodeId::new(2)));
}

#[test]
fn input_manager_double_click_count_is_owned_by_timer_state() {
    let mut surface = double_click_manager_surface();
    let mut manager = UiInputManager::default();
    let point = UiPoint::new(20.0, 20.0);
    let target = UiNodeId::new(2);

    manager
        .dispatch_input_event(
            &mut surface,
            pointer_event_at(UiPointerEventKind::Down, point, 10),
        )
        .unwrap();
    let first_release = manager
        .dispatch_input_event(
            &mut surface,
            pointer_event_at(UiPointerEventKind::Up, point, 20),
        )
        .unwrap();

    match &first_release.event {
        UiInputEvent::Pointer(pointer) => assert_eq!(pointer.event.click_count, 1),
        other => panic!("expected pointer input event, got {other:?}"),
    }
    assert!(!first_release.component_events.iter().any(|event| matches!(
        &event.event,
        UiComponentEvent::Commit { property, .. } if property == "double_activated"
    )));
    assert_eq!(manager.timers().double_click_target(), Some(target));
    assert_eq!(manager.timers().double_click_count(), Some(1));
    assert_eq!(
        manager.timers().double_click_expiration(),
        Some(UiInputTimestamp::from_micros(500_020))
    );

    manager
        .dispatch_input_event(
            &mut surface,
            pointer_event_at(UiPointerEventKind::Down, point, 120),
        )
        .unwrap();
    let second_release = manager
        .dispatch_input_event(
            &mut surface,
            pointer_event_at(UiPointerEventKind::Up, point, 140),
        )
        .unwrap();

    match &second_release.event {
        UiInputEvent::Pointer(pointer) => assert_eq!(pointer.event.click_count, 2),
        other => panic!("expected pointer input event, got {other:?}"),
    }
    assert!(second_release.component_events.iter().any(|event| {
        event.target == target
            && matches!(
                &event.event,
                UiComponentEvent::Commit { property, .. } if property == "double_activated"
            )
    }));
    assert_eq!(manager.timers().double_click_target(), Some(target));
    assert_eq!(manager.timers().double_click_count(), Some(2));

    let injected = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(500_140))
        .unwrap();
    assert!(injected.is_empty());
    assert_eq!(manager.timers().double_click_target(), None);
    assert_eq!(manager.timers().double_click_count(), None);
}

#[test]
fn input_manager_primary_touch_synthesizes_mouse_click() {
    let mut surface = double_click_manager_surface();
    let mut manager = UiInputManager::default();
    let pointer_id = UiPointerId::new(31);
    let point = UiPoint::new(20.0, 20.0);

    let down = manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(pointer_id, UiPointerEventKind::Down, point, 10),
        )
        .unwrap();
    let up = manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(pointer_id, UiPointerEventKind::Up, point, 20),
        )
        .unwrap();

    assert_pointer_button(&down, Some(UiPointerButton::Primary));
    assert_pointer_button(&up, Some(UiPointerButton::Primary));
    assert!(up.component_events.iter().any(|event| {
        event.target == UiNodeId::new(2)
            && matches!(
                &event.event,
                UiComponentEvent::Commit { property, value }
                    if property == "activated" && *value == UiValue::Bool(true)
            )
    }));
    let entry = manager.active_pointers().entry(pointer_id).unwrap();
    assert!(entry.is_primary);
    assert_eq!(entry.pressed_buttons, 0);
    assert_eq!(entry.last_point, Some(point));
    assert_eq!(surface.input.last_cursor_point(), None);
}

#[test]
fn input_manager_secondary_touch_keeps_table_press_without_mouse_activation() {
    let mut surface = double_click_manager_surface();
    let mut manager = UiInputManager::default();
    let primary_pointer = UiPointerId::new(41);
    let secondary_pointer = UiPointerId::new(42);
    let point = UiPoint::new(20.0, 20.0);

    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(primary_pointer, UiPointerEventKind::Down, point, 10),
        )
        .unwrap();
    let secondary_down = manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(secondary_pointer, UiPointerEventKind::Down, point, 20),
        )
        .unwrap();

    assert_pointer_button(&secondary_down, None);
    assert!(!secondary_down
        .component_events
        .iter()
        .any(|event| matches!(&event.event, UiComponentEvent::Press { pressed: true })));
    let secondary_entry = manager.active_pointers().entry(secondary_pointer).unwrap();
    assert!(!secondary_entry.is_primary);
    assert_eq!(secondary_entry.pressed_buttons, 0b001);
    assert_eq!(secondary_entry.pressed_target, Some(UiNodeId::new(2)));
    assert_eq!(secondary_entry.last_point, Some(point));

    let secondary_up = manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(secondary_pointer, UiPointerEventKind::Up, point, 30),
        )
        .unwrap();

    assert_pointer_button(&secondary_up, None);
    assert!(!secondary_up.component_events.iter().any(|event| matches!(
        &event.event,
        UiComponentEvent::Commit { property, .. } if property == "activated"
    )));
    let secondary_entry = manager.active_pointers().entry(secondary_pointer).unwrap();
    assert_eq!(secondary_entry.pressed_buttons, 0);
    assert_eq!(secondary_entry.pressed_target, None);
    assert_eq!(surface.input.last_cursor_point(), None);
}

#[test]
fn input_manager_touch_cancel_clears_pointer_entry_and_capture() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::default();
    let pointer_id = UiPointerId::new(51);
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
            UiPointerDispatchEffect::capture()
        });

    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                pointer_id,
                UiPointerEventKind::Down,
                UiPoint::new(20.0, 20.0),
                10,
            ),
        )
        .unwrap();

    assert!(manager.active_pointers().entry(pointer_id).is_some());
    assert_eq!(
        surface.input.pointer_capture_owner(pointer_id),
        Some(UiNodeId::new(2))
    );

    let canceled = manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                pointer_id,
                UiPointerEventKind::Cancel,
                UiPoint::new(120.0, 80.0),
                20,
            ),
        )
        .unwrap();

    assert_eq!(
        canceled.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert!(manager.active_pointers().entry(pointer_id).is_none());
    assert_eq!(surface.input.pointer_capture_owner(pointer_id), None);
    assert_eq!(surface.focus.captured, None);
}

#[test]
fn input_manager_two_touch_pointers_keep_independent_hover_and_press() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::default();
    let first_pointer = UiPointerId::new(61);
    let second_pointer = UiPointerId::new(62);

    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                first_pointer,
                UiPointerEventKind::Down,
                UiPoint::new(20.0, 20.0),
                10,
            ),
        )
        .unwrap();
    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                second_pointer,
                UiPointerEventKind::Down,
                UiPoint::new(120.0, 20.0),
                20,
            ),
        )
        .unwrap();

    let first_entry = manager.active_pointers().entry(first_pointer).unwrap();
    assert!(first_entry.is_primary);
    assert_eq!(first_entry.last_point, Some(UiPoint::new(20.0, 20.0)));
    assert_eq!(
        first_entry.hovered,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(first_entry.pressed_buttons, 0b001);
    assert_eq!(first_entry.pressed_target, Some(UiNodeId::new(2)));

    let second_entry = manager.active_pointers().entry(second_pointer).unwrap();
    assert!(!second_entry.is_primary);
    assert_eq!(second_entry.last_point, Some(UiPoint::new(120.0, 20.0)));
    assert_eq!(
        second_entry.hovered,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(second_entry.pressed_buttons, 0b001);
    assert_eq!(second_entry.pressed_target, Some(UiNodeId::new(3)));

    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                first_pointer,
                UiPointerEventKind::Cancel,
                UiPoint::new(20.0, 20.0),
                30,
            ),
        )
        .unwrap();

    assert!(manager.active_pointers().entry(first_pointer).is_none());
    let second_entry = manager.active_pointers().entry(second_pointer).unwrap();
    assert_eq!(second_entry.pressed_buttons, 0b001);
    assert_eq!(second_entry.pressed_target, Some(UiNodeId::new(3)));
}

#[test]
fn input_manager_multi_pointer_capture_isolation_survives_cancel() {
    let mut surface = route_matrix_surface();
    let mut manager = UiInputManager::default();
    let first_pointer = UiPointerId::new(71);
    let second_pointer = UiPointerId::new(72);
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(2), UiPointerEventKind::Down, |_| {
            UiPointerDispatchEffect::capture()
        });
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(3), UiPointerEventKind::Down, |_| {
            UiPointerDispatchEffect::capture()
        });
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(2), UiPointerEventKind::Move, |_| {
            UiPointerDispatchEffect::handled()
        });
    manager
        .pointer_dispatcher_mut()
        .register(UiNodeId::new(3), UiPointerEventKind::Move, |_| {
            UiPointerDispatchEffect::handled()
        });

    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                first_pointer,
                UiPointerEventKind::Down,
                UiPoint::new(20.0, 20.0),
                10,
            ),
        )
        .unwrap();
    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                second_pointer,
                UiPointerEventKind::Down,
                UiPoint::new(120.0, 20.0),
                20,
            ),
        )
        .unwrap();

    assert_eq!(
        surface.input.pointer_capture_owner(first_pointer),
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        surface.input.pointer_capture_owner(second_pointer),
        Some(UiNodeId::new(3))
    );

    let first_move = manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                first_pointer,
                UiPointerEventKind::Move,
                UiPoint::new(160.0, 80.0),
                30,
            ),
        )
        .unwrap();
    assert_eq!(
        first_move.diagnostics.route_policy,
        UiInputRoutePolicy::PointerCapture
    );
    assert_eq!(
        first_move.diagnostics.route_trace.direct_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        manager
            .active_pointers()
            .entry(first_pointer)
            .unwrap()
            .capture_target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        manager
            .active_pointers()
            .entry(second_pointer)
            .unwrap()
            .capture_target,
        Some(UiNodeId::new(3))
    );

    manager
        .dispatch_input_event(
            &mut surface,
            touch_pointer_event_at(
                first_pointer,
                UiPointerEventKind::Cancel,
                UiPoint::new(160.0, 80.0),
                40,
            ),
        )
        .unwrap();

    assert!(manager.active_pointers().entry(first_pointer).is_none());
    assert_eq!(surface.input.pointer_capture_owner(first_pointer), None);
    assert_eq!(
        surface.input.pointer_capture_owner(second_pointer),
        Some(UiNodeId::new(3))
    );
    assert_eq!(surface.focus.captured, Some(UiNodeId::new(3)));
    assert_eq!(
        manager
            .active_pointers()
            .entry(second_pointer)
            .unwrap()
            .capture_target,
        Some(UiNodeId::new(3))
    );
}
