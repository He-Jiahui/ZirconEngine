use super::*;

#[test]
fn runtime_ui_manager_routes_runtime_pointer_events_through_owned_dispatcher() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let point = UiPoint::new(320.0, 180.0);
    let target_node = manager
        .surface()
        .hit_test(point)
        .top_hit
        .expect("runtime fixture should expose a pointer hit target");
    manager.register_pointer_handler(target_node, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::capture()
    });

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                point.x,
                point.y,
            ),
        )
        .expect("runtime pointer ABI event should route through manager-owned dispatchers");

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(target_node));
    assert_eq!(result.diagnostics.route_trace.target, Some(target_node));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.reply.handler, Some(target_node));
    assert!(result
        .diagnostics
        .route_trace
        .bubble_path
        .contains(&target_node));
    assert!(result.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::CapturePointer {
            target,
            pointer_id,
            reason,
        } if *target == target_node
            && pointer_id.0 == 0
            && *reason == UiPointerCaptureReason::Press
    )));
    assert_eq!(manager.surface().focus.captured, Some(target_node));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| { note == "pointer_source=Mouse" }));
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(
        !manager.surface().dirty_flags().any(),
        "runtime UI manager should finish adapter-originated dispatch without leaving stale dirty domains"
    );
}

#[test]
fn runtime_ui_manager_routes_runtime_wheel_at_point_through_owned_scroll_route() {
    let mut manager = RuntimeUiManager::new(UVec2::new(960, 540));
    manager
        .load_builtin_fixture(RuntimeUiFixture::InventoryList)
        .expect("inventory runtime fixture should load");

    let inventory_list = node_id_by_control_id(&manager, "InventoryList");
    let list_frame = manager
        .surface()
        .surface_frame()
        .arranged_tree
        .get(inventory_list)
        .expect("inventory list should have an arranged frame")
        .frame;
    let point = UiPoint::new(
        list_frame.x + list_frame.width * 0.5,
        list_frame.y + list_frame.height * 0.5,
    );
    let before_scroll = manager
        .surface()
        .tree
        .nodes
        .get(&inventory_list)
        .and_then(|node| node.scroll_state)
        .expect("inventory list should retain scroll state");

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::mouse_wheel_delta_at(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
                point.x,
                point.y,
                0.0,
                48.0,
            ),
        )
        .expect("runtime wheel ABI event should retain point and scroll through manager route");

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(inventory_list));
    assert_eq!(result.diagnostics.route_trace.target, Some(inventory_list));
    assert_eq!(result.reply.handler, Some(inventory_list));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert!(result
        .diagnostics
        .route_trace
        .bubble_path
        .contains(&inventory_list));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "scroll_delta=48"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_input_pump"));
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(
                pointer.metadata.window_id,
                Some(UiWindowId::new("runtime.main"))
            );
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Mouse);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
            assert_eq!(pointer.event.point, point);
            assert_eq!(pointer.event.scroll_delta, 48.0);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta::pixels(0.0, 48.0))
            );
        }
        other => panic!(
            "expected runtime wheel ABI event to normalize into pointer input, got {other:?}"
        ),
    }

    let after_scroll = manager
        .surface()
        .tree
        .nodes
        .get(&inventory_list)
        .and_then(|node| node.scroll_state)
        .expect("inventory list should keep scroll state after wheel input");
    assert!(
        after_scroll.offset > before_scroll.offset,
        "runtime wheel ABI input should scroll the retained inventory list"
    );
    assert_eq!(after_scroll.offset, 48.0);
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(
        !manager.surface().dirty_flags().any(),
        "runtime UI manager should consume wheel-induced dirty domains before frame capture"
    );
}

#[test]
fn runtime_ui_manager_routes_runtime_pointer_moved_through_window_hover_pump() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let point = UiPoint::new(320.0, 180.0);
    let target_node = manager
        .surface()
        .hit_test(point)
        .top_hit
        .expect("runtime fixture should expose a pointer-move hit target");
    let handler_calls = Arc::new(AtomicUsize::new(0));
    let handler_calls_for_handler = Arc::clone(&handler_calls);
    manager.register_pointer_handler(target_node, UiPointerEventKind::Move, move |context| {
        handler_calls_for_handler.fetch_add(1, Ordering::SeqCst);
        assert_eq!(context.phase, UiDispatchPhase::Target);
        assert_eq!(context.node_id, target_node);
        assert_eq!(context.route.target, Some(target_node));
        assert_eq!(context.route.hit_path.target, Some(target_node));
        UiPointerDispatchEffect::handled()
    });

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::pointer_moved(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                point.x,
                point.y,
            ),
        )
        .expect("runtime pointer-move ABI event should route through the manager window pump");

    assert_eq!(handler_calls.load(Ordering::SeqCst), 1);
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(
                pointer.metadata.window_id,
                Some(UiWindowId::new("runtime.main"))
            );
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Mouse);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Move);
            assert_eq!(pointer.event.point, point);
            assert_eq!(pointer.precise_scroll, None);
        }
        other => {
            panic!("expected pointer-move ABI event to normalize into pointer input, got {other:?}")
        }
    }
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(result.diagnostics.route_target, Some(target_node));
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(target_node)
    );
    assert_eq!(result.diagnostics.route_trace.target, Some(target_node));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(target_node));
    assert!(result
        .diagnostics
        .route_steps
        .iter()
        .any(|step| step.phase == UiDispatchPhase::Direct && step.target == Some(target_node)));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "pointer_source=Mouse"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_normalized_input"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_input_pump"));
    assert_eq!(
        manager.surface().focus.hovered.first().copied(),
        Some(target_node)
    );
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(manager
        .surface()
        .component_state(target_node)
        .is_some_and(|state| state.flags.hovered));
    assert!(!manager.surface().dirty_flags().any());
}

#[test]
fn runtime_ui_manager_routes_runtime_cursor_left_through_window_pointer_cancel() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let point = UiPoint::new(320.0, 180.0);
    let target_node = manager
        .surface()
        .hit_test(point)
        .top_hit
        .expect("runtime fixture should expose a cursor-left hit target");

    manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::pointer_moved(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                point.x,
                point.y,
            ),
        )
        .expect("runtime pointer-move ABI event should seed cursor and hover state");
    assert_eq!(
        manager.surface().focus.hovered.first().copied(),
        Some(target_node)
    );
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(manager
        .surface()
        .component_state(target_node)
        .is_some_and(|state| state.flags.hovered));

    let cancel_calls = Arc::new(AtomicUsize::new(0));
    let cancel_calls_for_handler = Arc::clone(&cancel_calls);
    manager.register_pointer_handler(target_node, UiPointerEventKind::Cancel, move |context| {
        cancel_calls_for_handler.fetch_add(1, Ordering::SeqCst);
        assert_eq!(context.phase, UiDispatchPhase::Target);
        assert_eq!(context.node_id, target_node);
        assert_eq!(context.route.target, Some(target_node));
        assert_eq!(context.route.hit_path.target, Some(target_node));
        UiPointerDispatchEffect::handled()
    });

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::cursor_left(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport()),
        )
        .expect("runtime cursor-left ABI event should replay pointer cancel through the manager");

    assert_eq!(cancel_calls.load(Ordering::SeqCst), 1);
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(
                pointer.metadata.window_id,
                Some(UiWindowId::new("runtime.main"))
            );
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Mouse);
            assert_eq!(pointer.event.kind, UiPointerEventKind::Cancel);
            assert_eq!(pointer.event.point, point);
            assert_eq!(pointer.precise_scroll, None);
        }
        other => {
            panic!("expected cursor-left ABI event to normalize into pointer cancel, got {other:?}")
        }
    }
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Direct);
    assert_eq!(result.diagnostics.route_target, Some(target_node));
    assert_eq!(
        result.diagnostics.route_trace.direct_target,
        Some(target_node)
    );
    assert_eq!(result.diagnostics.route_trace.target, Some(target_node));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.reply.disposition, UiDispatchDisposition::Handled);
    assert_eq!(result.reply.handler, Some(target_node));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "pointer_source=Mouse"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_pointer_cancel"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_input_pump"));
    assert_eq!(manager.surface().input.last_cursor_point(), None);
    assert!(manager.surface().focus.hovered.is_empty());
    assert!(!manager
        .surface()
        .component_state(target_node)
        .is_some_and(|state| state.flags.hovered));
    assert!(!manager.surface().dirty_flags().any());
}

#[test]
fn runtime_ui_manager_routes_runtime_touch_events_through_owned_dispatcher() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let point = UiPoint::new(320.0, 180.0);
    let pointer_id = UiPointerId::new(73);
    let target_node = manager
        .surface()
        .hit_test(point)
        .top_hit
        .expect("runtime fixture should expose a touch hit target");
    manager.register_pointer_handler(target_node, UiPointerEventKind::Down, |_| {
        UiPointerDispatchEffect::capture()
    });

    let result = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::touch(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                pointer_id.0,
                ZR_RUNTIME_TOUCH_PHASE_STARTED_V1,
                point.x,
                point.y,
            ),
        )
        .expect("runtime touch ABI event should route through manager-owned dispatchers");

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(target_node));
    assert_eq!(result.diagnostics.route_trace.target, Some(target_node));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.reply.handler, Some(target_node));
    assert!(result
        .diagnostics
        .route_trace
        .bubble_path
        .contains(&target_node));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "touch_like_pointer"));
    assert!(result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "pointer_source=Touch"));
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.metadata.pointer_source, UiPointerSource::Touch);
            assert_eq!(pointer.metadata.pointer_id, Some(pointer_id));
            assert_eq!(pointer.event.kind, UiPointerEventKind::Down);
        }
        other => panic!("expected touch ABI event to normalize into pointer input, got {other:?}"),
    }
    assert!(result.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::CapturePointer {
            target,
            pointer_id: effect_pointer_id,
            reason,
        } if *target == target_node
            && *effect_pointer_id == pointer_id
            && *reason == UiPointerCaptureReason::Press
    )));
    assert_eq!(manager.surface().focus.captured, Some(target_node));
    assert_eq!(manager.surface().input.last_cursor_point(), None);
    assert!(
        !manager.surface().dirty_flags().any(),
        "runtime UI manager should finish touch adapter dispatch without leaving stale dirty domains"
    );
}
