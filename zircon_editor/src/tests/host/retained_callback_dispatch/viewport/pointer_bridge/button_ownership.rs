use super::*;
use crate::core::editing::engine::HistoryContextId;
use crate::core::editing::intent::EditorIntent;
use crate::scene::viewport::ViewportCameraSnapshot;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::{Transform, Vec2};

#[test]
fn shared_viewport_current_navigation_capture_survives_foreign_button_edges_outside() {
    assert_navigation_capture(false);
}

#[test]
fn shared_viewport_stale_navigation_capture_survives_foreign_button_edges_outside() {
    assert_navigation_capture(true);
}

fn assert_navigation_capture(stale: bool) {
    let _guard = env_lock().lock().unwrap();
    for owner in [UiPointerButton::Secondary, UiPointerButton::Middle] {
        let harness = EventRuntimeHarness::new("zircon_viewport_navigation_button_owner");
        let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 320.0, 180.0));
        let start = UiPoint::new(160.0, 90.0);
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Down,
            start,
            Some(owner),
        );
        {
            let shell = harness.runtime.shell().lock();
            shell.state.render_snapshot().unwrap();
            if stale {
                shell.state.world.expect_with_world_mut(|scene| {
                    scene.spawn_node(NodeKind::Empty).unwrap();
                });
            }
        }
        let before = camera(&harness);
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Down,
            start,
            Some(UiPointerButton::Primary),
        );
        assert!(matches!(
            harness.runtime.journal().records().last().unwrap().event,
            EditorEvent::Viewport(EditorViewportEvent::LeftPressed { .. })
        ));
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Move,
            UiPoint::new(400.0, 240.0),
            None,
        );
        let after_down = camera(&harness);
        assert_ne!(
            after_down, before,
            "foreign Down must not replace navigation"
        );
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Up,
            UiPoint::new(400.0, 240.0),
            Some(UiPointerButton::Primary),
        );
        assert_eq!(
            harness.runtime.journal().records().last().unwrap().event,
            EditorEvent::Viewport(EditorViewportEvent::LeftReleased)
        );
        let outside = UiPoint::new(460.0, 280.0);
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Move,
            outside,
            None,
        );
        assert_ne!(
            camera(&harness),
            after_down,
            "foreign Up must retain capture outside the viewport"
        );
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Up,
            outside,
            Some(owner),
        );
        assert_no_outside_delivery(&harness, &mut bridge, UiPoint::new(520.0, 320.0));
    }
}

#[test]
fn shared_viewport_primary_handle_capture_survives_foreign_edges_and_commits_once() {
    let _guard = env_lock().lock().unwrap();
    for foreign in [UiPointerButton::Secondary, UiPointerButton::Middle] {
        let harness = EventRuntimeHarness::new("zircon_viewport_handle_button_owner");
        let (cube, initial, press, direction, history) = prepare_handle(&harness);
        let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 1280.0, 720.0));
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Down,
            point(press),
            Some(UiPointerButton::Primary),
        );
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Down,
            point(press),
            Some(foreign),
        );
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Up,
            point(press),
            Some(foreign),
        );
        let outside = press + direction * 2000.0;
        assert!(outside.x < 0.0 || outside.x > 1280.0 || outside.y < 0.0 || outside.y > 720.0);
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Move,
            point(outside),
            None,
        );
        let preview = transform(&harness, cube);
        assert_ne!(preview, initial);
        {
            let shell = harness.runtime.shell().lock();
            assert!(shell.state.has_active_gizmo_interaction());
            assert_eq!(
                shell
                    .state
                    .transactions()
                    .history_status(history)
                    .unwrap()
                    .len,
                0
            );
        }
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Move,
            point(outside + direction * 64.0),
            None,
        );
        assert_ne!(
            transform(&harness, cube),
            preview,
            "capture must carry successive outside previews"
        );
        dispatch(
            &harness,
            &mut bridge,
            UiPointerEventKind::Up,
            point(outside),
            Some(UiPointerButton::Primary),
        );
        {
            let shell = harness.runtime.shell().lock();
            assert!(!shell.state.has_active_gizmo_interaction());
            assert_eq!(
                shell
                    .state
                    .transactions()
                    .history_status(history)
                    .unwrap()
                    .len,
                1
            );
        }
        assert_no_outside_delivery(&harness, &mut bridge, point(outside + direction * 128.0));
        harness
            .runtime
            .shell()
            .lock()
            .state
            .apply_intent(EditorIntent::Undo)
            .unwrap();
        assert_eq!(transform(&harness, cube), initial);
    }
}

#[test]
fn shared_viewport_cancel_clears_capture_and_restores_gizmo_world_and_history() {
    let _guard = env_lock().lock().unwrap();
    let harness = EventRuntimeHarness::new("zircon_viewport_cancel_button_owner");
    let (cube, initial, press, direction, history) = prepare_handle(&harness);
    let before_history = harness
        .runtime
        .shell()
        .lock()
        .state
        .transactions()
        .history_status(history)
        .unwrap();
    let mut bridge = SharedViewportPointerBridge::new(UiFrame::new(0.0, 0.0, 1280.0, 720.0));
    dispatch(
        &harness,
        &mut bridge,
        UiPointerEventKind::Down,
        point(press),
        Some(UiPointerButton::Primary),
    );
    let outside = press + direction * 2000.0;
    dispatch(
        &harness,
        &mut bridge,
        UiPointerEventKind::Move,
        point(outside),
        None,
    );
    assert_ne!(transform(&harness, cube), initial);
    bridge.cancel_interaction(&harness.runtime).unwrap();
    assert_eq!(transform(&harness, cube), initial);
    {
        let shell = harness.runtime.shell().lock();
        assert!(!shell.state.has_active_gizmo_interaction());
        assert_eq!(
            shell.state.transactions().history_status(history).unwrap(),
            before_history
        );
    }
    assert_no_outside_delivery(&harness, &mut bridge, point(outside + direction * 64.0));
    // A new button may own capture immediately after cancellation.
    dispatch(
        &harness,
        &mut bridge,
        UiPointerEventKind::Down,
        point(press),
        Some(UiPointerButton::Secondary),
    );
    let before = camera(&harness);
    dispatch(
        &harness,
        &mut bridge,
        UiPointerEventKind::Move,
        point(outside),
        None,
    );
    assert_ne!(camera(&harness), before);
    dispatch(
        &harness,
        &mut bridge,
        UiPointerEventKind::Up,
        point(outside),
        Some(UiPointerButton::Secondary),
    );
    assert_no_outside_delivery(&harness, &mut bridge, point(outside + direction * 128.0));
}

fn prepare_handle(harness: &EventRuntimeHarness) -> (u64, Transform, Vec2, Vec2, HistoryContextId) {
    let mut shell = harness.runtime.shell().lock();
    let state = &mut shell.state;
    let cube = state.world.expect_with_world(|scene| {
        scene
            .nodes()
            .iter()
            .find(|node| matches!(node.kind, NodeKind::Cube))
            .unwrap()
            .id
    });
    state.apply_intent(EditorIntent::SelectNode(cube)).unwrap();
    state
        .apply_viewport_command(&ViewportCommand::ActivateSceneMode(
            SceneModeActivation::Transform(TransformHandleKind::Move),
        ))
        .unwrap();
    let (press, moved) = crate::tests::editing::move_handle_drag_cursor_pair(state, cube);
    let initial = state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().transform);
    let history = HistoryContextId::Document(state.active_scene_document.unwrap());
    (cube, initial, press, (moved - press).normalize(), history)
}

fn transform(harness: &EventRuntimeHarness, cube: u64) -> Transform {
    harness
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().transform)
}

fn camera(harness: &EventRuntimeHarness) -> ViewportCameraSnapshot {
    let shell = harness.runtime.shell().lock();
    shell
        .state
        .world
        .expect_with_world(|scene| shell.state.viewport_controller.current_camera(scene))
}

fn dispatch(
    harness: &EventRuntimeHarness,
    bridge: &mut SharedViewportPointerBridge,
    kind: UiPointerEventKind,
    point: UiPoint,
    button: Option<UiPointerButton>,
) {
    let mut event = UiPointerEvent::new(kind, point);
    if let Some(button) = button {
        event = event.with_button(button);
    }
    dispatch_viewport_pointer_event(&harness.runtime, bridge, event, Default::default()).unwrap();
}

fn assert_no_outside_delivery(
    harness: &EventRuntimeHarness,
    bridge: &mut SharedViewportPointerBridge,
    outside: UiPoint,
) {
    let records = harness.runtime.journal().records().len();
    let before = camera(harness);
    dispatch(harness, bridge, UiPointerEventKind::Move, outside, None);
    assert_eq!(harness.runtime.journal().records().len(), records);
    assert_eq!(camera(harness), before);
}

fn point(position: Vec2) -> UiPoint {
    UiPoint::new(position.x, position.y)
}
