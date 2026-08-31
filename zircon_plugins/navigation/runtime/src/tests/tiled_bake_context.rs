use serde_json::json;
use std::sync::{Arc, Barrier};

use zircon_runtime::core::framework::navigation::{
    NavMeshBakeRequest, NavigationManager, NAV_MESH_SURFACE_COMPONENT_TYPE,
};
use zircon_runtime::scene::components::NodeKind;

use super::bake::{tiled_test_world, wait_for_dirty_bake, wait_for_tiled_bake};
use crate::test_support::navigation_manager;
use crate::{NavMeshBakeTaskState, NavMeshDirtyBounds};

#[test]
fn stale_async_bake_cannot_restore_snapshot_after_newer_full_bake() {
    let manager = navigation_manager();
    let stale_task = manager.start_tiled_bake(tiled_test_world(24), NavMeshBakeRequest::default());
    let mut newer_world = tiled_test_world(4);
    let surface = newer_world.node_records()[0].id;
    newer_world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [6.0, 4.0, 4.0]}),
        )
        .unwrap();
    manager
        .bake_surface(&newer_world, NavMeshBakeRequest::default())
        .unwrap();
    assert_eq!(manager.bake_task_state(stale_task), None);

    let error = wait_for_dirty_bake(
        &manager,
        manager.start_dirty_tile_rebuild(
            tiled_test_world(4),
            NavMeshBakeRequest::default(),
            NavMeshDirtyBounds::new([-1.0, -1.0, -1.0], [1.0, 2.0, 1.0]),
        ),
    )
    .unwrap_err();

    assert!(error.message.contains("previously completed tiled bake"));
}

#[test]
fn settings_update_retires_inflight_bake_and_snapshot() {
    let manager = navigation_manager();
    let stale_task = manager.start_tiled_bake(tiled_test_world(24), NavMeshBakeRequest::default());
    let mut settings = manager.active_settings();
    settings.areas[0].cost += 0.25;
    manager.load_navigation_settings(settings).unwrap();

    assert_eq!(manager.bake_task_state(stale_task), None);
    let error = wait_for_dirty_bake(
        &manager,
        manager.start_dirty_tile_rebuild(
            tiled_test_world(4),
            NavMeshBakeRequest::default(),
            NavMeshDirtyBounds::new([-1.0, -1.0, -1.0], [1.0, 2.0, 1.0]),
        ),
    )
    .unwrap_err();

    assert!(error.message.contains("previously completed tiled bake"));
}

#[test]
fn newer_bake_retires_superseded_full_and_dirty_tasks() {
    let manager = navigation_manager();
    manager
        .bake_surface(&tiled_test_world(6), NavMeshBakeRequest::default())
        .unwrap();
    let dirty_task = manager.start_dirty_tile_rebuild(
        tiled_test_world(6),
        NavMeshBakeRequest::default(),
        NavMeshDirtyBounds::new([-1.0, -1.0, -1.0], [1.0, 2.0, 1.0]),
    );
    let full_task = manager.start_tiled_bake(tiled_test_world(12), NavMeshBakeRequest::default());

    assert_eq!(manager.bake_task_state(dirty_task), None);
    assert!(matches!(
        manager.bake_task_state(full_task),
        Some(NavMeshBakeTaskState::Pending | NavMeshBakeTaskState::Ready)
    ));
}

#[test]
fn tiled_snapshots_are_isolated_per_explicit_surface() {
    let manager = navigation_manager();
    let mut world = tiled_test_world(6);
    let surface_a = world.node_records()[0].id;
    let surface_b = world.spawn_node(NodeKind::Empty);
    world
        .set_dynamic_component(
            surface_b,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [8.0, 4.0, 4.0], "override_tile_size": 1}),
        )
        .unwrap();
    let request_a = NavMeshBakeRequest {
        surface_entity: Some(surface_a),
        ..NavMeshBakeRequest::default()
    };
    let request_b = NavMeshBakeRequest {
        surface_entity: Some(surface_b),
        ..NavMeshBakeRequest::default()
    };
    manager.bake_surface(&world, request_a.clone()).unwrap();
    manager.bake_surface(&world, request_b).unwrap();

    let dirty = wait_for_dirty_bake(
        &manager,
        manager.start_dirty_tile_rebuild(
            world,
            request_a,
            NavMeshDirtyBounds::new([-1.0, -1.0, -1.0], [1.0, 2.0, 1.0]),
        ),
    )
    .unwrap();

    assert!(!dirty.rebuilt_tile_ids.is_empty());
}

#[test]
fn default_and_explicit_selection_share_the_same_surface_context() {
    let manager = navigation_manager();
    let mut world = tiled_test_world(4);
    manager
        .bake_surface(&world, NavMeshBakeRequest::default())
        .unwrap();
    let surface = world.node_records()[0].id;
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [6.0, 4.0, 4.0]}),
        )
        .unwrap();
    manager
        .bake_surface(
            &world,
            NavMeshBakeRequest {
                surface_entity: Some(surface),
                ..NavMeshBakeRequest::default()
            },
        )
        .unwrap();
    world
        .set_dynamic_component(
            surface,
            NAV_MESH_SURFACE_COMPONENT_TYPE,
            json!({"volume_size": [6.0, 4.0, 4.0], "override_tile_size": 1}),
        )
        .unwrap();

    let error = wait_for_dirty_bake(
        &manager,
        manager.start_dirty_tile_rebuild(
            world,
            NavMeshBakeRequest::default(),
            NavMeshDirtyBounds::new([-1.0, -1.0, -1.0], [1.0, 2.0, 1.0]),
        ),
    )
    .unwrap_err();

    assert!(error.message.contains("previously completed tiled bake"));
}

#[test]
fn concurrent_same_surface_submissions_atomically_retire_the_older_handle() {
    let manager = navigation_manager();
    let barrier = Arc::new(Barrier::new(3));
    let left_manager = manager.clone();
    let left_barrier = Arc::clone(&barrier);
    let left = std::thread::spawn(move || {
        left_barrier.wait();
        left_manager.start_tiled_bake(tiled_test_world(8), NavMeshBakeRequest::default())
    });
    let right_manager = manager.clone();
    let right_barrier = Arc::clone(&barrier);
    let right = std::thread::spawn(move || {
        right_barrier.wait();
        right_manager.start_tiled_bake(tiled_test_world(8), NavMeshBakeRequest::default())
    });
    barrier.wait();
    let handles = [left.join().unwrap(), right.join().unwrap()];
    let active = handles
        .into_iter()
        .filter(|handle| manager.bake_task_state(*handle).is_some())
        .collect::<Vec<_>>();

    assert_eq!(active.len(), 1);
    wait_for_tiled_bake(&manager, active[0]);
}
