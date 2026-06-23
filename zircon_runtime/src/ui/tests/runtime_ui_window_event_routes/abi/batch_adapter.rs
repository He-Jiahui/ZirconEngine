use super::*;

#[test]
fn runtime_ui_manager_dispatches_runtime_event_batch_through_window_adapter() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::QuestLogDialog)
        .expect("quest log runtime fixture should load");

    let results = manager
        .dispatch_runtime_event_batch(
            &runtime_event_context(),
            [
                ZrRuntimeEventV1::viewport_metrics(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    viewport(),
                    ZrRuntimeViewportMetricsV1::new(
                        ZrRuntimeViewportSizeV1::new(800, 450),
                        2.0,
                        ZrRuntimeViewportSizeV1::new(1600, 900),
                    ),
                ),
                ZrRuntimeEventV1::window_occluded(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport(), true),
                ZrRuntimeEventV1::window_close_requested(ZIRCON_RUNTIME_ABI_VERSION_V1, viewport()),
            ],
        )
        .expect("runtime UI manager should adapt ABI events into the window pump");

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|result| result
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_input_pump")));
    assert_eq!(
        results[0].diagnostics.route_policy,
        UiInputRoutePolicy::DefaultAction
    );
    assert!(results[0]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_layout_metrics_dirty"));
    assert!(results[1]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_occluded"));
    assert!(results[2]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_close_requested"));

    let window_state = &manager.surface().surface_frame().window_state;
    let metrics = window_state
        .metrics
        .as_ref()
        .expect("viewport metrics should be retained on the runtime frame");
    assert_eq!(metrics.logical_size, UiSize::new(800.0, 450.0));
    assert_eq!(metrics.scale_factor, 2.0);
    assert_eq!(window_state.occluded, Some(true));
    assert!(window_state.close_requested);
    assert!(
        !manager.surface().dirty_flags().any(),
        "runtime UI manager should consume adapter-originated dirty domains before frame capture"
    );
}

#[test]
fn runtime_ui_manager_runtime_event_batch_rebuilds_before_followup_pointer_input() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let resume_button = node_id_by_control_id(&manager, "ResumeButton");
    manager.register_pointer_handler(resume_button, UiPointerEventKind::Down, move |context| {
        assert_eq!(
            context.route.hit_path.target,
            Some(resume_button),
            "ABI pointer input in the same runtime-event batch should see the resized layout"
        );
        UiPointerDispatchEffect::handled()
    });

    let resized_resume_point = UiPoint::new(500.0, 219.0);
    assert_ne!(
        manager.surface().hit_test(resized_resume_point).top_hit,
        Some(resume_button),
        "pre-resize hit test should not already target the resized button point"
    );

    let results = manager
        .dispatch_runtime_event_batch(
            &runtime_event_context(),
            [
                ZrRuntimeEventV1::viewport_metrics(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    viewport(),
                    ZrRuntimeViewportMetricsV1::new(
                        ZrRuntimeViewportSizeV1::new(800, 450),
                        2.0,
                        ZrRuntimeViewportSizeV1::new(1600, 900),
                    ),
                ),
                ZrRuntimeEventV1::mouse_button(
                    ZIRCON_RUNTIME_ABI_VERSION_V1,
                    viewport(),
                    ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                    resized_resume_point.x,
                    resized_resume_point.y,
                ),
            ],
        )
        .expect("runtime-event batch should rebuild before routing follow-up pointer input");

    assert_eq!(results.len(), 2);
    assert!(results[0]
        .diagnostics
        .notes
        .iter()
        .any(|note| note == "window_layout_metrics_dirty"));
    assert_eq!(results[1].diagnostics.route_target, Some(resume_button));
    assert_eq!(results[1].reply.handler, Some(resume_button));
    assert_eq!(manager.build_frame().viewport_size, UVec2::new(800, 450));
    assert!(!manager.surface().dirty_flags().any());
}

#[test]
fn runtime_ui_manager_runtime_event_batch_keeps_prior_events_when_later_adapter_fails() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let err = match manager.dispatch_runtime_event_batch(
        &runtime_event_context(),
        [
            ZrRuntimeEventV1::viewport_metrics(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZrRuntimeViewportMetricsV1::new(
                    ZrRuntimeViewportSizeV1::new(800, 450),
                    2.0,
                    ZrRuntimeViewportSizeV1::new(1600, 900),
                ),
            ),
            ZrRuntimeEventV1::window_theme_changed(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_WINDOW_THEME_DARK_V1,
            ),
        ],
    ) {
        Ok(_) => {
            panic!("runtime-event batch should report the later adapter error after prior events apply")
        }
        Err(err) => err,
    };

    assert!(err
        .to_string()
        .contains("has no UI window/input pump equivalent"));
    assert!(err.to_string().contains("runtime event batch index 1"));
    let window_state = &manager.surface().surface_frame().window_state;
    let metrics = window_state
        .metrics
        .as_ref()
        .expect("accepted viewport metrics should survive a later ABI adapter error");
    assert_eq!(metrics.logical_size, UiSize::new(800.0, 450.0));
    assert_eq!(metrics.scale_factor, 2.0);
    assert_eq!(manager.build_frame().viewport_size, UVec2::new(800, 450));
    assert!(
        !manager.surface().dirty_flags().any(),
        "accepted prefix events should still be reduced through the manager rebuild path"
    );
}

#[test]
fn runtime_ui_manager_runtime_event_batch_reports_dispatch_error_index_after_adapter_success() {
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
        UiPointerDispatchEffect::set_focus(true)
    });
    manager.register_navigation_handler(target_node, UiNavigationEventKind::Right, |_| {
        UiNavigationDispatchEffect::focus(UiNodeId::new(u64::MAX))
    });

    let err = match manager.dispatch_runtime_event_batch(
        &runtime_event_context(),
        [
            ZrRuntimeEventV1::mouse_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                point.x,
                point.y,
            ),
            ZrRuntimeEventV1::gamepad_button(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                2,
                ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
                ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
                1.0,
            ),
        ],
    ) {
        Ok(_) => panic!("runtime-event batch should report the later dispatch error"),
        Err(err) => err,
    };

    let message = err.to_string();
    assert!(message.contains("runtime event batch index 1"));
    assert!(message.contains("ui tree is missing node"));
    assert_eq!(
        manager.surface().focus.focused,
        Some(target_node),
        "accepted ABI pointer prefix should keep focus before the later dispatch failure"
    );
    assert_eq!(manager.surface().input.last_cursor_point(), Some(point));
    assert!(!manager.surface().dirty_flags().any());
}

#[test]
fn runtime_ui_manager_reports_runtime_event_adapter_errors_without_surface_mutation() {
    let mut manager = RuntimeUiManager::new(UVec2::new(640, 360));
    manager
        .load_builtin_fixture(RuntimeUiFixture::PauseMenu)
        .expect("pause menu runtime fixture should load");

    let before = manager.surface().surface_frame().window_state.clone();
    let err = manager
        .dispatch_runtime_event(
            &runtime_event_context(),
            ZrRuntimeEventV1::window_theme_changed(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport(),
                ZR_RUNTIME_WINDOW_THEME_DARK_V1,
            ),
        )
        .expect_err("theme changes do not have a shared pump equivalent yet");

    assert!(err
        .to_string()
        .contains("has no UI window/input pump equivalent"));
    assert_eq!(manager.surface().surface_frame().window_state, before);
}
