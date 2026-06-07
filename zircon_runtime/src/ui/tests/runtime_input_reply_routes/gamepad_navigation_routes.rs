use super::*;

#[test]
fn unified_gamepad_dpad_right_routes_to_navigation_right_from_focused_path() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_gamepad_dpad_input(&mut surface, "Gamepad_DPad_Right");

    assert_gamepad_dpad_navigation_result(
        &surface,
        &result,
        "Gamepad_DPad_Right",
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );
}

#[test]
fn unified_gamepad_dpad_left_routes_to_navigation_left_from_focused_path() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = dispatch_gamepad_dpad_input(&mut surface, "Gamepad_DPad_Left");

    assert_gamepad_dpad_navigation_result(
        &surface,
        &result,
        "Gamepad_DPad_Left",
        UiNavigationEventKind::Left,
        UiNodeId::new(3),
        UiNodeId::new(2),
    );
}

#[test]
fn unified_gamepad_dpad_down_routes_to_navigation_down_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_gamepad_dpad_input(&mut surface, "Gamepad_DPad_Down");

    assert_gamepad_dpad_navigation_result(
        &surface,
        &result,
        "Gamepad_DPad_Down",
        UiNavigationEventKind::Down,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );
}

#[test]
fn unified_gamepad_dpad_up_routes_to_navigation_up_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = dispatch_gamepad_dpad_input(&mut surface, "Gamepad_DPad_Up");

    assert_gamepad_dpad_navigation_result(
        &surface,
        &result,
        "Gamepad_DPad_Up",
        UiNavigationEventKind::Up,
        UiNodeId::new(3),
        UiNodeId::new(2),
    );
}

fn dispatch_gamepad_dpad_input(
    surface: &mut UiSurface,
    logical_key: &str,
) -> UiInputDispatchResult {
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Keyboard(UiKeyboardInputEvent {
                metadata: input_metadata(),
                state: UiKeyboardInputState::Pressed,
                key_code: 0,
                scan_code: None,
                physical_key: logical_key.to_string(),
                logical_key: logical_key.to_string(),
                text: None,
            }),
        )
        .unwrap()
}

fn assert_gamepad_dpad_navigation_result(
    surface: &UiSurface,
    result: &UiInputDispatchResult,
    logical_key: &str,
    kind: UiNavigationEventKind,
    route_target: UiNodeId,
    focused_after: UiNodeId,
) {
    assert_eq!(surface.focus.focused, Some(focused_after));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == focused_after
                && *reason == UiFocusEffectReason::Navigation
    ));
    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(keyboard.logical_key, logical_key);
            assert_eq!(keyboard.key_code, 0);
            assert!(keyboard.text.is_none());
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
        .any(|note| note == &format!("keyboard_navigation={kind:?}")));
    assert_eq!(result.diagnostics.route_target, Some(route_target));
    assert_eq!(result.diagnostics.route_trace.target, Some(route_target));
    assert_eq!(
        result.diagnostics.route_trace.preview_tunnel,
        vec![UiNodeId::new(1), route_target]
    );
    assert_eq!(
        result.diagnostics.route_trace.bubble_path,
        vec![route_target, UiNodeId::new(1)]
    );
    assert_eq!(
        result.diagnostics.route_trace.focus_path,
        vec![route_target, UiNodeId::new(1)]
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
    assert_eq!(result.diagnostics.route_steps[2].target, Some(route_target));
    assert_eq!(
        result.diagnostics.route_steps[2].disposition,
        UiDispatchDisposition::Handled
    );
    assert_eq!(result.diagnostics.route_steps[2].effect_count, 1);
    assert!(result.diagnostics.route_steps[2].stopped);
    assert_eq!(surface.focus.focused_inputs.len(), 1);
    assert_eq!(surface.focus.focused_inputs[0].focused, focused_after);
    assert_eq!(
        surface.focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        surface.focus.focused_inputs[0].route,
        vec![focused_after, UiNodeId::new(1)]
    );
    assert_eq!(
        surface.focus.focused_inputs[0].handled_by,
        Some(route_target)
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
