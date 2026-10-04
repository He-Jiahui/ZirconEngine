use super::support::*;
use crate::core::editing::engine::{HistoryContextId, HistoryStatus};
use crate::core::editing::intent::EditorIntent;
use crate::scene::modes::SceneModeActivation;
use crate::scene::viewport::{
    HandleElementExtract, OverlayAxis, ProjectionMode, TransformHandleKind, ViewportCameraSnapshot,
};
use crate::ui::binding::ViewportCommand;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime_interface::math::{Transform, Vec2, Vec3};
use zircon_runtime_interface::ui::{
    dispatch::UiPointerEvent,
    layout::{UiFrame, UiPoint},
    surface::{UiPointerButton, UiPointerEventKind},
};

#[test]
fn native_escape_cancels_surface_capture_and_restores_gizmo_world_and_history() {
    assert_native_gizmo_cancel(false);
}

#[test]
fn native_focus_loss_cancels_surface_capture_and_restores_gizmo_world_and_history() {
    assert_native_gizmo_cancel(true);
}

fn assert_native_gizmo_cancel(focus_lost: bool) {
    let _guard = lock_env();
    let harness = ChildWindowHostHarness::new("zircon_native_viewport_cancel_owner");
    harness.activate_workbench_page();
    let (cube, initial, press, outside, history, history_before) = prepare_handle(&harness);
    dispatch_pointer(
        &harness,
        UiPointerEventKind::Down,
        press,
        Some(UiPointerButton::Primary),
    );
    dispatch_pointer(&harness, UiPointerEventKind::Move, outside, None);
    assert_ne!(world_transform(&harness, cube), initial);
    assert!(harness
        .host
        .borrow()
        .runtime
        .shell()
        .lock()
        .state
        .has_active_gizmo_interaction());

    let before_cancel = harness.journal_len();
    if focus_lost {
        host_context(&harness.root_ui).invoke_native_window_focus_lost();
    } else {
        harness.root_ui.dispatch_native_key_for_test(
            key_event(
                Key::Named(NamedKey::Escape),
                PhysicalKey::Code(KeyCode::Escape),
                None,
                ElementState::Pressed,
            ),
            ModifiersState::empty(),
        );
    }
    assert_eq!(
        harness.delta_events_since(before_cancel),
        vec![EditorEvent::Viewport(
            EditorViewportEvent::CancelInteraction
        )]
    );
    assert_eq!(world_transform(&harness, cube), initial);
    {
        let host = harness.host.borrow();
        let shell = host.runtime.shell().lock();
        assert!(!shell.state.has_active_gizmo_interaction());
        assert_eq!(
            shell.state.transactions().history_status(history).unwrap(),
            history_before
        );
    }
    assert_outside_move_is_idle(&harness, outside + Vec2::splat(64.0));

    // Cancellation must also remove the old trigger button before the next navigation gesture.
    dispatch_pointer(
        &harness,
        UiPointerEventKind::Down,
        press,
        Some(UiPointerButton::Secondary),
    );
    let camera_before = camera(&harness);
    dispatch_pointer(&harness, UiPointerEventKind::Move, outside, None);
    assert_ne!(camera(&harness), camera_before);
    dispatch_pointer(
        &harness,
        UiPointerEventKind::Up,
        outside,
        Some(UiPointerButton::Secondary),
    );
    assert_outside_move_is_idle(&harness, outside + Vec2::splat(128.0));
}

fn prepare_handle(
    harness: &ChildWindowHostHarness,
) -> (u64, Transform, Vec2, Vec2, HistoryContextId, HistoryStatus) {
    let mut host = harness.host.borrow_mut();
    let (cube, initial, press, outside, history, history_before, viewport) = {
        let mut shell = host.runtime.shell().lock();
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
        let viewport = state.viewport_state().size;
        let packet = state.render_snapshot().unwrap();
        let (start, end) = packet
            .overlays
            .handles
            .iter()
            .find(|handle| handle.owner == cube)
            .unwrap()
            .elements
            .iter()
            .find_map(|element| match element {
                HandleElementExtract::AxisLine {
                    axis: OverlayAxis::X,
                    start,
                    end,
                    ..
                } => Some((*start, *end)),
                _ => None,
            })
            .unwrap();
        let start = project_point(&packet.scene.camera, viewport, start);
        let direction = (project_point(&packet.scene.camera, viewport, end) - start).normalize();
        let press = start + direction * 24.0;
        let outside = press + direction * (viewport.x.max(viewport.y) as f32 * 3.0);
        assert!(
            outside.x < 0.0
                || outside.x > viewport.x as f32
                || outside.y < 0.0
                || outside.y > viewport.y as f32
        );
        let initial = state
            .world
            .expect_with_world(|scene| scene.find_node(cube).unwrap().transform);
        let history = HistoryContextId::Document(state.active_scene_document.unwrap());
        let history_before = state.transactions().history_status(history).unwrap();
        (
            cube,
            initial,
            press,
            outside,
            history,
            history_before,
            viewport,
        )
    };
    host.viewport_pointer_bridge
        .update_viewport_frame(UiFrame::new(0.0, 0.0, viewport.x as f32, viewport.y as f32));
    (cube, initial, press, outside, history, history_before)
}

fn project_point(camera: &ViewportCameraSnapshot, viewport: UVec2, world: Vec3) -> Vec2 {
    assert_eq!(camera.projection_mode, ProjectionMode::Perspective);
    let projection = zircon_runtime_interface::math::perspective(
        camera.fov_y_radians,
        viewport.x as f32 / viewport.y as f32,
        camera.z_near,
        camera.z_far,
    );
    let clip = projection
        * zircon_runtime_interface::math::view_matrix(camera.transform)
        * world.extend(1.0);
    assert!(clip.w > 0.0);
    let ndc = clip.truncate() / clip.w;
    Vec2::new(
        (ndc.x * 0.5 + 0.5) * viewport.x as f32,
        (-ndc.y * 0.5 + 0.5) * viewport.y as f32,
    )
}

fn dispatch_pointer(
    harness: &ChildWindowHostHarness,
    kind: UiPointerEventKind,
    position: Vec2,
    button: Option<UiPointerButton>,
) {
    let mut event = UiPointerEvent::new(kind, UiPoint::new(position.x, position.y));
    if let Some(button) = button {
        event = event.with_button(button);
    }
    let mut host = harness.host.borrow_mut();
    let host = &mut *host;
    callback_dispatch::dispatch_viewport_pointer_event(
        &host.runtime,
        &mut host.viewport_pointer_bridge,
        event,
        Default::default(),
    )
    .unwrap();
}

fn world_transform(harness: &ChildWindowHostHarness, cube: u64) -> Transform {
    harness
        .host
        .borrow()
        .runtime
        .shell()
        .lock()
        .state
        .world
        .expect_with_world(|scene| scene.find_node(cube).unwrap().transform)
}

fn camera(harness: &ChildWindowHostHarness) -> ViewportCameraSnapshot {
    let host = harness.host.borrow();
    let shell = host.runtime.shell().lock();
    shell
        .state
        .world
        .expect_with_world(|scene| shell.state.viewport_controller.current_camera(scene))
}

fn assert_outside_move_is_idle(harness: &ChildWindowHostHarness, outside: Vec2) {
    let before = harness.journal_len();
    let camera_before = camera(harness);
    dispatch_pointer(harness, UiPointerEventKind::Move, outside, None);
    assert_eq!(harness.journal_len(), before);
    assert_eq!(camera(harness), camera_before);
}
