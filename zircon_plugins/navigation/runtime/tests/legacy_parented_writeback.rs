//! The actual Legacy agent tick must keep world-space movement at the scene-write boundary.

#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_runtime::{
    navigation_component_descriptors, DefaultNavigationManager,
};
use zircon_runtime::core::framework::navigation::{
    NavAgentWritebackMode, NavAvoidanceQuality, NavMeshAgentDescriptor, NavigationManager,
    NAV_MESH_AGENT_COMPONENT_TYPE,
};
use zircon_runtime::core::framework::scene::EntityId;
use zircon_runtime::core::math::{Quat, Transform, Vec3};
use zircon_runtime::scene::{components::NodeKind, world::World};

fn spawn(world: &mut World, local: Transform) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(entity, local).unwrap();
    entity
}

fn position(world: &World, entity: EntityId) -> Vec3 {
    world
        .world_matrix(entity)
        .unwrap()
        .transform_point3(Vec3::ZERO)
}

fn step(world: &mut World, agent: EntityId, goal_x: f32) {
    let manager = test_support::navigation_manager();
    assert_eq!(NavigationManager::stats(&*manager).loaded_nav_meshes, 0);
    world
        .register_component_type(navigation_component_descriptors()[2].clone())
        .unwrap();
    world
        .set_dynamic_component(
            agent,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some([goal_x, 0.0, 0.0]),
                speed: 2.0,
                acceleration: 4.0,
                stopping_distance: 0.0,
                auto_braking: false,
                avoidance_quality: NavAvoidanceQuality::None,
                update_position: true,
                update_rotation: false,
                writeback_mode: NavAgentWritebackMode::Transform,
                ..NavMeshAgentDescriptor::default()
            })
            .unwrap(),
        )
        .unwrap();
    let report = manager.tick_world_agents(world, 0.5).unwrap();
    assert_eq!(
        (
            report.scanned_agents,
            report.moved_agents,
            report.blocked_agents
        ),
        (1, 1, 0),
        "{report:?}"
    );
    assert!(report.arrived_agents.is_empty() && report.off_mesh_events.is_empty());
}

#[test]
fn legacy_agent_root_moves_one_world_meter() {
    let mut world = World::new();
    let agent = spawn(
        &mut world,
        Transform::from_translation(Vec3::new(11.0, 0.0, 0.0)),
    );
    step(&mut world, agent, 15.0);
    assert_eq!(world.parent_of(agent), None);
    assert_eq!(position(&world, agent), Vec3::new(12.0, 0.0, 0.0));
    assert_eq!(world.local_transform(agent).unwrap().translation.x, 12.0);
}

#[test]
fn legacy_agent_converts_world_step_through_parent_translation() {
    let mut world = World::new();
    let parent = spawn(
        &mut world,
        Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
    );
    let agent = spawn(&mut world, Transform::from_translation(Vec3::X));
    assert!(world.set_parent_checked(agent, Some(parent)).unwrap());
    assert_eq!(position(&world, agent).x, 11.0);
    let parent_before = world.world_matrix(parent).unwrap();

    step(&mut world, agent, 15.0);

    assert_eq!(world.parent_of(agent), Some(parent));
    assert_eq!(world.world_matrix(parent), Some(parent_before));
    let local_x = world.local_transform(agent).unwrap().translation.x;
    let world_x = position(&world, agent).x;
    eprintln!("LEGACY_TRANSLATED_STEP before_world=11 local_after={local_x} world_after={world_x}");
    assert_eq!((local_x, world_x), (2.0, 12.0));
}

#[test]
fn legacy_agent_keeps_authored_basis_under_a_sheared_parent() {
    let mut world = World::new();
    let grandparent = spawn(
        &mut world,
        Transform {
            scale: Vec3::new(2.0, 1.0, 1.0),
            ..Transform::default()
        },
    );
    let parent = spawn(
        &mut world,
        Transform {
            rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
            ..Transform::default()
        },
    );
    let authored = Transform {
        rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
        scale: Vec3::new(1.25, 0.75, -2.0),
        ..Transform::default()
    };
    let agent = spawn(&mut world, authored);
    assert!(world.set_parent_checked(parent, Some(grandparent)).unwrap());
    assert!(world.set_parent_checked(agent, Some(parent)).unwrap());
    let parent_before = world.world_matrix(parent).unwrap();
    assert!(parent_before.x_axis.dot(parent_before.y_axis).abs() > 1.4);
    let before = world.world_matrix(agent).unwrap();
    assert_eq!(position(&world, agent), Vec3::ZERO);

    step(&mut world, agent, 4.0);

    assert_eq!(world.parent_of(agent), Some(parent));
    assert_eq!(world.world_matrix(parent), Some(parent_before));
    let local = world.local_transform(agent).unwrap();
    let after = world.world_matrix(agent).unwrap();
    let local_delta = 0.5 * std::f32::consts::FRAC_1_SQRT_2;
    let expected_local = Vec3::new(local_delta, -local_delta, 0.0);
    eprintln!(
        "LEGACY_SHEARED_STEP local_after={:?} world_after={:?} basis_same={}",
        local.translation,
        position(&world, agent),
        [after.x_axis, after.y_axis, after.z_axis] == [before.x_axis, before.y_axis, before.z_axis]
    );
    assert!(local.translation.abs_diff_eq(expected_local, 1.0e-5));
    assert!(position(&world, agent).abs_diff_eq(Vec3::X, 1.0e-5));
    assert_eq!(local.rotation, authored.rotation);
    assert_eq!(local.scale, authored.scale);
    assert_eq!(
        [after.x_axis, after.y_axis, after.z_axis],
        [before.x_axis, before.y_axis, before.z_axis]
    );
}
