use serde_json::json;
use zircon_runtime::asset::{NavMeshAsset, NavigationSettingsAsset};
use zircon_runtime::core::framework::navigation::{
    NavMeshAgentDescriptor, NavPathQuery, NavPathStatus, NavigationAreaSettings, NavigationManager,
    AREA_WALKABLE, NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Real, Transform, Vec3};
use zircon_runtime::core::CoreRuntime;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::world::World;

use crate::tests::support::two_island_navmesh;
use crate::{
    module_descriptor, navigation_component_descriptors, DefaultNavigationManager,
    NAVIGATION_MANAGER_NAME, NAVIGATION_MODULE_NAME,
};

#[test]
fn navigation_module_resolves_manager_and_queries_loaded_navmesh() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(NAVIGATION_MODULE_NAME).unwrap();
    let manager = runtime
        .handle()
        .resolve_manager::<DefaultNavigationManager>(NAVIGATION_MANAGER_NAME)
        .unwrap();

    let handle = manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 5.0))
        .unwrap();
    let mut query = NavPathQuery::new([0.0, 0.0, 0.0], [3.0, 0.0, 4.0]);
    query.nav_mesh = Some(handle);
    let path = manager.find_path(query).unwrap();

    assert_eq!(path.status, NavPathStatus::Complete);
    assert_eq!(path.points.len(), 2);
    assert_eq!(path.length, 5.0);
}

#[test]
fn navigation_manager_ticks_dynamic_agents_toward_destination() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    world
        .register_component_type(navigation_component_descriptors()[2].clone())
        .unwrap();
    let agent = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(agent, Transform::from_translation(Vec3::ZERO))
        .unwrap();

    let component = serde_json::to_value(NavMeshAgentDescriptor {
        destination: Some([3.0, 0.0, 0.0]),
        speed: 2.0,
        ..NavMeshAgentDescriptor::default()
    })
    .unwrap();
    world
        .set_dynamic_component(agent, NAV_MESH_AGENT_COMPONENT_TYPE, component)
        .unwrap();

    let report = manager.tick_world_agents(&mut world, 0.5).unwrap();

    assert_eq!(report.scanned_agents, 1);
    assert_eq!(report.moved_agents, 1);
    assert_eq!(
        world.world_transform(agent).unwrap().translation,
        Vec3::new(1.0, 0.0, 0.0)
    );
}

#[test]
fn navigation_manager_applies_basic_obstacle_avoidance_and_stats() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let agent = world.spawn_node(NodeKind::Cube);
    let obstacle = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(agent, Transform::from_translation(Vec3::ZERO))
        .unwrap();
    world
        .set_dynamic_component(
            agent,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some([4.0, 0.0, 0.0]),
                speed: 2.0,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    world
        .set_dynamic_component(
            obstacle,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            json!({
                "shape": "capsule",
                "center": [1.0, 0.0, 0.2],
                "radius": 1.0,
                "avoidance_enabled": true
            }),
        )
        .unwrap();

    let report = manager.tick_world_agents(&mut world, 0.5).unwrap();
    let transform = world.world_transform(agent).unwrap();
    let stats = manager.stats();

    assert_eq!(report.moved_agents, 1);
    assert!(transform.translation.x > 0.0);
    assert!(transform.translation.z < 0.0);
    assert_eq!(stats.active_agents, 1);
    assert_eq!(stats.active_obstacles, 1);
}

#[test]
fn loaded_navmesh_no_path_blocks_agent_instead_of_direct_fallback() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    world
        .register_component_type(navigation_component_descriptors()[2].clone())
        .unwrap();
    let agent = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(agent, Transform::from_translation(Vec3::ZERO))
        .unwrap();
    world
        .set_dynamic_component(
            agent,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some([8.0, 0.0, 0.0]),
                speed: 2.0,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    manager.load_nav_mesh(two_island_navmesh(false)).unwrap();

    let report = manager.tick_world_agents(&mut world, 0.5).unwrap();

    assert_eq!(report.moved_agents, 0);
    assert_eq!(report.blocked_agents, 1);
    assert_eq!(
        world.world_transform(agent).unwrap().translation,
        Vec3::ZERO
    );
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("no path")));
}

#[test]
fn carved_runtime_obstacle_blocks_agent_path_on_loaded_navmesh() {
    let manager = DefaultNavigationManager::new();
    let mut world = World::new();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    let agent = world.spawn_node(NodeKind::Cube);
    let obstacle = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(
            agent,
            Transform::from_translation(Vec3::new(-2.5, 0.0, 0.0)),
        )
        .unwrap();
    world
        .set_dynamic_component(
            agent,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some([2.5, 0.0, 0.0]),
                speed: 2.0,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    world
        .set_dynamic_component(
            obstacle,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            json!({
                "shape": "box",
                "center": [0.0, 0.0, 0.0],
                // Span the full simple-quad depth so this test proves a block, not a detour.
                "size": [1.5, 2.0, 7.0],
                "carve": true,
                "avoidance_enabled": false
            }),
        )
        .unwrap();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 3.0))
        .unwrap();

    let report = manager.tick_world_agents(&mut world, 0.5).unwrap();

    assert_eq!(report.moved_agents, 0);
    assert_eq!(report.blocked_agents, 1);
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.contains("no path")));
}

#[test]
fn default_navmesh_selection_uses_first_loaded_handle() {
    let manager = DefaultNavigationManager::new();
    let first = manager.load_nav_mesh(two_island_navmesh(false)).unwrap();
    let second = manager.load_nav_mesh(two_island_navmesh(true)).unwrap();
    assert!(first.0 < second.0);

    let result = manager
        .find_path(NavPathQuery::new([0.0, 0.0, 0.0], [8.0, 0.0, 0.0]))
        .unwrap();

    assert_eq!(result.status, NavPathStatus::NoPath);
}

#[test]
fn invalid_navigation_settings_are_rejected() {
    let manager = DefaultNavigationManager::new();
    let mut duplicate_area = NavigationSettingsAsset::default();
    duplicate_area.areas.push(NavigationAreaSettings {
        id: AREA_WALKABLE,
        name: "duplicate".to_string(),
        cost: 1.0,
        walkable: true,
    });

    let error = manager
        .load_navigation_settings(duplicate_area)
        .unwrap_err();

    assert_eq!(
        error.kind,
        zircon_runtime::core::framework::navigation::NavigationErrorKind::InvalidConfiguration
    );

    let mut bad_cost = NavigationSettingsAsset::default();
    bad_cost.areas[0].cost = Real::NAN;
    assert!(manager.load_navigation_settings(bad_cost).is_err());
}
