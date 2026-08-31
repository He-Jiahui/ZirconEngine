use zircon_runtime::core::framework::navigation::{
    NavAvoidanceQuality, NavLinkMotion, NavMeshAgentDescriptor, NavPathPoint, NavPathResult,
    NavPathStatus, NavigationManager, OffMeshTraverseEventKind, AREA_JUMP, AREA_WALKABLE,
    NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::navigation::{
    NavMeshAsset, NavMeshLinkAsset, NavMeshLinkCapacity,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::scene::components::{Mobility, NodeKind};
use zircon_runtime::scene::World;

use super::begin_from_path;
use crate::test_support::{navigation_manager, TestNavigationManager};

#[test]
fn started_event_waits_for_successful_transform_write() {
    let (manager, mut world, agent, handle, asset, path) = traversal_fixture();
    begin_from_path(
        &manager,
        agent,
        handle,
        &asset,
        &path,
        Vec3::new(1.0, 0.0, 0.0),
    )
    .unwrap();
    world.set_mobility(agent, Mobility::Static).unwrap();

    let report = manager.tick_world_agents(&mut world, 0.125).unwrap();

    assert_eq!(report.blocked_agents, 1);
    assert!(report.off_mesh_events.is_empty());
}

#[test]
fn completed_event_waits_for_successful_endpoint_write() {
    let (manager, mut world, agent, handle, asset, path) = traversal_fixture();
    begin_from_path(
        &manager,
        agent,
        handle,
        &asset,
        &path,
        Vec3::new(1.0, 0.0, 0.0),
    )
    .unwrap();

    manager.tick_world_agents(&mut world, 0.125).unwrap();
    manager.tick_world_agents(&mut world, 1.0).unwrap();
    world.set_mobility(agent, Mobility::Static).unwrap();
    let report = manager.tick_world_agents(&mut world, 0.125).unwrap();

    assert_eq!(report.blocked_agents, 1);
    assert!(report
        .off_mesh_events
        .iter()
        .all(|event| event.kind != OffMeshTraverseEventKind::Completed));
}

fn traversal_fixture() -> (
    TestNavigationManager,
    World,
    u64,
    zircon_runtime::core::framework::navigation::NavMeshHandle,
    NavMeshAsset,
    NavPathResult,
) {
    let manager = navigation_manager();
    let mut world = World::new();
    world
        .register_component_type(crate::navigation_component_descriptors()[2].clone())
        .unwrap();
    let agent = world.spawn_node(NodeKind::Cube);
    world
        .update_transform(agent, Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)))
        .unwrap();
    world
        .set_dynamic_component(
            agent,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some([8.0, 0.0, 0.0]),
                speed: 8.0,
                acceleration: 64.0,
                avoidance_quality: NavAvoidanceQuality::None,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();

    let mut asset = NavMeshAsset::simple_quad("humanoid", 8.0);
    asset.off_mesh_links.push(link());
    let handle = manager.load_nav_mesh(asset.clone()).unwrap();
    let path = NavPathResult {
        status: NavPathStatus::Complete,
        points: vec![
            NavPathPoint {
                position: [1.0, 0.0, 0.0],
                area: AREA_WALKABLE,
                off_mesh_link_id: None,
                flags: Vec::new(),
            },
            NavPathPoint {
                position: [1.0, 0.0, 0.0],
                area: AREA_JUMP,
                off_mesh_link_id: Some(1),
                flags: vec!["off_mesh_link".to_string()],
            },
        ],
        length: 6.0,
        visited_nodes: 2,
    };
    (manager, world, agent, handle, asset, path)
}

fn link() -> NavMeshLinkAsset {
    NavMeshLinkAsset {
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
    }
}
