use std::sync::Arc;

use zircon_runtime::scene::Scene;
use zircon_runtime_interface::math::UVec2;

use super::*;
use crate::core::editor_authoring_extension::SceneModeDescriptor;
use crate::core::editor_message::SharedEditorMessageBus;
use crate::core::editor_operation::EditorOperationPath;
use crate::scene::viewport::{DisplayMode, ViewOrientation};
use crate::ui::binding::ViewportCommand;

#[test]
fn viewport_sessions_keep_independent_view_state_across_focus_changes() {
    let left_id = ViewInstanceId::new("editor.scene#left");
    let right_id = ViewInstanceId::new("editor.scene#right");
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        left_id.clone(),
    );
    let mut sessions = SceneViewportSessionRegistry::new(controller, left_id.clone());
    sessions.session(&left_id).resize(UVec2::new(640, 480));
    sessions.session(&left_id).align_view(ViewOrientation::PosX);
    sessions.session(&right_id).resize(UVec2::new(320, 720));
    sessions
        .session(&right_id)
        .align_view(ViewOrientation::NegZ);
    let scene = Scene::new();
    let left_camera = sessions.session(&left_id).current_camera(&scene);
    let right_camera = sessions.session(&right_id).current_camera(&scene);

    assert_eq!(
        sessions.session(&left_id).viewport().size,
        UVec2::new(640, 480)
    );
    assert_eq!(
        sessions.session(&right_id).viewport().size,
        UVec2::new(320, 720)
    );
    assert_ne!(left_camera.transform, right_camera.transform);
    assert_ne!(left_camera.aspect_ratio, right_camera.aspect_ratio);
    assert!(sessions.focus(right_id));
    assert!(sessions.focus(left_id));
    assert_eq!(sessions.viewport().size, UVec2::new(640, 480));
    assert_eq!(sessions.current_camera(&scene), left_camera);
}

#[test]
fn targeted_toolbar_commands_keep_two_leaf_cameras_independent_and_reject_stale_ids() {
    let left_id = ViewInstanceId::new("editor.scene#left");
    let right_id = ViewInstanceId::new("editor.scene#right");
    let stale_id = ViewInstanceId::new("editor.scene#retired");
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        left_id.clone(),
    );
    let mut sessions = SceneViewportSessionRegistry::new(controller, left_id.clone());
    sessions.session(&right_id).resize(UVec2::new(320, 720));
    let scene = Scene::new();
    sessions.session(&left_id).align_view(ViewOrientation::PosX);
    sessions
        .session(&right_id)
        .align_view(ViewOrientation::NegZ);
    let left_before = sessions.session(&left_id).current_camera(&scene);
    let right_before = sessions.session(&right_id).current_camera(&scene);

    sessions
        .apply_command_for_view(
            &right_id,
            Some(&scene),
            &ViewportCommand::SetDisplayMode(DisplayMode::WireOnly),
        )
        .expect("a retained right Scene leaf accepts its toolbar command");

    assert_eq!(
        sessions.session(&left_id).current_camera(&scene),
        left_before
    );
    assert_eq!(
        sessions.session(&right_id).current_camera(&scene),
        right_before
    );
    assert_eq!(
        sessions.session(&right_id).settings().display_mode,
        DisplayMode::WireOnly
    );
    assert_ne!(
        sessions.session(&left_id).settings().display_mode,
        DisplayMode::WireOnly
    );

    let left_after = sessions.session(&left_id).current_camera(&scene);
    let right_after = sessions.session(&right_id).current_camera(&scene);
    let stale = sessions.apply_command_for_view(
        &stale_id,
        Some(&scene),
        &ViewportCommand::SetDisplayMode(DisplayMode::Shaded),
    );
    assert!(matches!(
        stale,
        Err(SceneViewportControllerError::StaleView { view_id }) if view_id == stale_id
    ));
    assert_eq!(
        sessions.session(&left_id).current_camera(&scene),
        left_after
    );
    assert_eq!(
        sessions.session(&right_id).current_camera(&scene),
        right_after
    );
}

#[test]
fn viewport_sessions_share_installed_scene_mode_registries() {
    let left_id = ViewInstanceId::new("editor.scene#left");
    let right_id = ViewInstanceId::new("editor.scene#right");
    let late_id = ViewInstanceId::new("editor.scene#late");
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        left_id.clone(),
    );
    let mut sessions = SceneViewportSessionRegistry::new(controller, left_id.clone());
    sessions.session(&right_id);
    let initial_len = sessions.state.scene_mode_registry.len();
    let descriptor = SceneModeDescriptor::new(
        "test.scene.shared",
        "Shared",
        "editor.scene",
        EditorOperationPath::parse("test.scene.shared.activate").unwrap(),
    );
    let registry = sessions
        .prepare_scene_modes(
            [crate::tests::support::pass_through_scene_mode_registration(
                descriptor,
            )],
        )
        .unwrap();

    sessions.install_prepared_scene_modes(registry);
    sessions.session(&late_id);

    for view_id in [&left_id, &right_id, &late_id] {
        assert_eq!(
            sessions.session(view_id).state.scene_mode_registry.len(),
            initial_len + 1
        );
    }
}

#[test]
fn viewport_sessions_submit_highlights_with_distinct_runtime_viewport_owners() {
    let left_id = ViewInstanceId::new("editor.scene#left");
    let right_id = ViewInstanceId::new("editor.scene#right");
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        left_id.clone(),
    );
    let mut sessions = SceneViewportSessionRegistry::new(controller, left_id.clone());
    sessions
        .session(&left_id)
        .set_runtime_viewport(zircon_runtime_interface::ZrRuntimeViewportHandle::new(11));
    sessions
        .session(&right_id)
        .set_runtime_viewport(zircon_runtime_interface::ZrRuntimeViewportHandle::new(22));

    assert_eq!(
        sessions
            .session(&left_id)
            .build_runtime_highlight_set()
            .viewport(),
        zircon_runtime_interface::ZrRuntimeViewportHandle::new(11)
    );
    assert_eq!(
        sessions
            .session(&right_id)
            .build_runtime_highlight_set()
            .viewport(),
        zircon_runtime_interface::ZrRuntimeViewportHandle::new(22)
    );
}

#[test]
fn empty_scene_view_retention_releases_leaf_leases_and_discards_leaf_camera_state() {
    let left_id = ViewInstanceId::new("editor.scene#left");
    let right_id = ViewInstanceId::new("editor.scene#right");
    let next_id = ViewInstanceId::new("editor.scene#next");
    let scheduler = ToolSchedulerService::new(SharedEditorMessageBus::default());
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        scheduler.clone(),
        left_id.clone(),
    );
    let mut sessions = SceneViewportSessionRegistry::new(controller, left_id.clone());
    let mode_id = crate::core::editor_message::SceneModeId::new("test.scene.retirement");
    let descriptor = SceneModeDescriptor::new(
        "test.scene.retirement",
        "Retirement",
        "editor.scene",
        EditorOperationPath::parse("test.scene.retirement.activate").unwrap(),
    );
    let registry = sessions
        .prepare_scene_modes(
            [crate::tests::support::pass_through_scene_mode_registration(
                descriptor,
            )],
        )
        .unwrap();
    sessions.install_prepared_scene_modes(registry);
    sessions
        .session(&left_id)
        .push_scene_mode_overlay(&mode_id)
        .unwrap();
    sessions
        .session(&right_id)
        .push_scene_mode_overlay(&mode_id)
        .unwrap();
    assert_eq!(scheduler.snapshot().state().active_leases().len(), 2);

    let scene = Scene::new();
    sessions.session(&left_id).align_view(ViewOrientation::PosX);
    let retired_camera = sessions.session(&left_id).current_camera(&scene);

    sessions.retain(&std::collections::BTreeSet::new());

    assert!(sessions.sessions.is_empty());
    assert!(sessions.active.is_none());
    assert!(scheduler.snapshot().state().active_leases().is_empty());

    assert!(sessions.focus(next_id.clone()));
    assert_eq!(sessions.active.as_ref(), Some(&next_id));
    let next_camera = sessions.session(&next_id).current_camera(&scene);
    assert_ne!(retired_camera.transform, next_camera.transform);
}

#[test]
fn viewport_workspace_session_roundtrip_preserves_leaf_camera_and_settings_without_transients() {
    let left_id = ViewInstanceId::new("editor.scene#left");
    let right_id = ViewInstanceId::new("editor.scene#right");
    let live = [left_id.clone(), right_id.clone()]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        left_id.clone(),
    );
    let mut source = SceneViewportSessionRegistry::new(controller, left_id.clone());
    {
        let left = source.session(&left_id);
        left.state.settings.projection_mode =
            zircon_runtime::core::framework::render::ProjectionMode::Orthographic;
        left.state.settings.grid_mode = crate::scene::viewport::GridMode::Hidden;
        left.state.pivot_mode = PivotMode::Primary;
        left.state.orbit_target = zircon_runtime_interface::math::Vec3::new(1.0, 2.0, 3.0);
        let mut camera = zircon_runtime::core::framework::render::ViewportCameraSnapshot::default();
        camera.transform.translation = zircon_runtime_interface::math::Vec3::new(4.0, 5.0, 6.0);
        camera.aspect_ratio = 1.25;
        left.state.camera = Some(camera);
    }
    {
        let right = source.session(&right_id);
        right.state.settings.projection_mode =
            zircon_runtime::core::framework::render::ProjectionMode::Perspective;
        right.state.settings.grid_mode = crate::scene::viewport::GridMode::VisibleAndSnap;
        right.state.orbit_target = zircon_runtime_interface::math::Vec3::new(-3.0, 2.0, 1.0);
        let mut camera = zircon_runtime::core::framework::render::ViewportCameraSnapshot::default();
        camera.transform.translation = zircon_runtime_interface::math::Vec3::new(-6.0, 5.0, 4.0);
        camera.aspect_ratio = 0.75;
        right.state.camera = Some(camera);
    }

    let snapshots = source.snapshot_workspace_sessions(&live);
    let encoded = serde_json::to_string(&snapshots).unwrap();
    assert_eq!(snapshots.len(), 2);
    assert!(encoded.contains("Orthographic"));
    assert!(!encoded.contains("temporal_jitter"));
    assert!(!encoded.contains("aspect_ratio"));
    assert!(!encoded.contains("runtime_viewport"));
    assert!(!encoded.contains("drag"));

    let restored_controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(1, 1),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        left_id.clone(),
    );
    let mut restored = SceneViewportSessionRegistry::new(restored_controller, left_id.clone());
    restored.restore_workspace_sessions(&live, &snapshots, Some(&right_id));

    assert_eq!(restored.active.as_ref(), Some(&right_id));
    assert_eq!(restored.session_count_for_test(), 2);
    assert_eq!(
        restored.sessions[&left_id].state.settings.projection_mode,
        zircon_runtime::core::framework::render::ProjectionMode::Orthographic
    );
    assert_eq!(
        restored.sessions[&left_id].state.settings.grid_mode,
        crate::scene::viewport::GridMode::Hidden
    );
    assert_eq!(
        restored.sessions[&left_id].state.orbit_target,
        zircon_runtime_interface::math::Vec3::new(1.0, 2.0, 3.0)
    );
    assert_eq!(
        restored.sessions[&left_id]
            .state
            .camera
            .as_ref()
            .unwrap()
            .transform
            .translation,
        zircon_runtime_interface::math::Vec3::new(4.0, 5.0, 6.0)
    );
    assert_eq!(
        restored.sessions[&right_id].state.settings.grid_mode,
        crate::scene::viewport::GridMode::VisibleAndSnap
    );
    assert_eq!(
        restored.sessions[&right_id].state.orbit_target,
        zircon_runtime_interface::math::Vec3::new(-3.0, 2.0, 1.0)
    );
    assert_eq!(
        restored.sessions[&right_id]
            .state
            .camera
            .as_ref()
            .unwrap()
            .transform
            .translation,
        zircon_runtime_interface::math::Vec3::new(-6.0, 5.0, 4.0)
    );
    restored.session(&left_id).resize(UVec2::new(800, 600));
    assert_eq!(
        restored.sessions[&left_id]
            .state
            .camera
            .as_ref()
            .unwrap()
            .aspect_ratio,
        4.0 / 3.0,
        "restored camera aspect must be derived from the leaf's live resize"
    );
    assert!(restored
        .sessions
        .values()
        .all(|session| session.state.drag.is_none()));
}

#[test]
fn viewport_workspace_session_restore_ignores_stale_view_and_focus_ids() {
    let live_id = ViewInstanceId::new("editor.scene#live");
    let stale_id = ViewInstanceId::new("editor.scene#stale");
    let live = [live_id.clone()]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        live_id.clone(),
    );
    let mut source = SceneViewportSessionRegistry::new(controller, live_id.clone());
    let mut snapshots = source.snapshot_workspace_sessions(&live);
    let stale_snapshot = snapshots[&live_id].clone();
    snapshots.insert(stale_id.clone(), stale_snapshot);

    let restored_controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        live_id.clone(),
    );
    let mut restored = SceneViewportSessionRegistry::new(restored_controller, live_id.clone());
    restored.restore_workspace_sessions(&live, &snapshots, Some(&stale_id));

    assert_eq!(restored.active.as_ref(), Some(&live_id));
    assert_eq!(restored.session_count_for_test(), 1);
    assert!(!restored.sessions.contains_key(&stale_id));
}

#[test]
fn viewport_workspace_session_snapshot_does_not_fork_missing_live_views() {
    let existing_id = ViewInstanceId::new("editor.scene#existing");
    let missing_id = ViewInstanceId::new("editor.scene#not-created");
    let live = [existing_id.clone(), missing_id.clone()]
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let controller = SceneViewportController::with_settings_and_tools(
        UVec2::new(640, 480),
        Arc::new(SettingsMutationCoordinator::in_memory_with_defaults()),
        ToolSchedulerService::new(SharedEditorMessageBus::default()),
        existing_id.clone(),
    );
    let source = SceneViewportSessionRegistry::new(controller, existing_id.clone());

    let snapshots = source.snapshot_workspace_sessions(&live);

    assert_eq!(snapshots.len(), 2);
    assert_eq!(source.active.as_ref(), Some(&existing_id));
    assert_eq!(source.session_count_for_test(), 1);
    assert!(!source.sessions.contains_key(&missing_id));
}
