use super::*;

#[test]
fn unified_analog_left_x_positive_routes_to_navigation_right_from_focused_path() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_analog_input(&mut surface, "Gamepad_LeftX", 0.75);

    assert_analog_navigation_result(
        &surface,
        &result,
        "Gamepad_LeftX",
        0.75,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );
}

#[test]
fn unified_analog_left_x_negative_routes_to_navigation_left_from_focused_path() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = dispatch_analog_input(&mut surface, "Gamepad_LeftX", -0.75);

    assert_analog_navigation_result(
        &surface,
        &result,
        "Gamepad_LeftX",
        -0.75,
        UiNavigationEventKind::Left,
        UiNodeId::new(3),
        UiNodeId::new(2),
    );
}

#[test]
fn unified_analog_left_y_negative_routes_to_navigation_down_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_analog_input(&mut surface, "Gamepad_LeftY", -0.75);

    assert_analog_navigation_result(
        &surface,
        &result,
        "Gamepad_LeftY",
        -0.75,
        UiNavigationEventKind::Down,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );
}

#[test]
fn unified_analog_left_y_positive_routes_to_navigation_up_from_focused_path() {
    let mut surface = route_surface();
    surface.focus_node(UiNodeId::new(3)).unwrap();

    let result = dispatch_analog_input(&mut surface, "Gamepad_LeftY", 0.75);

    assert_analog_navigation_result(
        &surface,
        &result,
        "Gamepad_LeftY",
        0.75,
        UiNavigationEventKind::Up,
        UiNodeId::new(3),
        UiNodeId::new(2),
    );
}

#[test]
fn unified_analog_left_x_threshold_stays_owner_routed_without_navigation() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let result = dispatch_analog_input(&mut surface, "Gamepad_LeftX", 0.5);

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert!(result.applied_effects.is_empty());
    match &result.event {
        UiInputEvent::Analog(analog) => {
            assert_eq!(analog.control, "Gamepad_LeftX");
            assert_eq!(analog.value, 0.5);
        }
        other => panic!("expected original analog input event, got {other:?}"),
    }
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("analog.focused")
    );
    assert!(!result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("analog_navigation=")));
    assert_eq!(result.diagnostics.route_target, Some(UiNodeId::new(2)));
    assert_eq!(surface.focus.focused_inputs.len(), 0);
}

#[test]
fn unified_analog_left_x_repeat_waits_for_initial_repeat_interval() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let first = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.75, 1_000_000);

    assert_analog_navigation_result(
        &surface,
        &first,
        "Gamepad_LeftX",
        0.75,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.focus.focused_inputs.clear();

    let waiting = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.75, 1_400_000);

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(waiting.reply.disposition, UiDispatchDisposition::Unhandled);
    assert!(waiting.reply.handler.is_none());
    assert!(waiting.applied_effects.is_empty());
    assert!(waiting.component_events.is_empty());
    assert!(surface.focus.focused_inputs.is_empty());
    assert_eq!(
        waiting.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_ne!(
        waiting.diagnostics.handled_phase.as_deref(),
        Some("analog.navigation")
    );
    assert!(waiting
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "analog_repeat_suppressed"));
    assert!(waiting
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "analog_navigation_repeat_suppressed=Right"));
    assert!(!waiting
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "analog_navigation=Right"));
}

#[test]
fn unified_analog_left_x_repeats_after_initial_interval_without_value_change() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let first = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.75, 1_000_000);

    assert_analog_navigation_result(
        &surface,
        &first,
        "Gamepad_LeftX",
        0.75,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.focus.focused_inputs.clear();

    let repeated = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.75, 1_510_000);

    assert_analog_navigation_result(
        &surface,
        &repeated,
        "Gamepad_LeftX",
        0.75,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );
}

#[test]
fn unified_analog_left_x_high_pressure_uses_shorter_initial_repeat_interval() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let first = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.95, 1_000_000);

    assert_analog_navigation_result(
        &surface,
        &first,
        "Gamepad_LeftX",
        0.95,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.focus.focused_inputs.clear();

    let repeated = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.95, 1_260_000);

    assert_analog_navigation_result(
        &surface,
        &repeated,
        "Gamepad_LeftX",
        0.95,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );
}

#[test]
fn unified_analog_left_x_dead_zone_resets_repeat_state() {
    let mut surface = horizontal_route_surface();
    surface.focus_node(UiNodeId::new(2)).unwrap();

    let first = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.75, 1_000_000);

    assert_analog_navigation_result(
        &surface,
        &first,
        "Gamepad_LeftX",
        0.75,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );

    surface.focus_node(UiNodeId::new(2)).unwrap();
    surface.focus.focused_inputs.clear();

    let neutral = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.0, 1_100_000);

    assert_eq!(surface.focus.focused, Some(UiNodeId::new(2)));
    assert_eq!(neutral.reply.disposition, UiDispatchDisposition::Handled);
    assert!(neutral.applied_effects.is_empty());
    assert!(!neutral
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("analog_navigation=")));

    let crossed_again = dispatch_analog_input_at(&mut surface, "Gamepad_LeftX", 0.75, 1_120_000);

    assert_analog_navigation_result(
        &surface,
        &crossed_again,
        "Gamepad_LeftX",
        0.75,
        UiNavigationEventKind::Right,
        UiNodeId::new(2),
        UiNodeId::new(3),
    );
}

fn dispatch_analog_input(
    surface: &mut UiSurface,
    control: &str,
    value: f32,
) -> UiInputDispatchResult {
    dispatch_analog_input_at(surface, control, value, 10)
}

fn dispatch_analog_input_at(
    surface: &mut UiSurface,
    control: &str,
    value: f32,
    monotonic_micros: u64,
) -> UiInputDispatchResult {
    let mut metadata = input_metadata();
    metadata.timestamp = UiInputTimestamp::from_micros(monotonic_micros);
    metadata.sequence = UiInputSequence::new(monotonic_micros);
    surface
        .dispatch_input_event(
            &UiPointerDispatcher::default(),
            &UiNavigationDispatcher::default(),
            UiInputEvent::Analog(UiAnalogInputEvent {
                metadata,
                control: control.to_string(),
                value,
            }),
        )
        .unwrap()
}

fn assert_analog_navigation_result(
    surface: &UiSurface,
    result: &UiInputDispatchResult,
    control: &str,
    value: f32,
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
        UiInputEvent::Analog(analog) => {
            assert_eq!(analog.control, control);
            assert_eq!(analog.value, value);
        }
        other => panic!("expected original analog input event, got {other:?}"),
    }
    assert_eq!(
        surface
            .input
            .analog_controls
            .get(control)
            .map(|state| state.value),
        Some(value)
    );
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("analog.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == &format!("analog_navigation={kind:?}")));
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
