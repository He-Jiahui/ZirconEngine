use super::*;
use crate::ui::dispatch::UiInputManager;

#[test]
fn unified_keyboard_tab_routes_to_navigation_next_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_event(false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(3)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Tab");
            assert!(!keyboard.metadata.modifiers.shift);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Next"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), UiNodeId::new(2)]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(result.diagnostics.route_steps.len(), 3);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[1].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(
        result.diagnostics.route_steps[2].phase,
        UiDispatchPhase::Target
    );
    assert_eq!(
        result.diagnostics.route_steps[2].target,
        Some(UiNodeId::new(2))
    );
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 1);
    assert!(result.diagnostics.route_steps[2].stopped);
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_shift_tab_routes_to_navigation_previous_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_event(true),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(2)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "Tab");
            assert!(keyboard.metadata.modifiers.shift);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Previous"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_down_routes_to_directional_navigation_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowDown", 40, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(3)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "ArrowDown");
            assert_eq!(keyboard.key_code, 40);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Down"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_right_prefers_semantic_tabs_keyboard_action_binding() {
    let mut surface = semantic_tabs_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowRight", 39, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert!(result.applied_effects.is_empty());
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.component_action")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_component_action=Next"));
    assert!(!result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("keyboard_navigation=")));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::KeyboardAction {
            action: UiComponentKeyboardAction::Next,
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_right_prefers_tree_view_expand_keyboard_action_binding() {
    let mut surface = semantic_tree_view_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowRight", 39, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert!(result.applied_effects.is_empty());
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.component_action")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_component_action=Increment"));
    assert!(!result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("keyboard_navigation=")));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::KeyboardAction {
            action: UiComponentKeyboardAction::Increment,
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_left_prefers_tree_view_collapse_keyboard_action_binding() {
    let mut surface = semantic_tree_view_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowLeft", 37, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert!(result.applied_effects.is_empty());
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.component_action")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_component_action=Decrement"));
    assert!(!result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("keyboard_navigation=")));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::KeyboardAction {
            action: UiComponentKeyboardAction::Decrement,
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_f2_prefers_tree_view_begin_edit_keyboard_action_binding() {
    let mut surface = semantic_tree_view_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("F2", 113, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert!(result.applied_effects.is_empty());
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.component_action")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_component_action=BeginEdit"));
    assert!(!result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("keyboard_navigation=")));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::KeyboardAction {
            action: UiComponentKeyboardAction::BeginEdit,
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_printable_text_prefers_semantic_menu_keyboard_text_binding() {
    let mut surface = semantic_menu_list_text_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("c", 67, Some("c"), false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(UiNodeId::new(2)));
    assert!(result.applied_effects.is_empty());
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.component_text")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_component_text=c"));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, UiNodeId::new(2));
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::KeyboardText {
            text: "c".to_string(),
        }
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

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

#[test]
fn unified_keyboard_arrow_up_routes_to_directional_navigation_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowUp", 38, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(2)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "ArrowUp");
            assert_eq!(keyboard.key_code, 38);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Up"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_right_routes_to_directional_navigation_from_horizontal_focused_path() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowRight", 39, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(3)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(3)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "ArrowRight");
            assert_eq!(keyboard.key_code, 39);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Right"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(2), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(3));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(2))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

#[test]
fn unified_keyboard_arrow_left_routes_to_directional_navigation_from_horizontal_focused_path() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            keyboard_navigation_key_event("ArrowLeft", 37, None, false),
        )
        .unwrap();

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == UiNodeId::new(2)
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, "ArrowLeft");
            assert_eq!(keyboard.key_code, 37);
        }
        other => panic!("expected original keyboard input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Left"));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(3)));
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![UiNodeId::new(3), UiNodeId::new(1)]
    );
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, UiNodeId::new(2));
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(UiNodeId::new(3))
    );
    assert!(surface.focus.focused_inputs[0].accepted);
}

fn horizontal_route_surface() -> UiSurface {
    let mut surface = route_surface();
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(2))
        .unwrap()
        .layout_cache
        .frame = UiFrame::new(10.0, 10.0, 60.0, 30.0);
    surface
        .tree
        .nodes
        .get_mut(&UiNodeId::new(3))
        .unwrap()
        .layout_cache
        .frame = UiFrame::new(90.0, 10.0, 60.0, 30.0);
    surface
}

fn semantic_tabs_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "Tabs".to_string(),
        control_id: Some("MainTabs".to_string()),
        bindings: vec![binding("Tabs/KeyboardAction", UiEventKind::Change)],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_tree_view_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "TreeView".to_string(),
        control_id: Some("AssetTree".to_string()),
        bindings: vec![binding("TreeView/KeyboardAction", UiEventKind::Change)],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_menu_list_text_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MenuList".to_string(),
        control_id: Some("SceneMenu".to_string()),
        bindings: vec![binding("MenuList/KeyboardText", UiEventKind::Change)],
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_menu_list_typeahead_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MenuList".to_string(),
        control_id: Some("SceneMenu".to_string()),
        bindings: vec![
            binding("MenuList/KeyboardText", UiEventKind::Change),
            binding("MenuList/TypeaheadExpired", UiEventKind::Change),
        ],
        attributes: toml::from_str("typeahead_timeout_ms = 100").unwrap(),
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_menu_list_submenu_hover_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "MenuList".to_string(),
        control_id: Some("SceneMenu".to_string()),
        bindings: vec![binding("MenuList/ValueChanged", UiEventKind::Change)],
        attributes: toml::from_str("submenu_hover_delay_ms = 100").unwrap(),
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn semantic_snackbar_toast_route_surface() -> UiSurface {
    let mut surface = route_surface();
    let target = surface.tree.nodes.get_mut(&UiNodeId::new(2)).unwrap();
    target.template_metadata = Some(UiTemplateNodeMetadata {
        component: "Snackbar".to_string(),
        control_id: Some("StatusToast".to_string()),
        bindings: vec![binding("Snackbar/Commit", UiEventKind::Change)],
        attributes: toml::from_str(
            r#"
current_toast_id = "save"
auto_hide_duration_ms = 4000
open = true
"#,
        )
        .unwrap(),
        ..Default::default()
    });
    surface.rebuild();
    surface
}

fn keyboard_navigation_event(shift: bool) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = shift;
    keyboard_navigation_key_event_with_metadata(metadata, "Tab", 9, Some("\t"))
}

fn keyboard_navigation_key_event(
    logical_key: &str,
    key_code: u32,
    text: Option<&str>,
    shift: bool,
) -> UiInputEvent {
    let mut metadata = input_metadata();
    metadata.modifiers.shift = shift;
    keyboard_navigation_key_event_with_metadata(metadata, logical_key, key_code, text)
}

fn keyboard_navigation_key_event_with_metadata(
    metadata: UiInputEventMetadata,
    logical_key: &str,
    key_code: u32,
    text: Option<&str>,
) -> UiInputEvent {
    UiInputEvent::Keyboard(UiKeyboardInputEvent {
        metadata,
        state: UiKeyboardInputState::Pressed,
        key_code,
        scan_code: None,
        physical_key: logical_key.to_string(),
        logical_key: logical_key.to_string(),
        text: text.map(str::to_string),
    })
}
