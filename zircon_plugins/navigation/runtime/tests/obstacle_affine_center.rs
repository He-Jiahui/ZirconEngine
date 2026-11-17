//! Exercise live carving and fallback steering with the same authored local point.

#[path = "../src/tests/test_support.rs"]
mod test_support;

use zircon_plugin_navigation_recast::{RecastNavigationObstacle, RecastTileCache};
use zircon_plugin_navigation_runtime::{
    navigation_component_descriptors, DefaultNavigationManager,
};
use zircon_runtime::core::framework::navigation::{
    NavAgentTickReport, NavAvoidanceQuality, NavMeshAgentDescriptor, NavMeshAsset,
    NavMeshObstacleDescriptor, NavMeshObstacleShape, NavPathQuery, NavPathStatus,
    NavigationManager, NAV_MESH_AGENT_COMPONENT_TYPE, NAV_MESH_OBSTACLE_COMPONENT_TYPE,
};
use zircon_runtime::core::math::{Quat, Transform, Vec3};
use zircon_runtime::navigation::BuiltinNavigationManager;
use zircon_runtime::scene::components::NodeKind;
use zircon_runtime::scene::{EntityId, World};

#[derive(Clone, Copy)]
enum Anchor {
    Rigid,
    ShearX,
    ShearZ,
}

impl Anchor {
    fn label(self) -> &'static str {
        match self {
            Self::Rigid => "rigid",
            Self::ShearX => "shear_x",
            Self::ShearZ => "shear_z",
        }
    }
}

fn spawn(world: &mut World, transform: Transform) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(entity, transform).unwrap();
    entity
}

fn world_with_anchor(anchor: Anchor) -> (World, EntityId, Vec3, f32) {
    let mut world = World::empty();
    for descriptor in navigation_component_descriptors() {
        world.register_component_type(descriptor).unwrap();
    }
    // Scalar oracle: a 45-degree yaw maps 16X -> (d,0,-d), 16Z -> (d,0,d).
    // Child translation cancels this point before the parent's nonuniform scale.
    let d = 16.0 * std::f32::consts::FRAC_1_SQRT_2;
    let (local, translation, dot) = match anchor {
        Anchor::Rigid => (Vec3::X * 16.0, Vec3::X * -16.0, 0.0),
        Anchor::ShearX => (Vec3::X * 16.0, Vec3::new(-d, 0.0, d), -7.96875),
        Anchor::ShearZ => (Vec3::Z * 16.0, Vec3::new(-d, 0.0, -d), -7.96875),
    };
    let obstacle = if matches!(anchor, Anchor::Rigid) {
        spawn(&mut world, Transform::from_translation(translation))
    } else {
        let parent = spawn(
            &mut world,
            Transform {
                scale: Vec3::new(0.25, 1.0, 4.0),
                ..Transform::default()
            },
        );
        let child = spawn(
            &mut world,
            Transform {
                translation,
                rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
                ..Transform::default()
            },
        );
        assert!(world.set_parent_checked(child, Some(parent)).unwrap());
        child
    };
    let matrix = world.world_matrix(obstacle).unwrap();
    assert!((matrix.x_axis.dot(matrix.z_axis) - dot).abs() < 1.0e-4);
    assert!(matrix.transform_point3(local).length() < 1.0e-4);
    // Report the bad projection as a diagnostic, never use it as the expected center.
    let projected_error = world
        .world_transform(obstacle)
        .unwrap()
        .matrix()
        .transform_point3(local)
        .length();
    (world, obstacle, local, projected_error)
}

fn set_obstacle(world: &mut World, entity: EntityId, descriptor: &NavMeshObstacleDescriptor) {
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_OBSTACLE_COMPONENT_TYPE,
            serde_json::to_value(descriptor).unwrap(),
        )
        .unwrap();
}

fn spawn_agent(world: &mut World, position: Vec3, descriptor: NavMeshAgentDescriptor) -> EntityId {
    let entity = spawn(world, Transform::from_translation(position));
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(descriptor).unwrap(),
        )
        .unwrap();
    entity
}

const WALK_START: Vec3 = Vec3::new(-2.5, 0.0, 0.0);
const WALK_END: Vec3 = Vec3::new(2.5, 0.0, 0.0);

fn plugin_step(
    manager: &DefaultNavigationManager,
    world: &mut World,
    agent: EntityId,
) -> (NavAgentTickReport, f32) {
    world
        .update_transform(agent, Transform::from_translation(WALK_START))
        .unwrap();
    let report = manager.tick_world_agents(world, 0.5).unwrap();
    let x = world.world_matrix(agent).unwrap().w_axis.x;
    (report, x)
}

fn witness_carving(anchor: Anchor) {
    // Adapt the existing native flat-quad control: a forbidden strip separates both ends.
    let asset = NavMeshAsset::simple_quad("humanoid", 3.0);
    let query = NavPathQuery::new(WALK_START.to_array(), WALK_END.to_array());
    let mut native = RecastTileCache::from_asset(&asset).expect("real Detour cache");
    let clear = native.find_path(&query).status;
    let blocker = native
        .add_obstacle(RecastNavigationObstacle::box_obstacle(
            [0.0; 3],
            [0.75, 1.0, 3.5],
        ))
        .expect("native box blocker");
    native.update().unwrap();
    let blocked = native.find_path(&query).status;
    native.remove_obstacle(blocker).unwrap();
    native.update().unwrap();
    let restored = native.find_path(&query).status;
    assert_eq!(clear, NavPathStatus::Complete);
    assert_eq!(blocked, NavPathStatus::NoPath);
    assert_eq!(restored, NavPathStatus::Complete);

    let manager = test_support::navigation_manager();
    let handle = NavigationManager::load_nav_mesh(&*manager, asset).unwrap();
    let (mut world, obstacle, local, projected_error) = world_with_anchor(anchor);
    let agent = spawn_agent(
        &mut world,
        WALK_START,
        NavMeshAgentDescriptor {
            nav_mesh: Some(handle),
            destination: Some(WALK_END.to_array()),
            speed: 2.0,
            avoidance_quality: NavAvoidanceQuality::None,
            update_rotation: false,
            ..NavMeshAgentDescriptor::default()
        },
    );
    let mut desc = NavMeshObstacleDescriptor {
        shape: NavMeshObstacleShape::Box,
        center: local.to_array(),
        size: [1.5, 2.0, 7.0],
        avoidance_enabled: false,
        carve: false,
        ..NavMeshObstacleDescriptor::default()
    };
    set_obstacle(&mut world, obstacle, &desc);
    let (before, x0) = plugin_step(&manager, &mut world, agent);
    desc.carve = true;
    set_obstacle(&mut world, obstacle, &desc);
    let (during, x1) = plugin_step(&manager, &mut world, agent);
    world
        .remove_dynamic_component(obstacle, NAV_MESH_OBSTACLE_COMPONENT_TYPE)
        .unwrap();
    let (after, x2) = plugin_step(&manager, &mut world, agent);
    println!(
        "NAV_OBSTACLE_CARVE label={} projection_gap={} native={clear:?}/{blocked:?}/{restored:?} moved={}/{}/{} blocked={}/{}/{} no_path={}/{}/{} x={}/{}/{}",
        anchor.label(),
        projected_error,
        before.moved_agents,
        during.moved_agents,
        after.moved_agents,
        before.blocked_agents,
        during.blocked_agents,
        after.blocked_agents,
        before.no_path_agents.len(),
        during.no_path_agents.len(),
        after.no_path_agents.len(),
        x0,
        x1,
        x2,
    );
    for (report, x) in [(&before, x0), (&after, x2)] {
        assert_eq!(report.scanned_agents, 1);
        assert_eq!(report.moved_agents, 1);
        assert_eq!(report.blocked_agents, 0);
        assert!(report.no_path_agents.is_empty());
        assert!(x > WALK_START.x + 0.10);
    }
    assert_eq!(during.scanned_agents, 1);
    assert_eq!(
        during.no_path_agents,
        vec![(agent, WALK_END.to_array())],
        "the affine center must carve the actual native path: {during:?}"
    );
    assert_eq!(during.blocked_agents, 1);
    assert_eq!(during.moved_agents, 0);
    assert_eq!(x1, WALK_START.x);
}

fn witness_avoidance(anchor: Anchor) {
    let manager = BuiltinNavigationManager::new();
    let asset = NavMeshAsset::simple_quad("humanoid", 6.0);
    NavigationManager::load_nav_mesh(&manager, asset).unwrap();
    let (mut world, obstacle, local, projected_error) = world_with_anchor(anchor);
    let start = Vec3::new(-0.35, 0.0, 0.08);
    let end = Vec3::X * 4.0;
    let path = NavigationManager::find_path(
        &manager,
        NavPathQuery::new(start.to_array(), end.to_array()),
    )
    .unwrap();
    assert_eq!(path.status, NavPathStatus::Complete);
    let agent = spawn_agent(
        &mut world,
        start,
        NavMeshAgentDescriptor {
            destination: Some(end.to_array()),
            speed: 2.0,
            radius: 0.3,
            stopping_distance: 0.02,
            avoidance_quality: NavAvoidanceQuality::High,
            update_rotation: false,
            ..NavMeshAgentDescriptor::default()
        },
    );
    // Box uses the same local point and does not mix in cylinder base placement.
    let mut desc = NavMeshObstacleDescriptor {
        shape: NavMeshObstacleShape::Box,
        center: local.to_array(),
        size: [0.7, 2.0, 0.7],
        carve: false,
        avoidance_enabled: false,
        ..NavMeshObstacleDescriptor::default()
    };
    let mut step = |avoidance_enabled| {
        world
            .update_transform(agent, Transform::from_translation(start))
            .unwrap();
        desc.avoidance_enabled = avoidance_enabled;
        set_obstacle(&mut world, obstacle, &desc);
        let report = manager.tick_world_agent(&mut world, agent, 0.1).unwrap();
        assert_eq!(report.scanned_agents, 1);
        assert_eq!(report.moved_agents, 1);
        assert_eq!(report.blocked_agents, 0);
        world.world_matrix(agent).unwrap().w_axis.truncate()
    };
    let without = step(false);
    let with = step(true);
    let again = step(false);
    let start_distance = start.length();
    let clear_distance = without.length();
    let avoid_distance = with.length();
    let restored_distance = again.length();
    println!(
        "NAV_OBSTACLE_AVOID label={} projection_gap={} route={:?} distance={}/{}/{}/{} x={}/{}/{}/{}",
        anchor.label(),
        projected_error,
        path.status,
        start_distance,
        clear_distance,
        avoid_distance,
        restored_distance,
        start.x,
        without.x,
        with.x,
        again.x,
    );
    assert!(clear_distance < start_distance - 0.10 && without.x > start.x);
    assert!((again - without).length() < 1.0e-4);
    assert!(
        avoid_distance > start_distance + 0.10 && with.x < start.x,
        "built-in avoidance must steer away from the affine center, not approach it"
    );
}

#[test]
fn rigid_offset_box_carves_and_restores_native_route() {
    witness_carving(Anchor::Rigid);
}

#[test]
fn shear_x_offset_box_carves_and_restores_native_route() {
    witness_carving(Anchor::ShearX);
}

#[test]
fn shear_z_offset_box_carves_and_restores_native_route() {
    witness_carving(Anchor::ShearZ);
}

#[test]
fn rigid_offset_box_steers_builtin_agent_away() {
    witness_avoidance(Anchor::Rigid);
}

#[test]
fn shear_x_offset_box_steers_builtin_agent_away() {
    witness_avoidance(Anchor::ShearX);
}

#[test]
fn shear_z_offset_box_steers_builtin_agent_away() {
    witness_avoidance(Anchor::ShearZ);
}
