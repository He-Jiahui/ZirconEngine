use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::asset::{NavMeshAsset, NavigationSettingsAsset};
use crate::core::framework::navigation::{
    NavAvoidanceQuality, NavMeshAgentDescriptor, NavMeshObstacleDescriptor, NavMeshObstacleShape,
    NavSampleQuery, NavigationManager, DEFAULT_AGENT_TYPE, DEFAULT_AREA_MASK,
    NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use crate::core::math::{Real, Transform, Vec3};
use crate::scene::components::NodeKind;
use crate::scene::World;

use super::math::distance_xz;
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
