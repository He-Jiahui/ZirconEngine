//! World movement and local writeback are separate for an attached Legacy agent.

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
use zircon_runtime::core::math::{Mat4, Quat, Transform, Vec3};
use zircon_runtime::scene::{components::NodeKind, world::World};

fn spawn(world: &mut World, transform: Transform, parent: Option<EntityId>) -> EntityId {
    let entity = world.spawn_node(NodeKind::Empty).unwrap();
    world.update_transform(entity, transform).unwrap();
    if let Some(parent) = parent {
        assert!(world.set_parent_checked(entity, Some(parent)).unwrap());
    }
    entity
}

struct Pose {
    local: Transform,
    world: Mat4,
}

fn pose(world: &World, entity: EntityId) -> Pose {
    Pose {
        local: world.local_transform(entity).unwrap(),
        world: world.world_matrix(entity).unwrap(),
    }
}

fn step(world: &mut World, entity: EntityId, angular_speed: f32) -> Pose {
    step_toward(world, entity, angular_speed, Vec3::NEG_Z)
}

fn step_toward(world: &mut World, entity: EntityId, angular_speed: f32, direction: Vec3) -> Pose {
    let manager = test_support::navigation_manager();
    assert_eq!(NavigationManager::stats(&*manager).loaded_nav_meshes, 0);
    world
        .register_component_type(
            navigation_component_descriptors()
                .into_iter()
                .find(|descriptor| descriptor.type_id == NAV_MESH_AGENT_COMPONENT_TYPE)
                .unwrap(),
        )
        .unwrap();
    let before = pose(world, entity);
    let parent = world.parent_of(entity);
    let parent_matrix = parent.map(|parent| world.world_matrix(parent).unwrap());
    world
        .set_dynamic_component(
            entity,
            NAV_MESH_AGENT_COMPONENT_TYPE,
            serde_json::to_value(NavMeshAgentDescriptor {
                destination: Some((before.world.w_axis.truncate() + 4.0 * direction).to_array()),
                speed: 2.0,
                acceleration: 4.0,
                angular_speed,
                stopping_distance: 0.0,
                auto_braking: false,
                avoidance_quality: NavAvoidanceQuality::None,
                update_position: true,
                update_rotation: true,
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
    assert_eq!(world.parent_of(entity), parent);
    assert_eq!(
        parent.map(|parent| world.world_matrix(parent).unwrap()),
        parent_matrix
    );
    let after = pose(world, entity);
    eprintln!(
        "LEGACY_PARENTED_FACING local={:?} world={:?} forward={:?} local_scale={:?} world_xz_dot={}",
        after.local.translation,
        after.world.w_axis.truncate(),
        after.world.transform_vector3(Vec3::NEG_Z).normalize(),
        after.local.scale,
        after.world.x_axis.dot(after.world.z_axis),
    );
    after
}

fn close(label: &str, actual: Vec3, expected: Vec3) {
    assert!(
        actual.abs_diff_eq(expected, 2.0e-5),
        "{label}: {actual:?}; expected {expected:?}"
    );
}

fn assert_upright_pose(
    after: Pose,
    world_position: Vec3,
    local_position: Vec3,
    local_scale: Vec3,
    world_scale: Vec3,
) {
    // Observe the attached world transform (including the rotated forward),
    // then independently check the local values that own the authoring state.
    close(
        "world position",
        after.world.w_axis.truncate(),
        world_position,
    );
    close(
        "world forward",
        after.world.transform_vector3(Vec3::NEG_Z).normalize(),
        Vec3::NEG_Z,
    );
    close(
        "world X basis",
        after.world.x_axis.truncate(),
        Vec3::X * world_scale.x,
    );
    close(
        "world Y basis",
        after.world.y_axis.truncate(),
        Vec3::Y * world_scale.y,
    );
    close(
        "world Z basis",
        after.world.z_axis.truncate(),
        Vec3::Z * world_scale.z,
    );
    close("local position", after.local.translation, local_position);
    assert_eq!(
        after.local.scale, local_scale,
        "authored local scale must not be reprojected"
    );
}

#[test]
fn legacy_facing_root_keeps_the_world_step_and_authored_scale() {
    let mut world = World::new();
    let child = spawn(&mut world, Transform::default(), None);
    let after = step(&mut world, child, 360.0);
    assert_upright_pose(after, Vec3::NEG_Z, Vec3::NEG_Z, Vec3::ONE, Vec3::ONE);
}

#[test]
fn legacy_facing_converts_world_point_under_a_translated_parent() {
    let mut world = World::new();
    let parent = spawn(
        &mut world,
        Transform::from_translation(10.0 * Vec3::X),
        None,
    );
    let child = spawn(&mut world, Transform::default(), Some(parent));
    let after = step(&mut world, child, 360.0);
    assert_upright_pose(
        after,
        10.0 * Vec3::X - Vec3::Z,
        Vec3::NEG_Z,
        Vec3::ONE,
        Vec3::ONE,
    );
}

#[test]
fn legacy_facing_converts_world_turn_under_a_rotated_parent() {
    let mut world = World::new();
    let parent = spawn(
        &mut world,
        Transform {
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ..Transform::default()
        },
        None,
    );
    let child = spawn(&mut world, Transform::default(), Some(parent));
    close(
        "initial child forward",
        world
            .world_matrix(child)
            .unwrap()
            .transform_vector3(Vec3::NEG_Z),
        Vec3::NEG_X,
    );
    // A sufficiently large turn budget reaches world -Z in either convention
    // of the independent yaw-sign bug; this is a test of the scene boundary.
    let after = step(&mut world, child, 360.0);
    close("local forward", after.local.rotation * Vec3::NEG_Z, Vec3::X);
    assert_upright_pose(after, Vec3::NEG_Z, Vec3::X, Vec3::ONE, Vec3::ONE);
}

#[test]
fn legacy_facing_preserves_local_scale_under_a_tilted_uniform_parent() {
    let mut world = World::new();
    let parent = spawn(
        &mut world,
        Transform {
            rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_6),
            scale: Vec3::splat(2.0),
            ..Transform::default()
        },
        None,
    );
    let child_scale = Vec3::new(0.75, 1.25, 1.5);
    let child = spawn(
        &mut world,
        Transform {
            scale: child_scale,
            ..Transform::default()
        },
        Some(parent),
    );
    let after = step(&mut world, child, 360.0);
    let local_position = Vec3::new(0.0, -0.25, -3.0_f32.sqrt() * 0.25);
    assert_upright_pose(
        after,
        Vec3::NEG_Z,
        local_position,
        child_scale,
        2.0 * child_scale,
    );
}

#[test]
fn legacy_facing_composes_world_yaw_after_the_inverse_of_a_noncommuting_parent() {
    let mut world = World::new();
    let parent = spawn(
        &mut world,
        Transform {
            rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_6),
            ..Transform::default()
        },
        None,
    );
    let child = spawn(&mut world, Transform::default(), Some(parent));
    // +Z is a half turn in both yaw conventions, and the world half-turn
    // does not commute with the parent's rotation about a different axis.
    let after = step_toward(&mut world, child, 720.0, Vec3::Z);
    let local_forward = Vec3::new(0.0, 0.5, 3.0_f32.sqrt() * 0.5);
    close(
        "local forward after the half-turn",
        after.local.rotation * Vec3::NEG_Z,
        local_forward,
    );
    close(
        "local point after world movement",
        after.local.translation,
        local_forward,
    );
    close("world position", after.world.w_axis.truncate(), Vec3::Z);
    close(
        "world forward",
        after.world.transform_vector3(Vec3::NEG_Z),
        Vec3::Z,
    );
    close("world right", after.world.x_axis.truncate(), Vec3::NEG_X);
    close("world up", after.world.y_axis.truncate(), Vec3::Y);
    assert_eq!(after.local.scale, Vec3::ONE);
}

#[test]
fn legacy_facing_with_zero_angular_speed_preserves_a_sheared_world_basis() {
    let mut world = World::new();
    let grandparent = spawn(
        &mut world,
        Transform {
            scale: Vec3::new(2.0, 1.0, 4.0),
            ..Transform::default()
        },
        None,
    );
    let parent = spawn(
        &mut world,
        Transform {
            rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_4),
            ..Transform::default()
        },
        Some(grandparent),
    );
    let local = Transform {
        rotation: Quat::from_rotation_x(std::f32::consts::FRAC_PI_6),
        scale: Vec3::new(1.5, 1.25, 0.5),
        ..Transform::default()
    };
    let child = spawn(&mut world, local, Some(parent));
    let before = pose(&world, child);
    assert!(before.world.x_axis.dot(before.world.z_axis).abs() > 1.0);
    let after = step(&mut world, child, 0.0);
    let component = std::f32::consts::FRAC_1_SQRT_2 / 4.0;
    close(
        "world position under shear",
        after.world.w_axis.truncate(),
        Vec3::NEG_Z,
    );
    close(
        "local point under shear",
        after.local.translation,
        Vec3::new(component, 0.0, -component),
    );
    assert_eq!(after.local.rotation, before.local.rotation);
    assert_eq!(after.local.scale, before.local.scale);
    assert_eq!(after.world.x_axis, before.world.x_axis);
    assert_eq!(after.world.y_axis, before.world.y_axis);
    assert_eq!(after.world.z_axis, before.world.z_axis);
}
