use super::*;

#[test]
fn unified_keyboard_text_arms_typeahead_expiry_timer_and_tick_dispatches_event() {
    let mut surface = semantic_menu_list_typeahead_route_surface();
    let mut manager = UiInputManager::default();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = manager
        .dispatch_input_event(
            &mut surface,
            keyboard_navigation_key_event("c", 67, Some("c"), false),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.component_text")
    );
    assert_eq!(
        manager.timers().typeahead_expiration(UiNodeId::new(2)),
        Some(UiInputTimestamp::from_micros(100_010))
    );

    let early = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(100_009))
        .unwrap();
    assert!(early.is_empty());
    assert_eq!(
        manager.timers().typeahead_expiration(UiNodeId::new(2)),
        Some(UiInputTimestamp::from_micros(100_010))
    );

    let expired = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(100_010))
        .unwrap();

    assert_eq!(expired.len(), 1);
    let expired = &expired[0];
    assert_eq!(expired.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(expired.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(
        expired.diagnostics.handled_phase.as_deref(),
        Some("typeahead_timer.component_event")
    );
    assert_eq!(
        expired.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(expired.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(expired.component_events.len(), 1);
    assert_eq!(expired.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        expired.component_events[0].event,
        UiComponentEvent::TypeaheadExpired
    );
    match &expired.event {
        UiInputEvent::TypeaheadTimer(timer) => {
            assert_eq!(
                timer.metadata.timestamp,
                UiInputTimestamp::from_micros(100_010)
            );
            assert_eq!(timer.target, UiNodeId::new(2));
        }
        other => panic!("expected typeahead timer input event, got {other:?}"),
    }
    assert_eq!(
        manager.timers().typeahead_expiration(UiNodeId::new(2)),
        None
    );

    let repeat = manager
        .tick(&mut surface, UiInputTimestamp::from_micros(100_011))
        .unwrap();
    assert!(repeat.is_empty());
}

#[test]
fn submenu_hover_timer_dispatches_ready_value_changed_event() {
    let mut surface = semantic_menu_list_submenu_hover_route_surface();
    let mut manager = UiInputManager::default();

    let result = manager
        .dispatch_input_event(
            &mut surface,
            UiInputEvent::SubmenuHoverTimer(UiSubmenuHoverTimerInputEvent {
                metadata: input_metadata(),
                target: UiNodeId::new(2),
                option_id: "file".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("submenu_hover_timer.component_event")
    );
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::ValueChanged {
            property: "submenu_hover_ready".to_string(),
            value: UiValue::Bool(true),
        }
    );
    match &result.event {
        UiInputEvent::SubmenuHoverTimer(timer) => {
            assert_eq!(timer.target, UiNodeId::new(2));
            assert_eq!(timer.option_id, "file");
        }
        other => panic!("expected submenu hover timer input event, got {other:?}"),
    }
}

#[test]
fn toast_timer_dispatches_expired_commit_event() {
    let mut surface = semantic_snackbar_toast_route_surface();
    let mut manager = UiInputManager::default();

    let result = manager
        .dispatch_input_event(
            &mut surface,
            UiInputEvent::ToastTimer(UiToastTimerInputEvent {
                metadata: input_metadata(),
                target: UiNodeId::new(2),
                toast_id: "save".to_string(),
            }),
        )
        .unwrap();

    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("toast_timer.component_event")
    );
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "expired_toast_id".to_string(),
            value: UiValue::String("save".to_string()),
        }
    );
    match &result.event {
        UiInputEvent::ToastTimer(timer) => {
            assert_eq!(timer.target, UiNodeId::new(2));
            assert_eq!(timer.toast_id, "save");
        }
        other => panic!("expected toast timer input event, got {other:?}"),
    }
}

#[test]
fn unified_keyboard_printable_text_respects_disabled_component_gate() {
    let mut surface = semantic_menu_list_text_route_surface();
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .state_flags
        .enabled = false;
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("c", 67, Some("c"), false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Unhandled);
    assert_eq!(result.reply.handler, None);
    assert!(result.applied_effects.is_empty());
    assert_eq!(result.diagnostics.handled_phase, None);
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "focused_route_len=2"));
    assert!(!result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_component_text=c"));
    assert!(result.component_events.is_empty());
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(surface.focus.focused_inputs[0].handled_by, None);
    assert!(surface.focus.focused_inputs[0].accepted);
}
