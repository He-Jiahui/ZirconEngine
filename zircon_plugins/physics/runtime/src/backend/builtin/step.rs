use zircon_runtime::core::framework::physics::PhysicsWorldStepPlan;
use zircon_runtime::core::math::{Quat, Real, Vec3};
use zircon_runtime::scene::components::RigidBodyType;
use zircon_runtime::scene::world::World;

use crate::manager::validation::{rigid_body_step_input_is_finite, transform_is_finite};

pub fn integrate_builtin_physics_steps(world: &mut World, plan: PhysicsWorldStepPlan) {
    const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

    if plan.steps == 0 || !plan.step_seconds.is_finite() || plan.step_seconds <= 0.0 {
        return;
    }

    let entities = world
        .node_records()
        .iter()
        .map(|node| node.id)
        .collect::<Vec<_>>();
    for _ in 0..plan.steps {
        for entity in &entities {
            let Some(mut rigid_body) = world.rigid_body(*entity).cloned() else {
                continue;
            };
            if rigid_body.body_type == RigidBodyType::Static {
                continue;
            }
            if !rigid_body_step_input_is_finite(&rigid_body) {
                continue;
            }
            let Some(mut transform) = world.find_node(*entity).map(|node| node.transform) else {
                continue;
            };
            if !transform_is_finite(transform) {
                continue;
            }

            let mut velocity = match rigid_body.body_type {
                RigidBodyType::Dynamic => {
                    let damping = (1.0 - rigid_body.linear_damping.max(0.0) * plan.step_seconds)
                        .clamp(0.0, 1.0);
                    (rigid_body.linear_velocity
                        + GRAVITY * rigid_body.gravity_scale * plan.step_seconds)
                        * damping
                }
                RigidBodyType::Kinematic => rigid_body.linear_velocity,
                RigidBodyType::Static => unreachable!(),
            };
            for axis in 0..3 {
                if rigid_body.lock_translation[axis] {
                    velocity[axis] = 0.0;
                } else {
                    transform.translation[axis] += velocity[axis] * plan.step_seconds;
                }
            }
            rigid_body.linear_velocity = velocity;

            let mut angular_velocity = match rigid_body.body_type {
                RigidBodyType::Dynamic => {
                    rigid_body.angular_velocity
                        * (1.0 - rigid_body.angular_damping.max(0.0) * plan.step_seconds)
                            .clamp(0.0, 1.0)
                }
                RigidBodyType::Kinematic => rigid_body.angular_velocity,
                RigidBodyType::Static => unreachable!(),
            };
            for axis in 0..3 {
                if rigid_body.lock_rotation[axis] {
                    angular_velocity[axis] = 0.0;
                }
            }
            let rotation_step = angular_velocity * plan.step_seconds;
            if rotation_step.length_squared() > Real::EPSILON {
                transform.rotation =
                    (Quat::from_scaled_axis(rotation_step) * transform.rotation).normalize();
            }
            rigid_body.angular_velocity = angular_velocity;

            if !transform_is_finite(transform) || !rigid_body_step_input_is_finite(&rigid_body) {
                continue;
            }

            let _ = world.update_transform(*entity, transform);
            let _ = world.set_rigid_body(*entity, Some(rigid_body));
        }
    }
}

pub(super) fn integrate_body_sync_state(
    body: &mut zircon_runtime::core::framework::physics::PhysicsBodySyncState,
    accumulated_force: Vec3,
    step_seconds: Real,
) -> bool {
    use zircon_runtime::core::framework::physics::PhysicsBodyType;

    if body.body_type == PhysicsBodyType::Static
        || !step_seconds.is_finite()
        || step_seconds <= 0.0
        || !body.mass.is_finite()
        || body.mass <= 0.0
        || !body.transform.translation.is_finite()
        || !body.transform.rotation.is_finite()
        || !body.transform.scale.is_finite()
    {
        return false;
    }

    let mut velocity = Vec3::from_array(body.linear_velocity);
    if body.body_type == PhysicsBodyType::Dynamic {
        let damping = (1.0 - body.linear_damping.max(0.0) * step_seconds).clamp(0.0, 1.0);
        velocity = (velocity
            + (Vec3::new(0.0, -9.81, 0.0) * body.gravity_scale + accumulated_force / body.mass)
                * step_seconds)
            * damping;
    }
    for axis in 0..3 {
        if body.lock_translation[axis] {
            velocity[axis] = 0.0;
        } else {
            body.transform.translation[axis] += velocity[axis] * step_seconds;
        }
    }
    body.linear_velocity = velocity.to_array();

    let mut angular_velocity = Vec3::from_array(body.angular_velocity);
    if body.body_type == PhysicsBodyType::Dynamic {
        angular_velocity *= (1.0 - body.angular_damping.max(0.0) * step_seconds).clamp(0.0, 1.0);
    }
    for axis in 0..3 {
        if body.lock_rotation[axis] {
            angular_velocity[axis] = 0.0;
        }
    }
    let rotation_step = angular_velocity * step_seconds;
    if rotation_step.length_squared() > Real::EPSILON {
        body.transform.rotation =
            (Quat::from_scaled_axis(rotation_step) * body.transform.rotation).normalize();
    }
    body.angular_velocity = angular_velocity.to_array();
    true
}
