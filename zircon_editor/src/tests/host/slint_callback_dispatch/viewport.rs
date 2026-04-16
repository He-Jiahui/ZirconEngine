use super::support::*;

#[test]
fn builtin_viewport_toolbar_set_tool_dispatches_dynamic_binding_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_viewport_tool");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "SetTool",
        UiEventKind::Change,
        vec![zircon_ui::UiBindingValue::string("Scale")],
    )
    .expect("viewport toolbar control should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::SetTool {
            tool: SceneViewportTool::Scale,
        })
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_viewport_toolbar_frame_selection_dispatches_static_binding_from_template() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_template_bridge_viewport_frame");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();

    let effects = dispatch_builtin_viewport_toolbar_control(
        &harness.runtime,
        &bridge,
        "FrameSelection",
        UiEventKind::Click,
        Vec::new(),
    )
    .expect("viewport toolbar frame selection should resolve through template bridge")
    .unwrap();

    let journal = harness.runtime.journal();
    assert_eq!(
        journal.records().last().unwrap().event,
        EditorEvent::Viewport(EditorViewportEvent::FrameSelection)
    );
    assert!(effects.render_dirty);
    assert!(effects.presentation_dirty);
}

#[test]
fn builtin_viewport_toolbar_set_tool_matches_legacy_viewport_command_dispatch() {
    let _guard = env_lock().lock().unwrap();

    let legacy_harness = EventRuntimeHarness::new("zircon_slint_parity_viewport_tool_legacy");
    let legacy_effects = dispatch_viewport_command(
        &legacy_harness.runtime,
        ViewportCommand::SetTool(SceneViewportTool::Scale),
    )
    .unwrap();
    let legacy_record = legacy_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    let builtin_harness = EventRuntimeHarness::new("zircon_slint_parity_viewport_tool_builtin");
    let bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();
    let builtin_effects = dispatch_builtin_viewport_toolbar_control(
        &builtin_harness.runtime,
        &bridge,
        "SetTool",
        UiEventKind::Change,
        vec![zircon_ui::UiBindingValue::string("Scale")],
    )
    .expect("templated viewport tool control should resolve")
    .unwrap();
    let builtin_record = builtin_harness
        .runtime
        .journal()
        .records()
        .last()
        .unwrap()
        .clone();

    assert_eq!(builtin_effects, legacy_effects);
    assert_eq!(builtin_record, legacy_record);
}

#[test]
fn shared_viewport_pointer_bridge_maps_secondary_button_to_right_pressed_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_secondary");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 320.0, 180.0));

    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(24.0, 32.0))
            .with_button(UiPointerButton::Secondary),
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::Viewport(EditorViewportEvent::RightPressed { x: 24.0, y: 32.0 })
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
}

#[test]
fn shared_viewport_pointer_bridge_keeps_move_and_up_routed_to_captured_viewport() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_capture");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 100.0, 100.0));

    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(10.0, 10.0))
            .with_button(UiPointerButton::Primary),
    )
    .unwrap();
    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Move, UiPoint::new(180.0, 180.0)),
    )
    .unwrap();
    dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Up, UiPoint::new(180.0, 180.0))
            .with_button(UiPointerButton::Primary),
    )
    .unwrap();

    let events: Vec<_> = harness
        .runtime
        .journal()
        .records()
        .iter()
        .rev()
        .take(3)
        .map(|record| record.event.clone())
        .collect();
    assert_eq!(
        events.into_iter().rev().collect::<Vec<_>>(),
        vec![
            EditorEvent::Viewport(EditorViewportEvent::LeftPressed { x: 10.0, y: 10.0 }),
            EditorEvent::Viewport(EditorViewportEvent::PointerMoved { x: 180.0, y: 180.0 }),
            EditorEvent::Viewport(EditorViewportEvent::LeftReleased),
        ]
    );
}

#[test]
fn shared_viewport_pointer_bridge_maps_scroll_to_viewport_scrolled_event() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_scroll");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 160.0, 90.0));

    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Scroll, UiPoint::new(32.0, 24.0))
            .with_scroll_delta(-48.0),
    )
    .unwrap();

    let journal = harness.runtime.journal();
    let record = journal.records().last().unwrap();
    assert_eq!(
        record.event,
        EditorEvent::Viewport(EditorViewportEvent::Scrolled { delta: -48.0 })
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
}

#[test]
fn shared_viewport_pointer_bridge_respects_updated_viewport_frame_bounds() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_shared_pointer_frame");
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 160.0, 90.0));
    bridge.update_viewport_frame(UiFrame::new(0.0, 0.0, 80.0, 60.0));

    let record_count_before = harness.runtime.journal().records().len();
    let effects = dispatch_viewport_pointer_event(
        &harness.runtime,
        &mut bridge,
        UiPointerEvent::new(UiPointerEventKind::Down, UiPoint::new(120.0, 70.0))
            .with_button(UiPointerButton::Primary),
    )
    .unwrap();

    assert_eq!(
        effects,
        crate::host::slint_host::event_bridge::SlintDispatchEffects::default()
    );
    assert_eq!(
        harness.runtime.journal().records().len(),
        record_count_before
    );
}

#[test]
fn shared_viewport_surface_replaces_legacy_direct_pointer_callback_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let app = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app.rs"
    ));
    let viewport = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/viewport.rs"
    ));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/callback_wiring.rs"
    ));

    for needle in [
        "callback viewport_pointer_moved(",
        "callback viewport_left_pressed(",
        "callback viewport_left_released(",
        "callback viewport_right_pressed(",
        "callback viewport_right_released(",
        "callback viewport_middle_pressed(",
        "callback viewport_middle_released(",
        "callback viewport_scrolled(",
        "viewport_pointer_moved(x, y) =>",
        "viewport_left_pressed(x, y) =>",
        "viewport_left_released() =>",
        "viewport_right_pressed(x, y) =>",
        "viewport_right_released() =>",
        "viewport_middle_pressed(x, y) =>",
        "viewport_middle_released() =>",
        "viewport_scrolled(delta) =>",
    ] {
        assert!(
            !workbench.contains(needle),
            "workbench shell still exposes legacy direct viewport callback `{needle}`"
        );
    }

    assert!(
        workbench.contains("callback viewport_pointer_event("),
        "workbench shell must expose unified shared viewport pointer callback"
    );

    for needle in [
        "ui.on_viewport_pointer_moved(",
        "ui.on_viewport_left_pressed(",
        "ui.on_viewport_left_released(",
        "ui.on_viewport_right_pressed(",
        "ui.on_viewport_right_released(",
        "ui.on_viewport_middle_pressed(",
        "ui.on_viewport_middle_released(",
        "ui.on_viewport_scrolled(",
    ] {
        assert!(
            !wiring.contains(needle),
            "slint host wiring still registers legacy direct viewport callback `{needle}`"
        );
    }

    assert!(
        wiring.contains("ui.on_viewport_pointer_event("),
        "slint host wiring must register unified shared viewport callback"
    );

    for needle in [
        "InputManager",
        "InputButton",
        "InputEvent",
        "submit_event(InputEvent::CursorMoved",
        "submit_event(InputEvent::ButtonPressed",
        "submit_event(InputEvent::ButtonReleased",
        "submit_event(InputEvent::WheelScrolled",
    ] {
        assert!(
            !app.contains(needle) && !viewport.contains(needle),
            "slint viewport host still depends on legacy raw input manager path `{needle}`"
        );
    }

    assert!(
        viewport.contains("dispatch_viewport_pointer_event("),
        "slint viewport host must dispatch through shared viewport pointer bridge"
    );
}

#[test]
fn typed_viewport_command_dispatch_updates_render_packet_without_pointer_bridge() {
    let _guard = env_lock().lock().unwrap();

    let harness = EventRuntimeHarness::new("zircon_slint_viewport_command_dispatch");

    let effects = dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetDisplayMode(DisplayMode::WireOnly),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetGridMode(GridMode::VisibleAndSnap),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetProjectionMode(ProjectionMode::Orthographic),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::SetTool(SceneViewportTool::Rotate),
    )
    .unwrap();
    dispatch_viewport_command(
        &harness.runtime,
        ViewportCommand::AlignView(ViewOrientation::NegX),
    )
    .unwrap();

    let snapshot = harness.runtime.editor_snapshot();
    let packet = harness.runtime.render_snapshot().expect("render packet");

    assert_eq!(
        snapshot.scene_viewport_settings.display_mode,
        DisplayMode::WireOnly
    );
    assert_eq!(
        snapshot.scene_viewport_settings.grid_mode,
        GridMode::VisibleAndSnap
    );
    assert_eq!(
        snapshot.scene_viewport_settings.projection_mode,
        ProjectionMode::Orthographic
    );
    assert_eq!(
        snapshot.scene_viewport_settings.tool,
        SceneViewportTool::Rotate
    );
    assert_eq!(
        snapshot.scene_viewport_settings.view_orientation,
        ViewOrientation::NegX
    );
    assert_eq!(packet.overlays.display_mode, DisplayMode::WireOnly);
    assert_eq!(
        packet.overlays.grid.as_ref().map(|grid| grid.snap_enabled),
        Some(true)
    );
    assert!(effects.presentation_dirty);
    assert!(effects.render_dirty);
}
