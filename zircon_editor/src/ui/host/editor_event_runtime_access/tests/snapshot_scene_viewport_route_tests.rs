use super::core_view_instance_id;
use std::collections::BTreeMap;

use crate::core::editor_event::ViewInstanceId;
use crate::scene::viewport::{
    PivotMode, SceneViewportCameraSnapshot, SceneViewportSettings,
    SceneViewportWorkspaceSessionSnapshot, ViewOrientation, ViewportCameraSnapshot,
};
use crate::tests::editor_event::support::{env_lock, EventRuntimeHarness};
use zircon_runtime_interface::ui::surface::UiPointerEventKind;

#[test]
fn viewport_pointer_routes_accept_live_leaf_ids_and_reject_stale_ids() {
    let _guard = env_lock().lock().expect("viewport test environment lock");
    let harness = EventRuntimeHarness::new("zircon_scene_viewport_route_currentness");
    let view_id = harness
        .runtime
        .view_instance_ids_for_descriptor_key("editor.scene")
        .into_iter()
        .next()
        .expect("the editor test workbench has a live Scene view");
    let (saved_focus, initial_session_count) = {
        let shell = harness.runtime.shell().lock();
        (
            shell.manager.current_focused_view(),
            shell.state.viewport_controller.session_count_for_test(),
        )
    };

    assert!(harness
        .runtime
        .route_scene_viewport_pointer(core_view_instance_id(&view_id), UiPointerEventKind::Move));
    let session_count_after_live_route = harness
        .runtime
        .shell()
        .lock()
        .state
        .viewport_controller
        .session_count_for_test();
    assert!(session_count_after_live_route >= initial_session_count);
    assert_eq!(
        harness
            .runtime
            .shell()
            .lock()
            .manager
            .current_focused_view(),
        saved_focus,
        "hovering a live leaf selects its input session without saving workbench focus"
    );

    assert!(!harness.runtime.route_scene_viewport_pointer(
        ViewInstanceId::new("editor.scene#stale"),
        UiPointerEventKind::Move,
    ));
    let shell = harness.runtime.shell().lock();
    assert_eq!(shell.manager.current_focused_view(), saved_focus);
    assert_eq!(
        shell.state.viewport_controller.session_count_for_test(),
        session_count_after_live_route,
        "a stale callback must not fork or reserve another viewport session"
    );
}

#[test]
fn viewport_workspace_session_access_filters_stale_ids_and_restores_live_focus_state() {
    let _guard = env_lock().lock().expect("viewport test environment lock");
    let harness = EventRuntimeHarness::new("zircon_scene_viewport_workspace_session_restore");
    let live_id = harness
        .runtime
        .view_instance_ids_for_descriptor_key("editor.scene")
        .into_iter()
        .next()
        .expect("the editor test workbench has a live Scene view");
    let live_id = core_view_instance_id(&live_id);
    let stale_id = ViewInstanceId::new("editor.scene#stale");
    let mut camera = ViewportCameraSnapshot::default();
    camera.transform.translation = zircon_runtime_interface::math::Vec3::new(4.0, 5.0, 6.0);
    camera.projection_mode = zircon_runtime::core::framework::render::ProjectionMode::Orthographic;
    let session = SceneViewportWorkspaceSessionSnapshot {
        settings: SceneViewportSettings {
            projection_mode: camera.projection_mode,
            view_orientation: ViewOrientation::PosY,
            ..SceneViewportSettings::default()
        },
        pivot_mode: PivotMode::Primary,
        orbit_target: zircon_runtime_interface::math::Vec3::new(1.0, 2.0, 3.0),
        camera: Some(SceneViewportCameraSnapshot::from(&camera)),
    };
    let saved = BTreeMap::from([
        (live_id.clone(), session.clone()),
        (stale_id.clone(), session),
    ]);

    harness
        .runtime
        .restore_scene_viewport_workspace_sessions(&saved, Some(&stale_id));
    let restored = harness.runtime.scene_viewport_workspace_sessions();

    assert_eq!(restored.len(), 1);
    assert_eq!(restored.get(&live_id), Some(&saved[&live_id]));
    assert!(!restored.contains_key(&stale_id));
    assert_eq!(
        harness.runtime.scene_viewport_settings().projection_mode,
        zircon_runtime::core::framework::render::ProjectionMode::Orthographic
    );
    assert_eq!(
        harness.runtime.scene_viewport_settings().view_orientation,
        ViewOrientation::PosY
    );
}
