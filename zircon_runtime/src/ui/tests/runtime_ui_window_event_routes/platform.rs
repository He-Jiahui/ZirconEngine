use super::*;

#[test]
fn runtime_ui_manager_normalizes_platform_pointer_event_through_owned_router() {
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
        .dispatch_platform_input_event(UiWindowPlatformInputEvent::mouse_button_down(
            input_context(),
            UiPointerButton::Primary,
            point,
        ))
        .expect("platform pointer input should normalize through the manager boundary");

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(target_node));
    assert_eq!(result.reply.handler, Some(target_node));
    assert!(result.reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::CapturePointer {
            target,
            pointer_id,
            reason,
        } if *target == target_node
            && pointer_id.0 == 29
            && *reason == UiPointerCaptureReason::Press
    )));
    assert_eq!(manager.surface().focus.captured, Some(target_node));
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(!manager.surface().dirty_flags().any());
}
#[test]
fn runtime_ui_manager_platform_wheel_scrolls_inventory_list_through_owned_router() {
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
        .dispatch_platform_input_event(UiWindowPlatformInputEvent::mouse_wheel_delta(
            input_context(),
            point,
            UiPreciseScrollDelta::pixels(0.0, 48.0),
        ))
        .expect("platform wheel input should normalize through the manager boundary");

    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(inventory_list));
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
    match &result.event {
        UiInputEvent::Pointer(pointer) => {
            assert_eq!(pointer.event.kind, UiPointerEventKind::Scroll);
            assert_eq!(pointer.event.scroll_delta, 48.0);
            assert_eq!(
                pointer.precise_scroll,
                Some(UiPreciseScrollDelta::pixels(0.0, 48.0))
            );
        }
        other => panic!("expected wheel input to normalize into pointer input, got {other:?}"),
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
        "platform wheel input should scroll the retained inventory list"
    );
    assert_eq!(after_scroll.offset, 48.0);
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(
        !manager.surface().dirty_flags().any(),
        "runtime UI manager should consume wheel-induced dirty domains before frame capture"
    );
}
#[test]
fn runtime_ui_manager_pointer_preview_phase_stops_before_target_handler() {
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
    let root_node = manager.surface().tree.roots[0];
    let target_calls = Arc::new(AtomicUsize::new(0));
    let target_calls_for_handler = Arc::clone(&target_calls);

    manager.register_pointer_phase_handler(
        root_node,
        UiPointerEventKind::Down,
        UiDispatchPhase::PreviewTunnel,
        move |context| {
            assert_eq!(context.phase, UiDispatchPhase::PreviewTunnel);
            assert_eq!(context.node_id, root_node);
            assert_eq!(context.route.hit_path.target, Some(target_node));
            UiPointerDispatchEffect::handled()
        },
    );
    manager.register_pointer_handler(target_node, UiPointerEventKind::Down, move |_| {
        target_calls_for_handler.fetch_add(1, Ordering::SeqCst);
        UiPointerDispatchEffect::capture()
    });

    let result = manager
        .dispatch_platform_input_event(UiWindowPlatformInputEvent::mouse_button_down(
            input_context(),
            UiPointerButton::Primary,
            point,
        ))
        .expect("platform pointer input should route through manager-owned phase handlers");

    assert_eq!(target_calls.load(Ordering::SeqCst), 0);
    assert_eq!(result.diagnostics.route_policy, UiInputRoutePolicy::Bubble);
    assert_eq!(result.diagnostics.route_target, Some(target_node));
    assert_eq!(result.diagnostics.handled_phase.as_deref(), Some("pointer"));
    assert_eq!(result.reply.handler, Some(root_node));
    assert_eq!(result.reply.phase, Some(UiDispatchPhase::PreviewTunnel));
    assert_eq!(result.diagnostics.route_steps.len(), 1);
    assert_eq!(
        result.diagnostics.route_steps[0].phase,
        UiDispatchPhase::PreviewTunnel
    );
    assert_eq!(result.diagnostics.route_steps[0].target, Some(root_node));
    assert_eq!(result.diagnostics.route_steps[0].handler, Some(root_node));
    assert!(result.diagnostics.route_steps[0].stopped);
    assert!(result.reply.effects.is_empty());
    assert_ne!(manager.surface().focus.captured, Some(target_node));
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(!manager.surface().dirty_flags().any());
}
#[test]
fn runtime_ui_manager_platform_input_batch_reports_failing_index() {
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
    let root_node = manager.surface().tree.roots[0];
    manager.register_navigation_handler(root_node, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::focus(UiNodeId::new(u64::MAX))
    });

    let err = match manager.dispatch_platform_input_batch([
        UiWindowPlatformInputEvent::mouse_button_down(
            input_context(),
            UiPointerButton::Primary,
            point,
        ),
        UiWindowPlatformInputEvent::navigation(
            navigation_input_context(),
            UiNavigationEventKind::Activate,
        ),
    ]) {
        Ok(_) => panic!("platform input batch should report the failing normalized event"),
        Err(err) => err,
    };

    let message = err.to_string();
    assert!(message.contains("platform input batch index 1"));
    assert!(message.contains("ui tree is missing node"));
    assert_eq!(
        manager.surface().focus.captured,
        Some(target_node),
        "accepted platform prefix pointer input should keep its capture before the later batch failure"
    );
    assert!(!manager.surface().dirty_flags().any());
}
#[test]
fn runtime_ui_manager_dispatches_platform_input_batch_in_order() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let root_node = manager.surface().tree.roots[0];
    manager.register_navigation_handler(root_node, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::Handled
    });

    let results = manager
        .dispatch_platform_input_batch([
            UiWindowPlatformInputEvent::navigation(
                navigation_input_context(),
                UiNavigationEventKind::Next,
            ),
            UiWindowPlatformInputEvent::navigation(
                navigation_input_context(),
                UiNavigationEventKind::Activate,
            ),
        ])
        .expect("platform input batch should enter the manager-owned router in order");

    assert_eq!(results.len(), 2);
    assert_eq!(
        results[0].event,
        UiInputEvent::Navigation(UiNavigationInputEvent {
            metadata: input_metadata(),
            kind: UiNavigationEventKind::Next,
        })
    );
    assert_eq!(
        results[1].event,
        UiInputEvent::Navigation(UiNavigationInputEvent {
            metadata: input_metadata(),
            kind: UiNavigationEventKind::Activate,
        })
    );
    assert_eq!(
        results[0].diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(
        results[1].diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert!(results[1]
        .diagnostics
        .route_trace
        .bubble_path
        .contains(&root_node));
    assert_eq!(results[1].reply.handler, Some(root_node));
    assert_eq!(
        results[1].diagnostics.handled_phase.as_deref(),
        Some("navigation")
    );
    assert!(!manager.surface().dirty_flags().any());
}
