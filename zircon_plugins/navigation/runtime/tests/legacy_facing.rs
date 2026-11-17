//! Observe an unparented agent's forward axis through the real Legacy plugin tick.

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

struct LegacyMotion {
    manager: test_support::TestNavigationManager,
    world: World,
    entity: EntityId,
}

impl LegacyMotion {
    fn new(goal: Vec3, initial: Quat, configure: impl FnOnce(&mut NavMeshAgentDescriptor)) -> Self {
        let manager = test_support::navigation_manager();
        assert_eq!(NavigationManager::stats(&*manager).loaded_nav_meshes, 0);
        let mut world = World::new();
        let entity = world.spawn_node(NodeKind::Empty).unwrap();
        assert_eq!(world.parent_of(entity), None);
        world
            .update_transform(
                entity,
                Transform {
                    rotation: initial,
                    ..Transform::default()
                },
            )
            .unwrap();
        world
            .register_component_type(
                navigation_component_descriptors()
                    .into_iter()
                    .find(|descriptor| descriptor.type_id == NAV_MESH_AGENT_COMPONENT_TYPE)
                    .expect("the actual agent component descriptor"),
            )
            .unwrap();
        let mut agent = NavMeshAgentDescriptor {
            destination: Some(goal.to_array()),
            speed: 2.0,
            acceleration: 4.0,
            stopping_distance: 0.0,
            auto_braking: false,
            avoidance_quality: NavAvoidanceQuality::None,
            writeback_mode: NavAgentWritebackMode::Transform,
            ..NavMeshAgentDescriptor::default()
        };
        configure(&mut agent);
        world
            .set_dynamic_component(
                entity,
                NAV_MESH_AGENT_COMPONENT_TYPE,
                serde_json::to_value(agent).unwrap(),
            )
            .unwrap();
        Self {
            manager,
            world,
            entity,
        }
    }

    fn advance(&mut self, expected_position: Vec3, expected_forward: Vec3) {
        let report = self
            .manager
            .tick_world_agents(&mut self.world, 0.5)
            .unwrap();
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
        let world_matrix = self
            .world
            .world_matrix(self.entity)
            .expect("the agent must have a resolved transform");
        let position = world_matrix.transform_point3(Vec3::ZERO);
        assert!(
            position.abs_diff_eq(expected_position, 1.0e-5),
            "world position {position:?}; expected {expected_position:?}"
        );
        // As in the Unreal orientation test, check the actual rotated forward
        // axis, not a quaternion component or an angle in either convention.
        let forward = world_matrix.transform_vector3(Vec3::NEG_Z).normalize();
        eprintln!(
            "LEGACY_FACING position={position:?} forward={forward:?} expected={expected_forward:?}"
        );
        assert!(
            forward.abs_diff_eq(expected_forward, 1.0e-5),
            "world forward {forward:?}; expected {expected_forward:?}"
        );
    }
}

#[test]
fn legacy_turns_to_the_positive_x_movement() {
    LegacyMotion::new(Vec3::X * 4.0, Quat::IDENTITY, |_| {}).advance(Vec3::X, Vec3::X);
}

#[test]
fn legacy_turns_to_the_negative_x_movement() {
    LegacyMotion::new(Vec3::NEG_X * 4.0, Quat::IDENTITY, |_| {}).advance(Vec3::NEG_X, Vec3::NEG_X);
}

#[test]
fn legacy_caps_turn_from_a_nonzero_heading_on_the_correct_side() {
    // Start 30 degrees to the left of -Z; speed 90 deg/sec permits 45 deg
    // per step toward +X. The geometric targets are 15°, 60°, 90° to the right of -Z.
    let left = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_6);
    let mut motion = LegacyMotion::new(Vec3::X * 4.0, left, |agent| agent.angular_speed = 90.0);
    let (sin15, cos15) = 15.0_f32.to_radians().sin_cos();
    motion.advance(Vec3::X, Vec3::new(sin15, 0.0, -cos15));
    motion.advance(Vec3::X * 2.0, Vec3::new(3.0_f32.sqrt() * 0.5, 0.0, -0.5));
    motion.advance(Vec3::X * 3.0, Vec3::X);
}

#[test]
fn legacy_retains_forward_when_movement_is_already_aligned() {
    LegacyMotion::new(Vec3::NEG_Z * 4.0, Quat::IDENTITY, |_| {}).advance(Vec3::NEG_Z, Vec3::NEG_Z);
}

#[test]
fn legacy_retains_authored_facing_when_updates_are_disabled() {
    let initial = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
    LegacyMotion::new(Vec3::NEG_Z * 4.0, initial, |agent| {
        agent.update_rotation = false
    })
    .advance(Vec3::NEG_Z, Vec3::X);
}

#[test]
fn legacy_retains_authored_facing_with_zero_angular_speed() {
    let initial = Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2);
    LegacyMotion::new(Vec3::NEG_Z * 4.0, initial, |agent| {
        agent.angular_speed = 0.0
    })
    .advance(Vec3::NEG_Z, Vec3::X);
}
