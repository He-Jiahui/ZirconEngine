use zircon_runtime::core::framework::navigation::{
    NavAvoidanceQuality, NavLinkMotion, NavMeshAgentDescriptor, NavigationManager,
    OffMeshTraverseEvent, OffMeshTraverseEventKind, AREA_JUMP, NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::navigation::{NavMeshLinkAsset, NavMeshLinkCapacity};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::World;

use crate::navigation_component_descriptors;
use crate::test_support::navigation_manager;
use crate::tests::support::two_island_navmesh;

#[test]
fn jump_link_end_to_end_traverse() {
    let manager = navigation_manager();
    let mut world = agent_world();
    world.register_event::<OffMeshTraverseEvent>();
    let agent = spawn_agent(&mut world, 0.0);
    manager.load_nav_mesh(two_island_navmesh(true)).unwrap();

    let mut events = Vec::new();
    let mut max_height = 0.0_f32;
    for _ in 0..48 {
        let report = manager.tick_world_agents(&mut world, 0.125).unwrap();
        events.extend(report.off_mesh_events);
        max_height = max_height.max(world.world_transform(agent).unwrap().translation.y);
    }

    assert!(events
        .iter()
        .any(|event| event.kind == OffMeshTraverseEventKind::Started));
    assert!(events
        .iter()
        .any(|event| event.kind == OffMeshTraverseEventKind::Completed));
    assert!(
        max_height > 0.5,
        "jump traversal must use its parabolic arc"
    );
    assert!(world.world_transform(agent).unwrap().translation.x > 7.5);
}

#[test]
fn bridge_capacity_queues_agents() {
    let manager = navigation_manager();
    let mut world = agent_world();
    let first = spawn_agent(&mut world, -0.2);
    let second = spawn_agent(&mut world, 0.2);
    manager.load_nav_mesh(capacity_one_bridge_asset()).unwrap();

    let mut observed_queue = false;
    let mut completed = Vec::new();
    for _ in 0..96 {
        let report = manager.tick_world_agents(&mut world, 0.125).unwrap();
        observed_queue |= report.traversing_agents == 1 && report.queued_link_agents == 1;
        completed.extend(
            report
                .off_mesh_events
                .into_iter()
                .filter(|event| event.kind == OffMeshTraverseEventKind::Completed)
                .map(|event| event.agent_entity),
        );
    }

    assert!(
        observed_queue,
        "the second agent must queue behind capacity one"
    );
    assert!(completed.contains(&first));
    assert!(completed.contains(&second));
    assert!(world.world_transform(first).unwrap().translation.x > 7.5);
    assert!(world.world_transform(second).unwrap().translation.x > 7.5);
}

#[test]
fn clearing_queued_agent_destination_releases_bridge_capacity() {
    assert_cancelled_waiter_releases_capacity(|agent| agent.destination = None);
}

#[test]
fn disabling_queued_agent_position_updates_releases_bridge_capacity() {
    assert_cancelled_waiter_releases_capacity(|agent| agent.update_position = false);
}

fn assert_cancelled_waiter_releases_capacity(cancel: impl FnOnce(&mut NavMeshAgentDescriptor)) {
    let manager = navigation_manager();
    let mut world = agent_world();
    let first = spawn_agent(&mut world, -0.2);
    let waiter = spawn_agent(&mut world, 0.2);
    manager.load_nav_mesh(capacity_one_bridge_asset()).unwrap();

    let queued = (0..48).any(|_| {
        let report = manager.tick_world_agents(&mut world, 0.125).unwrap();
        report.traversing_agents == 1 && report.queued_link_agents == 1
    });
    assert!(queued, "the second agent must reach the capacity queue");

    update_agent(&mut world, waiter, cancel);
    let successor = spawn_agent(&mut world, 0.4);
    let mut completed = Vec::new();
    for _ in 0..96 {
        let report = manager.tick_world_agents(&mut world, 0.125).unwrap();
        completed.extend(
            report
                .off_mesh_events
                .into_iter()
                .filter(|event| event.kind == OffMeshTraverseEventKind::Completed)
                .map(|event| event.agent_entity),
        );
    }

    assert!(completed.contains(&first));
    assert!(completed.contains(&successor));
    assert!(!completed.contains(&waiter));
}

fn capacity_one_bridge_asset() -> zircon_runtime::core::framework::navigation::NavMeshAsset {
    let mut asset = two_island_navmesh(false);
    asset.off_mesh_links.push(NavMeshLinkAsset {
        id: 1,
        owner_entity: 42,
        lane_index: 0,
        capacity: NavMeshLinkCapacity::Shared {
            group: 42,
            limit: 1,
        },
        motion: NavLinkMotion::Linear,
        arc_height: 0.0,
        start: [1.0, 0.0, 0.0],
        end: [7.0, 0.0, 0.0],
        width: 0.5,
        bidirectional: true,
        area: AREA_JUMP,
        cost_override: None,
        traversal_mode: Default::default(),
    });
    asset
}

fn update_agent(world: &mut World, entity: u64, update: impl FnOnce(&mut NavMeshAgentDescriptor)) {
    let mut agent = serde_json::from_value::<NavMeshAgentDescriptor>(
        world
            .dynamic_component(entity, NAV_MESH_AGENT_COMPONENT_TYPE)
            .unwrap()
            .clone(),
    )
    .unwrap();
    update(&mut agent);
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(agent).unwrap(),
        )
        .unwrap();
}

fn agent_world() -> World {
    let mut world = World::new();
    world
        .register_component_type(navigation_component_descriptors()[2].clone())
        .unwrap();
    world
}

fn spawn_agent(world: &mut World, z: f32) -> u64 {
    let entity = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(entity, Transform::from_translation(Vec3::new(0.0, 0.0, z)))
        .unwrap();
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some([8.0, 0.0, 0.0]),
                speed: 8.0,
                acceleration: 64.0,
                stopping_distance: 0.05,
                avoidance_quality: NavAvoidanceQuality::None,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    entity
}
