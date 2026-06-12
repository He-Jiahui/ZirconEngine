use super::*;

#[test]
fn runtime_ui_manager_dispatches_normalized_pointer_event_through_owned_router() {
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

    let mut metadata = input_metadata();
    metadata.pointer_id = Some(UiPointerId::new(17));
    let result = manager
        .dispatch_input_event(UiInputEvent::Pointer(UiPointerInputEvent {
            metadata,
            event: zircon_runtime_interface::ui::dispatch::UiPointerEvent::new(
                UiPointerEventKind::Down,
                point,
            )
            .with_button(UiPointerButton::Primary),
            precise_scroll: None,
        }))
        .expect("normalized pointer input should route through manager-owned dispatchers");

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
            && pointer_id.0 == 17
            && *reason == UiPointerCaptureReason::Press
    )));
    assert_eq!(manager.surface().focus.captured, Some(target_node));
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(
        !manager.surface().dirty_flags().any(),
        "manager-level normalized input dispatch should consume dirty domains before frame capture"
    );
}
#[test]
fn runtime_ui_manager_dispatches_normalized_navigation_event_through_owned_router() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let root_node = manager.surface().tree.roots[0];
    manager.register_navigation_handler(root_node, UiNavigationEventKind::Activate, |_| {
        UiNavigationDispatchEffect::Handled
    });

    let result = manager
        .dispatch_input_event(UiInputEvent::Navigation(UiNavigationInputEvent {
            metadata: input_metadata(),
            kind: UiNavigationEventKind::Activate,
        }))
        .expect("normalized navigation input should route through manager-owned dispatchers");

    assert_eq!(
        result.diagnostics.route_policy,
        UiInputRoutePolicy::FocusPath
    );
    assert_eq!(result.diagnostics.route_target, Some(root_node));
    assert_eq!(
        result.diagnostics.handled_phase.as_deref(),
        Some("navigation")
    );
    assert_eq!(result.reply.handler, Some(root_node));
    assert!(
        !manager.surface().dirty_flags().any(),
        "manager-level normalized navigation dispatch should not leave stale dirty domains"
    );
}
#[test]
fn runtime_ui_manager_dispatches_normalized_input_batch_in_order() {
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

    let mut pointer_metadata = input_metadata();
    pointer_metadata.pointer_id = Some(UiPointerId::new(31));
    let results = manager
        .dispatch_input_batch([
            UiInputEvent::Navigation(UiNavigationInputEvent {
                metadata: input_metadata(),
                kind: UiNavigationEventKind::Next,
            }),
            UiInputEvent::Pointer(UiPointerInputEvent {
                metadata: pointer_metadata,
                event: zircon_runtime_interface::ui::dispatch::UiPointerEvent::new(
                    UiPointerEventKind::Down,
                    point,
                )
                .with_button(UiPointerButton::Primary),
                precise_scroll: None,
            }),
        ])
        .expect("normalized input batch should reuse the manager-owned router in order");

    assert_eq!(results.len(), 2);
    assert!(matches!(
        &results[0].event,
        UiInputEvent::Navigation(UiNavigationInputEvent {
            kind: UiNavigationEventKind::Next,
            ..
        })
    ));
    assert!(matches!(&results[1].event, UiInputEvent::Pointer(_)));
    assert_eq!(
        results[1].diagnostics.route_policy,
        UiInputRoutePolicy::Bubble
    );
    assert_eq!(results[1].diagnostics.route_target, Some(target_node));
    assert_eq!(results[1].reply.handler, Some(target_node));
    assert!(results[1].reply.effects.iter().any(|effect| matches!(
        effect,
        UiDispatchEffect::CapturePointer {
            target,
            pointer_id,
            reason,
        } if *target == target_node
            && pointer_id.0 == 31
            && *reason == UiPointerCaptureReason::Press
    )));
    assert_eq!(manager.surface().focus.captured, Some(target_node));
    assert!(!manager.surface().dirty_flags().any());
}
#[test]
fn runtime_ui_manager_normalized_input_batch_reports_failing_index() {
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
    let mut pointer_metadata = input_metadata();
    pointer_metadata.pointer_id = Some(UiPointerId::new(41));

    let err = match manager.dispatch_input_batch([
        UiInputEvent::Pointer(UiPointerInputEvent {
            metadata: pointer_metadata,
            event: zircon_runtime_interface::ui::dispatch::UiPointerEvent::new(
                UiPointerEventKind::Down,
                point,
            )
            .with_button(UiPointerButton::Primary),
            precise_scroll: None,
        }),
        UiInputEvent::Navigation(UiNavigationInputEvent {
            metadata: input_metadata(),
            kind: UiNavigationEventKind::Activate,
        }),
    ]) {
        Ok(_) => panic!("normalized input batch should report the failing navigation event"),
        Err(err) => err,
    };

    let message = err.to_string();
    assert!(message.contains("normalized input batch index 1"));
    assert!(message.contains("ui tree is missing node"));
    assert_eq!(
        manager.surface().focus.captured,
        Some(target_node),
        "accepted prefix pointer input should keep its capture before the later batch failure"
    );
    assert!(!manager.surface().dirty_flags().any());
}
