use std::collections::HashMap;

use zircon_runtime::core::framework::navigation::NavMeshAsset;
use zircon_runtime::core::framework::navigation::{
    NavAgentWritebackMode, NavDesiredVelocity, NavMeshAgentDescriptor, NavigationDebugCapture,
    NavigationManager, NAV_DESIRED_VELOCITY_COMPONENT_TYPE, NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::World;

use zircon_runtime::navigation::NavRepathBudget;

use crate::navigation_component_descriptors;
use crate::test_support::navigation_manager;

#[test]
fn repath_budget_caps_queries_per_frame() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    world.insert_resource(NavRepathBudget::new(2));
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 10.0))
        .unwrap();
    for index in 0..5 {
        spawn_agent(
            &mut world,
            Vec3::new(-6.0, 0.0, index as f32 - 2.0),
            [6.0, 0.0, index as f32 - 2.0],
            NavAgentWritebackMode::Transform,
        );
    }

    manager.tick_world_agents(&mut world, 0.1).unwrap();

    assert_eq!(world.resource::<NavRepathBudget>().queries_used, 2);
}

#[test]
fn agent_tick_event_payload_contains_typed_editor_debug_state() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let entity = spawn_agent(
        &mut world,
        Vec3::new(-3.0, 0.0, 0.0),
        [3.0, 0.0, 0.0],
        NavAgentWritebackMode::Transform,
    );

    let disabled = manager.tick_world_agents(&mut world, 0.1).unwrap();
    assert!(disabled.debug_agents.is_empty());
    world.insert_resource(NavigationDebugCapture { enabled: true });
    let report = manager.tick_world_agents(&mut world, 0.1).unwrap();
    let debug = report
        .debug_agents
        .iter()
        .find(|debug| debug.entity == entity)
        .expect("runtime tick report publishes typed agent debug state");
    assert_eq!(debug.destination, Some([3.0, 0.0, 0.0]));
    assert!(!debug.path.is_empty());
    assert!(debug.path_status.is_some());
    assert_ne!(debug.avoidance_velocity, [0.0; 3]);

    let round_trip = serde_json::from_value::<
        zircon_runtime::core::framework::navigation::NavAgentTickReport,
    >(serde_json::to_value(&report).unwrap())
    .unwrap();
    assert_eq!(round_trip.debug_agents, report.debug_agents);
}

#[test]
fn agent_tick_report_publishes_arrival_without_enabling_debug_capture() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let destination = [1.0, 0.0, 0.0];
    let entity = spawn_agent(
        &mut world,
        Vec3::from_array(destination),
        destination,
        NavAgentWritebackMode::Transform,
    );

    let report = manager.tick_world_agents(&mut world, 0.1).unwrap();

    assert!(report.debug_agents.is_empty());
    assert_eq!(report.arrived_agents, vec![(entity, destination)]);
    assert!(report.no_path_agents.is_empty());
}

#[test]
fn repath_budget_rotates_across_pending_agents_without_starvation() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    world.insert_resource(NavRepathBudget::new(1));
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 10.0))
        .unwrap();
    let entities = (0..4)
        .map(|index| {
            spawn_agent(
                &mut world,
                Vec3::new(-6.0, 0.0, index as f32 - 1.5),
                [6.0, 0.0, index as f32 - 1.5],
                NavAgentWritebackMode::Transform,
            )
        })
        .collect::<Vec<_>>();

    for _ in 0..8 {
        manager.tick_world_agents(&mut world, 0.1).unwrap();
    }

    for entity in entities {
        assert!(
            world.world_transform(entity).unwrap().translation.x > -6.0,
            "every pending target must eventually receive a repath slot"
        );
    }
}

#[test]
fn desired_velocity_writeback_keeps_transform_owned_by_character_controller() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 6.0))
        .unwrap();
    let entity = spawn_agent(
        &mut world,
        Vec3::new(-3.0, 0.0, 0.0),
        [3.0, 0.0, 0.0],
        NavAgentWritebackMode::DesiredVelocity,
    );

    manager.tick_world_agents(&mut world, 0.1).unwrap();

    assert_eq!(
        world.world_transform(entity).unwrap().translation,
        Vec3::new(-3.0, 0.0, 0.0)
    );
    let desired = serde_json::from_value::<NavDesiredVelocity>(
        world
            .dynamic_component(entity, NAV_DESIRED_VELOCITY_COMPONENT_TYPE)
            .cloned()
            .expect("desired velocity component"),
    )
    .unwrap();
    assert!(desired.linear[0] > 0.0);
}

#[test]
fn desired_velocity_feedback_synchronizes_controller_motion_each_frame() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let entity = spawn_agent(
        &mut world,
        Vec3::new(-4.0, 0.0, 0.0),
        [4.0, 0.0, 0.0],
        NavAgentWritebackMode::DesiredVelocity,
    );

    for _ in 0..12 {
        manager.tick_world_agents(&mut world, 0.1).unwrap();
        let desired = serde_json::from_value::<NavDesiredVelocity>(
            world
                .dynamic_component(entity, NAV_DESIRED_VELOCITY_COMPONENT_TYPE)
                .cloned()
                .unwrap(),
        )
        .unwrap();
        let transform = world.world_transform(entity).unwrap();
        world
            .update_transform(
                entity,
                Transform::from_translation(
                    transform.translation + Vec3::from_array(desired.linear) * 0.1,
                ),
            )
            .unwrap();
    }

    assert!(world.world_transform(entity).unwrap().translation.x > -4.0);
}

#[test]
fn agents_route_to_their_explicit_nav_mesh_crowd() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 2.0))
        .unwrap();
    let large = manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let entity = spawn_agent(
        &mut world,
        Vec3::new(-5.0, 0.0, 0.0),
        [5.0, 0.0, 0.0],
        NavAgentWritebackMode::Transform,
    );
    let mut agent = serde_json::from_value::<NavMeshAgentDescriptor>(
        world
            .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
            .cloned()
            .unwrap(),
    )
    .unwrap();
    agent.nav_mesh = Some(large);
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(agent).unwrap(),
        )
        .unwrap();

    manager.tick_world_agents(&mut world, 0.1).unwrap();

    assert!(world.world_transform(entity).unwrap().translation.x > -5.0);
}

#[test]
fn repath_budget_rotates_across_nav_mesh_crowds_without_starvation() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    world.insert_resource(NavRepathBudget::new(1));
    let first = manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let second = manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let first_agent = spawn_agent_on_mesh(&mut world, first, -4.0, 4.0);
    let second_agent = spawn_agent_on_mesh(&mut world, second, -4.0, 4.0);

    for _ in 0..4 {
        manager.tick_world_agents(&mut world, 0.1).unwrap();
    }

    assert!(world.world_transform(first_agent).unwrap().translation.x > -4.0);
    assert!(world.world_transform(second_agent).unwrap().translation.x > -4.0);
}

#[test]
fn switching_nav_mesh_retires_the_previous_crowd_binding() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    let first = manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let second = manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let entity = spawn_agent_on_mesh(&mut world, first, -4.0, 4.0);
    manager.tick_world_agents(&mut world, 0.2).unwrap();
    set_agent_mesh_and_destination(&mut world, entity, second, [-4.0, 0.0, 0.0]);
    manager.tick_world_agents(&mut world, 0.2).unwrap();
    let before_return = world.world_transform(entity).unwrap().translation.x;
    set_agent_mesh_and_destination(&mut world, entity, first, [-4.0, 0.0, 0.0]);

    manager.tick_world_agents(&mut world, 0.05).unwrap();

    let after_return = world.world_transform(entity).unwrap().translation.x;
    assert!(
        (after_return - before_return).abs() < 0.5,
        "returning to a mesh must not restore the old native agent position"
    );
}

#[test]
fn invalid_agent_does_not_abort_other_crowd_agents() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    let mesh = manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 8.0))
        .unwrap();
    let invalid = spawn_agent_on_mesh(&mut world, mesh, -4.0, 4.0);
    let mut descriptor = agent_descriptor(&world, invalid);
    descriptor.area_mask = 0;
    set_agent_descriptor(&mut world, invalid, descriptor);
    let valid = spawn_agent_on_mesh(&mut world, mesh, -3.0, 3.0);

    let report = manager.tick_world_agents(&mut world, 0.1).unwrap();

    assert!(report.blocked_agents >= 1);
    assert!(world.world_transform(valid).unwrap().translation.x > -3.0);
}

#[test]
fn twenty_agent_corridor_crossing_has_no_deadlock() {
    let manager = navigation_manager();
    let mut world = crowd_world();
    manager
        .load_nav_mesh(NavMeshAsset::simple_quad("humanoid", 12.0))
        .unwrap();
    let mut starts = HashMap::new();
    for index in 0..20 {
        let left_to_right = index % 2 == 0;
        let lane = (index / 2) as f32 * 0.35 - 1.6;
        let start_x = if left_to_right { -8.0 } else { 8.0 };
        let target_x = -start_x;
        let entity = spawn_agent(
            &mut world,
            Vec3::new(start_x, 0.0, lane),
            [target_x, 0.0, lane],
            NavAgentWritebackMode::Transform,
        );
        starts.insert(entity, start_x);
    }

    for _ in 0..120 {
        manager.tick_world_agents(&mut world, 0.05).unwrap();
    }

    for (entity, start_x) in starts {
        let current_x = world.world_transform(entity).unwrap().translation.x;
        assert!(
            (current_x - start_x).abs() > 0.5,
            "agent {entity} made no meaningful corridor progress"
        );
    }
}

fn crowd_world() -> World {
    let mut world = World::new();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    world
}

fn spawn_agent(
    world: &mut World,
    position: Vec3,
    destination: [f32; 3],
    writeback_mode: NavAgentWritebackMode,
) -> u64 {
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(entity, Transform::from_translation(position))
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some(destination),
                radius: 0.16,
                height: 1.8,
                speed: 3.0,
                acceleration: 12.0,
                stopping_distance: 0.1,
                writeback_mode,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    entity
}

fn spawn_agent_on_mesh(
    world: &mut World,
    mesh: zircon_runtime::core::framework::navigation::NavMeshHandle,
    start: f32,
    target: f32,
) -> u64 {
    let entity = spawn_agent(
        world,
        Vec3::new(start, 0.0, 0.0),
        [target, 0.0, 0.0],
        NavAgentWritebackMode::Transform,
    );
    let mut descriptor = agent_descriptor(world, entity);
    descriptor.nav_mesh = Some(mesh);
    set_agent_descriptor(world, entity, descriptor);
    entity
}

fn set_agent_mesh_and_destination(
    world: &mut World,
    entity: u64,
    mesh: zircon_runtime::core::framework::navigation::NavMeshHandle,
    destination: [f32; 3],
) {
    let mut descriptor = agent_descriptor(world, entity);
    descriptor.nav_mesh = Some(mesh);
    descriptor.destination = Some(destination);
    set_agent_descriptor(world, entity, descriptor);
}

fn agent_descriptor(world: &World, entity: u64) -> NavMeshAgentDescriptor {
    serde_json::from_value(
        world
            .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
            .cloned()
            .unwrap(),
    )
    .unwrap()
}

fn set_agent_descriptor(world: &mut World, entity: u64, descriptor: NavMeshAgentDescriptor) {
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(descriptor).unwrap(),
        )
        .unwrap();
}
