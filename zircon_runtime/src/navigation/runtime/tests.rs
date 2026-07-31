use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::core::framework::navigation::{
    NavAvoidanceQuality, NavMeshAgentDescriptor, NavMeshAsset, NavMeshObstacleDescriptor,
    NavMeshObstacleShape, NavSampleQuery, NavigationManager, NavigationSettingsAsset,
    DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK, NAV_MESH_AGENT_COMPONENT_TYPE,
    NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use crate::core::framework::scene::ComponentPropertyPath;
use crate::core::math::{Real, Transform, Vec3};
use crate::navigation::NavRepathBudget;
use crate::scene::components::NodeKind;
use crate::scene::World;

use super::math::distance_xz;
use super::world_scan::{
    MAX_NAVIGATION_AVOIDANCE_CANDIDATE_VISITS, MAX_NAVIGATION_AVOIDANCE_CELL_VISITS,
    MAX_NAVIGATION_AVOIDANCE_NEIGHBORS,
};
use super::BuiltinNavigationManager;

#[test]
fn tick_world_agent_moves_only_selected_agent_and_avoids_local_colliders() {
    let manager = BuiltinNavigationManager::new();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 6.0))
        .unwrap();
    let mut world = World::empty();
    let selected = spawn_test_agent(
        &mut world,
        Vec3::new(-0.35, 0.0, 0.08),
        Vec3::new(4.0, 0.0, 0.0),
    );
    let other = spawn_test_agent(
        &mut world,
        Vec3::new(-0.42, 0.0, 0.16),
        Vec3::new(4.0, 0.0, 0.0),
    );
    let obstacle = spawn_test_obstacle(&mut world, Vec3::ZERO, 0.35);
    let selected_before = world.world_transform(selected).unwrap().translation;
    let other_before = world.world_transform(other).unwrap().translation;
    let obstacle_position = world.world_transform(obstacle).unwrap().translation;
    let obstacle_distance_before = distance_xz(selected_before, obstacle_position);
    let other_distance_before = distance_xz(selected_before, other_before);

    let report = manager.tick_world_agent(&mut world, selected, 0.1).unwrap();

    let selected_after = world.world_transform(selected).unwrap().translation;
    let other_after = world.world_transform(other).unwrap().translation;
    assert_eq!(report.scanned_agents, 1);
    assert_eq!(report.moved_agents, 1);
    assert_eq!(
        other_after, other_before,
        "targeted navigation ticks must not advance every agent in the scene"
    );
    assert!(
        distance_xz(selected_after, obstacle_position) > obstacle_distance_before,
        "selected agent should steer away from local obstacle instead of moving deeper into it: before={selected_before:?} after={selected_after:?}"
    );
    assert!(
        distance_xz(selected_after, other_before) > other_distance_before,
        "selected agent should also separate from nearby agents: before={selected_before:?} after={selected_after:?} other={other_before:?}"
    );
}

#[test]
fn avoidance_projection_excludes_distant_rows_from_local_candidates() {
    let manager = BuiltinNavigationManager::new();
    let mut world = World::empty();
    let selected = spawn_test_agent(&mut world, Vec3::ZERO, Vec3::new(4.0, 0.0, 0.0));
    let nearby = spawn_test_agent(
        &mut world,
        Vec3::new(0.2, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    );
    for index in 0..100 {
        spawn_test_agent(
            &mut world,
            Vec3::new(100.0 + index as Real, 0.0, 100.0),
            Vec3::new(104.0, 0.0, 100.0),
        );
    }
    spawn_test_obstacle(&mut world, Vec3::new(100.0, 0.0, 100.0), 0.5);

    let mut projection = manager.lock_state().take_navigation_projection(&world);
    projection.begin_avoidance_frame();
    let (obstacles, agents) = projection.local_avoidance_rows(selected, Vec3::ZERO, 0.3);

    assert!(obstacles.is_empty());
    assert_eq!(agents.len(), 1);
    assert_eq!(agents[0].0, nearby);
}

#[test]
fn avoidance_projection_bounds_cell_visits_for_extreme_radius_obstacles() {
    let manager = BuiltinNavigationManager::new();
    let mut world = World::empty();
    let selected = spawn_test_agent(&mut world, Vec3::ZERO, Vec3::new(4.0, 0.0, 0.0));
    let nearby = spawn_test_obstacle(&mut world, Vec3::new(0.2, 0.0, 0.0), 0.5);
    spawn_test_obstacle(&mut world, Vec3::new(10_000.0, 0.0, 10_000.0), 1_000_000.0);

    let mut projection = manager.lock_state().take_navigation_projection(&world);
    projection.begin_avoidance_frame();
    let nearby_is_retained = {
        let (obstacles, _) = projection.local_avoidance_rows(selected, Vec3::ZERO, 0.3);
        obstacles.iter().any(|obstacle| obstacle.entity == nearby)
    };

    assert!(nearby_is_retained);
    assert!(
        projection.avoidance_cell_visits <= MAX_NAVIGATION_AVOIDANCE_CELL_VISITS,
        "an extreme obstacle radius must not turn a local avoidance query into an unbounded cell scan"
    );
}

#[test]
fn avoidance_projection_updates_the_spatial_index_after_agent_movement() {
    let manager = BuiltinNavigationManager::new();
    let mut world = World::empty();
    let selected = spawn_test_agent(&mut world, Vec3::ZERO, Vec3::new(4.0, 0.0, 0.0));
    let nearby = spawn_test_agent(
        &mut world,
        Vec3::new(0.2, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    );

    let mut projection = manager.lock_state().take_navigation_projection(&world);
    projection.update_agent_position(nearby, Vec3::new(10.0, 0.0, 0.0));
    projection.begin_avoidance_frame();
    let (_, agents) = projection.local_avoidance_rows(selected, Vec3::ZERO, 0.3);

    assert!(agents.is_empty());
}

#[test]
fn avoidance_projection_rotates_a_bounded_dense_neighborhood() {
    let manager = BuiltinNavigationManager::new();
    let mut world = World::empty();
    let selected = spawn_test_agent(&mut world, Vec3::ZERO, Vec3::new(4.0, 0.0, 0.0));
    for _ in 0..=MAX_NAVIGATION_AVOIDANCE_NEIGHBORS {
        spawn_test_agent(
            &mut world,
            Vec3::new(0.2, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
        );
    }

    let mut projection = manager.lock_state().take_navigation_projection(&world);
    projection.begin_avoidance_frame();
    let first = {
        let (_, agents) = projection.local_avoidance_rows(selected, Vec3::ZERO, 0.3);
        assert_eq!(agents.len(), MAX_NAVIGATION_AVOIDANCE_NEIGHBORS);
        agents[0].0
    };
    assert_eq!(
        projection.avoidance_candidate_visits,
        MAX_NAVIGATION_AVOIDANCE_NEIGHBORS
    );
    projection.begin_avoidance_frame();
    let second = {
        let (_, agents) = projection.local_avoidance_rows(selected, Vec3::ZERO, 0.3);
        assert_eq!(agents.len(), MAX_NAVIGATION_AVOIDANCE_NEIGHBORS);
        agents[0].0
    };
    assert_eq!(
        projection.avoidance_candidate_visits,
        MAX_NAVIGATION_AVOIDANCE_NEIGHBORS
    );

    assert_ne!(
        first, second,
        "dense avoidance candidates must rotate fairly"
    );
}

#[test]
fn navigation_projection_counts_typed_rows_and_index_lookups_at_scale() {
    for expected_rows in [1, 100, 10_000] {
        let manager = BuiltinNavigationManager::new();
        let mut world = World::empty();
        for index in 0..expected_rows {
            let position = Vec3::new(index as Real * 4.0, 0.0, 0.0);
            spawn_test_agent(&mut world, position, position + Vec3::X);
            spawn_test_obstacle(&mut world, position + Vec3::Z, 0.3);
            let unrelated = world.spawn_node(NodeKind::Empty);
            world
                .set_dynamic_component(
                    unrelated,
                    "test.NavigationProjectionUnrelated",
                    serde_json::json!({ "index": index }),
                )
                .expect("unrelated dynamic component should attach");
        }

        let mut state = manager.lock_state();
        let mut projection = state.take_navigation_projection(&world);

        assert_eq!(projection.agent_component_rows, expected_rows);
        assert_eq!(projection.obstacle_component_rows, expected_rows);
        assert_eq!(
            state.navigation_projection_component_rows,
            (expected_rows * 2) as u64,
            "the state work counter must report exactly one row visit per typed agent and obstacle"
        );
        let selected = projection
            .agents
            .last()
            .expect("each scale case should include an agent")
            .entity;
        projection.update_agent_position(selected, Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(projection.agent_position_lookups, 1);
        projection.begin_avoidance_frame();
        let _ = projection.local_avoidance_rows(selected, Vec3::new(-1.0, 0.0, 0.0), 0.3);
        assert!(
            projection.avoidance_cell_visits <= MAX_NAVIGATION_AVOIDANCE_CELL_VISITS,
            "the {expected_rows}-row projection must retain a bounded cell query"
        );
        assert!(
            projection.avoidance_candidate_visits <= MAX_NAVIGATION_AVOIDANCE_CANDIDATE_VISITS,
            "the {expected_rows}-row projection must retain a bounded candidate query"
        );

        state.store_navigation_projection(projection);
        let _reused_projection = state.take_navigation_projection(&world);
        assert_eq!(state.navigation_projection_builds, 1);
        assert_eq!(
            state.navigation_projection_component_rows,
            (expected_rows * 2) as u64,
            "a stable world must reuse its typed projection without more component-row visits"
        );
    }
}

#[test]
fn navigation_world_projection_avoids_full_world_and_json_value_scans() {
    let source = include_str!("world_scan.rs");

    assert!(source.contains("dynamic_component_rows"));
    assert!(
        !source.contains("node_records()"),
        "navigation projection must enumerate only navigation component rows"
    );
    assert!(
        !source.contains("serde_json::from_value"),
        "navigation projection must deserialize borrowed component values without cloning JSON"
    );
    assert!(
        !source
            .contains(".iter()\n            .position(|(candidate, _, _)| *candidate == entity)"),
        "single-agent projection updates must use an entity-row index instead of scanning agents"
    );
    assert!(source.contains("NavMeshAgentDescriptor::deserialize(value)"));
    assert!(source.contains("NavMeshObstacleDescriptor::deserialize(value)"));
    assert!(
        !source.contains("NavMeshAgentDescriptor::deserialize(value.as_ref())")
            && !source.contains("NavMeshObstacleDescriptor::deserialize(value.as_ref())"),
        "dynamic component rows already borrow serde_json::Value and must not add an invalid wrapper conversion"
    );
}

#[test]
fn stable_world_ticks_reuse_the_typed_navigation_projection() {
    let manager = BuiltinNavigationManager::new();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 6.0))
        .unwrap();
    let mut world = World::empty();
    spawn_test_agent(
        &mut world,
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    );

    manager.tick_world_agents(&mut world, 0.1).unwrap();
    let (first_builds, first_component_rows) = {
        let state = manager.lock_state();
        (
            state.navigation_projection_builds,
            state.navigation_projection_component_rows,
        )
    };

    manager.tick_world_agents(&mut world, 0.1).unwrap();
    let state = manager.lock_state();
    assert_eq!(
        state.navigation_projection_builds, first_builds,
        "a stable navigation tick must reuse the existing typed projection"
    );
    assert_eq!(
        state.navigation_projection_component_rows, first_component_rows,
        "a stable navigation tick must not enumerate component rows again"
    );
}

#[test]
fn dynamic_agent_property_write_rebuilds_the_typed_navigation_projection() {
    let manager = BuiltinNavigationManager::new();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 6.0))
        .unwrap();
    let mut world = World::empty();
    let agent = spawn_test_agent(
        &mut world,
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    );

    manager.tick_world_agents(&mut world, 0.1).unwrap();
    let (first_builds, first_component_rows) = {
        let state = manager.lock_state();
        (
            state.navigation_projection_builds,
            state.navigation_projection_component_rows,
        )
    };
    let destination = ComponentPropertyPath::new(
        NAV_MESH_AGENT_COMPONENT_TYPE,
        vec!["destination".to_string()],
    )
    .expect("navigation agent destination property path should be valid");
    assert!(world
        .set_dynamic_component_json_property(
            agent,
            &destination,
            serde_json::json!([5.0, 0.0, 0.0]),
        )
        .expect("dynamic navigation agent property write should succeed"));

    manager.tick_world_agents(&mut world, 0.1).unwrap();
    let state = manager.lock_state();
    assert_eq!(
        state.navigation_projection_builds,
        first_builds + 1,
        "a changed navigation dynamic property must invalidate the cached projection"
    );
    assert_eq!(
        state.navigation_projection_component_rows,
        first_component_rows + 1,
        "the invalidated projection must rebuild from the updated typed agent row"
    );
}

#[test]
fn stable_single_agent_ticks_reuse_the_typed_navigation_projection() {
    let manager = BuiltinNavigationManager::new();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 6.0))
        .unwrap();
    let mut world = World::empty();
    let selected = spawn_test_agent(
        &mut world,
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    );
    spawn_test_agent(
        &mut world,
        Vec3::new(-1.0, 0.0, 2.0),
        Vec3::new(4.0, 0.0, 2.0),
    );

    manager.tick_world_agent(&mut world, selected, 0.1).unwrap();
    let (first_builds, first_component_rows) = {
        let state = manager.lock_state();
        (
            state.navigation_projection_builds,
            state.navigation_projection_component_rows,
        )
    };
    assert_eq!(first_builds, 1);
    assert_eq!(first_component_rows, 2);

    manager.tick_world_agent(&mut world, selected, 0.1).unwrap();
    let state = manager.lock_state();
    assert_eq!(state.navigation_projection_builds, first_builds);
    assert_eq!(
        state.navigation_projection_component_rows, first_component_rows,
        "a stable targeted tick must reuse typed agent and obstacle rows"
    );
}

#[test]
fn fallback_queries_borrow_immutable_mesh_snapshots_outside_the_state_lock() {
    let source = include_str!("../runtime.rs");

    assert!(source.contains("selected_mesh_snapshot"));
    assert!(!source
        .contains("let state = self.lock_state();\n            match state.selected_mesh(None)"));
}

#[test]
fn stable_destination_reuses_cached_path_without_another_query() {
    let manager = BuiltinNavigationManager::new();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 6.0))
        .unwrap();
    let mut world = World::empty();
    spawn_test_agent(
        &mut world,
        Vec3::new(-4.0, 0.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
    );

    manager.tick_world_agents(&mut world, 0.1).unwrap();
    let first_queries = manager.lock_state().repath_queries;
    assert_eq!(first_queries, 1);

    manager.tick_world_agents(&mut world, 0.1).unwrap();
    assert_eq!(manager.lock_state().repath_queries, first_queries);
}

#[test]
fn fallback_repath_budget_rotates_path_queries_across_pending_agents() {
    let manager = BuiltinNavigationManager::new();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 10.0))
        .unwrap();
    let mut world = World::empty();
    let first = spawn_test_agent(
        &mut world,
        Vec3::new(-6.0, 0.0, -4.0),
        Vec3::new(6.0, 0.0, -4.0),
    );
    let second = spawn_test_agent(
        &mut world,
        Vec3::new(-6.0, 0.0, 0.0),
        Vec3::new(6.0, 0.0, 0.0),
    );
    let third = spawn_test_agent(
        &mut world,
        Vec3::new(-6.0, 0.0, 4.0),
        Vec3::new(6.0, 0.0, 4.0),
    );
    let before = [
        world.world_transform(first).unwrap().translation,
        world.world_transform(second).unwrap().translation,
        world.world_transform(third).unwrap().translation,
    ];
    manager.lock_state().repath_budget = NavRepathBudget::new(1);

    let first_frame = manager.tick_world_agents(&mut world, 0.1).unwrap();
    assert_eq!(first_frame.moved_agents, 1);
    assert_ne!(world.world_transform(first).unwrap().translation, before[0]);
    assert_eq!(
        world.world_transform(second).unwrap().translation,
        before[1]
    );
    assert_eq!(world.world_transform(third).unwrap().translation, before[2]);

    let second_frame = manager.tick_world_agents(&mut world, 0.1).unwrap();
    assert_eq!(second_frame.moved_agents, 1);
    assert_ne!(
        world.world_transform(second).unwrap().translation,
        before[1]
    );
    assert_eq!(world.world_transform(third).unwrap().translation, before[2]);

    let third_frame = manager.tick_world_agents(&mut world, 0.1).unwrap();
    assert!(third_frame.moved_agents >= 1);
    assert_ne!(world.world_transform(third).unwrap().translation, before[2]);
    assert_eq!(manager.lock_state().repath_queries, 3);
}

#[test]
fn navigation_manager_accessors_recover_poisoned_state_lock() {
    let manager = BuiltinNavigationManager::new();

    let poison = catch_unwind(AssertUnwindSafe(|| {
        let _guard = manager.state.lock().unwrap();
        panic!("poison navigation state lock");
    }));
    assert!(poison.is_err());

    let handle = manager
        .load_nav_mesh(NavMeshAsset::simple_quad(DEFAULT_AGENT_TYPE, 2.0))
        .unwrap();
    manager
        .load_navigation_settings(NavigationSettingsAsset::default_3d())
        .unwrap();

    let hit = manager
        .sample_position(NavSampleQuery {
            nav_mesh: Some(handle),
            position: [0.0, 0.0, 0.0],
            extents: [0.5, 0.5, 0.5],
            agent_type: DEFAULT_AGENT_TYPE.to_string(),
            area_mask: DEFAULT_AREA_MASK,
        })
        .unwrap();

    assert!(
        hit.is_some(),
        "poison recovery should leave loaded navmesh queries usable"
    );
    assert_eq!(manager.stats().loaded_nav_meshes, 1);
}

fn spawn_test_agent(world: &mut World, position: Vec3, destination: Vec3) -> u64 {
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(entity, Transform::from_translation(position))
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                agent_type: DEFAULT_AGENT_TYPE.to_string(),
                radius: 0.3,
                height: 1.7,
                speed: 2.0,
                stopping_distance: 0.02,
                avoidance_quality: NavAvoidanceQuality::High,
                area_mask: DEFAULT_AREA_MASK,
                destination: Some(destination.to_array()),
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    entity
}

fn spawn_test_obstacle(world: &mut World, position: Vec3, radius: Real) -> u64 {
    let entity = world.spawn_node(NodeKind::Empty);
    world
        .update_transform(entity, Transform::from_translation(position))
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            serde_json::to_value(NavMeshObstacleDescriptor {
                shape: NavMeshObstacleShape::Capsule,
                radius,
                avoidance_enabled: true,
                ..NavMeshObstacleDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    entity
}
