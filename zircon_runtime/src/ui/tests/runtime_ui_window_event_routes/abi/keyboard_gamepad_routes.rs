use super::*;

#[test]
fn runtime_ui_manager_routes_runtime_keyboard_enter_through_focused_window_path() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    let track_button = node_id_by_control_id(&manager, "TrackQuestButton");
    let track_frame = manager
        .surface()
        .surface_frame()
        .arranged_tree
        .get(track_button)
        .expect("track button should have an arranged frame")
        .frame;
    let focus_point = UiPoint::new(
        track_frame.x + track_frame.width * 0.5,
        track_frame.y + track_frame.height * 0.5,
    );
    manager.register_pointer_handler(track_button, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::set_focus()
    });

    manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                focus_point.x,
                focus_point.y,
            ),
        )
        .expect("runtime mouse-button ABI event should focus the track button");
    assert_eq!(manager.surface().focus.focused, Some(track_button));
    assert!(
        !manager
            .surface()
            .component_state(track_button)
            .expect("focused pointer target")
            .flags
            .focus_visible
    );

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::keyboard(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_KEY_ACTION_PRESSED_V1,
                13,
                28,
                ZrByteSlice::empty(),
            ),
        )
        .expect("runtime keyboard ABI event should route through the focused manager path");

    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(
                keyboard.metadata.window_id,
                Some(UiWindowId::new("runtime.main"))
            );
            assert_eq!(
                keyboard.metadata.timestamp,
                UiInputTimestamp::from_micros(42)
            );
            assert_eq!(keyboard.metadata.sequence, UiInputSequence::new(7));
            assert_eq!(keyboard.state, UiKeyboardInputState::Pressed);
            assert_eq!(keyboard.key_code, 13);
            assert_eq!(keyboard.scan_code, Some(28));
            assert_eq!(keyboard.physical_key, "Enter");
            assert_eq!(keyboard.logical_key, "Enter");
            assert_eq!(keyboard.text, None);
        }
        other => panic!(
            "expected runtime keyboard ABI event to normalize into keyboard input, got {other:?}"
        ),
    }
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(result.diagnostics.route_target, Some(track_button));
    assert_eq!(result.diagnostics.route_trace.target, Some(track_button));
    assert_eq!(
        result.diagnostics.route_trace.focus_path.first().copied(),
        Some(track_button)
    );
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.widget")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note.starts_with("focused_route_len=")));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(track_button));
    assert_eq!(result.reply.phase, Some(UiDispatchPhase::Target));
    assert!(result.diagnostics.route_steps.iter().any(|step| {
        step.phase == UiDispatchPhase::Target
            && step.target == Some(track_button)
            && step.handler == Some(track_button)
            && step.disposition == UiDispatchDisposition::Handled
    }));
    assert_eq!(result.component_events.len(), 1);
    assert_eq!(result.component_events[0].target, track_button);
    assert_eq!(
        result.component_events[0].event,
        UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        }
    );
    assert_eq!(manager.surface().focus.focused, Some(track_button));
    assert_eq!(manager.surface().focus.focused_inputs.len(), 1);
    assert_eq!(
        manager.surface().focus.focused_inputs[0].kind,
        UiFocusedInputKind::Keyboard
    );
    assert_eq!(
        manager.surface().focus.focused_inputs[0].handled_by,
        Some(track_button)
    );
    assert!(manager.surface().focus.focused_inputs[0].accepted);
    assert!(!manager.surface().dirty_flags().any());
}

#[test]
fn runtime_ui_manager_routes_runtime_gamepad_dpad_right_through_focused_navigation_path() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    let track_button = node_id_by_control_id(&manager, "TrackQuestButton");
    let close_button = node_id_by_control_id(&manager, "CloseQuestLogButton");
    let track_frame = manager
        .surface()
        .surface_frame()
        .arranged_tree
        .get(track_button)
        .expect("track button should have an arranged frame")
        .frame;
    let focus_point = UiPoint::new(
        track_frame.x + track_frame.width * 0.5,
        track_frame.y + track_frame.height * 0.5,
    );
    manager.register_pointer_handler(track_button, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::set_focus()
    });
    manager.register_navigation_handler(track_button, UiNavigationEventKind::Right, move |_| {
        UiNavigationDispatchEffect::focus(close_button)
    });

    manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                focus_point.x,
                focus_point.y,
            ),
        )
        .expect("runtime mouse-button ABI event should focus the track button");
    assert_eq!(manager.surface().focus.focused, Some(track_button));

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::gamepad_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                2,
                ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                1.0,
            ),
        )
        .expect("runtime gamepad ABI event should route through focused navigation");

    match &result.event {
        UiInputEvent::Keyboard(keyboard) => {
            assert_eq!(
                keyboard.metadata.window_id,
                Some(UiWindowId::new("runtime.main"))
            );
            assert_eq!(
                keyboard.metadata.timestamp,
                UiInputTimestamp::from_micros(42)
            );
            assert_eq!(keyboard.metadata.sequence, UiInputSequence::new(7));
            assert_eq!(keyboard.state, UiKeyboardInputState::Pressed);
            assert_eq!(keyboard.key_code, 0);
            assert_eq!(keyboard.scan_code, None);
            assert_eq!(keyboard.physical_key, "Gamepad_DPad_Right");
            assert_eq!(keyboard.logical_key, "Gamepad_DPad_Right");
            assert_eq!(keyboard.text, None);
        }
        other => panic!(
            "expected runtime gamepad ABI event to normalize into keyboard input, got {other:?}"
        ),
    }
    assert_eq!(manager.surface().focus.focused, Some(close_button));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(result.diagnostics.route_target, Some(track_button));
    assert_eq!(result.diagnostics.route_trace.target, Some(track_button));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("keyboard.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "keyboard_navigation=Right"));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(track_button));
    assert_eq!(result.reply.phase, Some(UiDispatchPhase::Target));
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == close_button && *reason == UiFocusEffectReason::Navigation
    ));
    assert!(result.diagnostics.route_steps.iter().any(|step| {
        step.phase == UiDispatchPhase::Target
            && step.target == Some(track_button)
            && step.handler == Some(track_button)
            && step.disposition == UiDispatchDisposition::Handled
            && step.effect_count == 1
    }));
    assert_eq!(manager.surface().focus.focused_inputs.len(), 1);
    assert_eq!(
        manager.surface().focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        manager.surface().focus.focused_inputs[0].focused,
        close_button
    );
    assert_eq!(
        manager.surface().focus.focused_inputs[0].handled_by,
        Some(track_button)
    );
    assert!(manager.surface().focus.focused_inputs[0].accepted);
    assert!(result.component_events.is_empty());
    assert!(!manager.surface().dirty_flags().any());
}

#[test]
fn runtime_ui_manager_routes_runtime_gamepad_axis_right_through_focused_analog_navigation_path() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    let track_button = node_id_by_control_id(&manager, "TrackQuestButton");
    let close_button = node_id_by_control_id(&manager, "CloseQuestLogButton");
    let track_frame = manager
        .surface()
        .surface_frame()
        .arranged_tree
        .get(track_button)
        .expect("track button should have an arranged frame")
        .frame;
    let focus_point = UiPoint::new(
        track_frame.x + track_frame.width * 0.5,
        track_frame.y + track_frame.height * 0.5,
    );
    manager.register_pointer_handler(track_button, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::set_focus()
    });
    manager.register_navigation_handler(track_button, UiNavigationEventKind::Right, move |_| {
        UiNavigationDispatchEffect::focus(close_button)
    });

    manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                focus_point.x,
                focus_point.y,
            ),
        )
        .expect("runtime mouse-button ABI event should focus the track button");
    assert_eq!(manager.surface().focus.focused, Some(track_button));

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::gamepad_axis(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                2,
                ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
                0.75,
            ),
        )
        .expect("runtime gamepad axis ABI event should route through focused analog navigation");

    match &result.event {
        UiInputEvent::Analog(analog) => {
            assert_eq!(
                analog.metadata.window_id,
                Some(UiWindowId::new("runtime.main"))
            );
            assert_eq!(analog.metadata.timestamp, UiInputTimestamp::from_micros(42));
            assert_eq!(analog.metadata.sequence, UiInputSequence::new(7));
            assert_eq!(analog.control, "Gamepad_LeftX");
            assert_eq!(analog.value, 0.75);
        }
        other => panic!(
            "expected runtime gamepad axis ABI event to normalize into analog input, got {other:?}"
        ),
    }
    assert_eq!(
        manager
            .surface()
            .input
            .analog_controls
            .get("Gamepad_LeftX")
            .map(|state| state.value),
        Some(0.75)
    );
    assert_eq!(manager.surface().focus.focused, Some(close_button));
    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(result.diagnostics.route_target, Some(track_button));
    assert_eq!(result.diagnostics.route_trace.target, Some(track_button));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("analog.navigation")
    );
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "analog_navigation=Right"));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(track_button));
    assert_eq!(result.reply.phase, Some(UiDispatchPhase::Target));
    assert_eq!(result.applied_effects.len(), 1);
    assert!(matches!(
        &result.applied_effects[0].effect,
        UiDispatchEffect::SetFocus { target, reason }
            if *target == close_button && *reason == UiFocusEffectReason::Navigation
    ));
    assert!(result.diagnostics.route_steps.iter().any(|step| {
        step.phase == UiDispatchPhase::Target
            && step.target == Some(track_button)
            && step.handler == Some(track_button)
            && step.disposition == UiDispatchDisposition::Handled
            && step.effect_count == 1
    }));
    assert_eq!(manager.surface().focus.focused_inputs.len(), 1);
    assert_eq!(
        manager.surface().focus.focused_inputs[0].kind,
        UiFocusedInputKind::Navigation
    );
    assert_eq!(
        manager.surface().focus.focused_inputs[0].focused,
        close_button
    );
    assert_eq!(
        manager.surface().focus.focused_inputs[0].handled_by,
        Some(track_button)
    );
    assert!(manager.surface().focus.focused_inputs[0].accepted);
    assert!(result.component_events.is_empty());
    assert!(!manager.surface().dirty_flags().any());
}
