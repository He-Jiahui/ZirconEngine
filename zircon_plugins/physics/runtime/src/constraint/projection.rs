use zircon_runtime::core::framework::physics::{PhysicsBodySyncState, PhysicsBodyType};
use zircon_runtime::core::math::{Quat, Real, Vec3};

use super::{ConstraintDesc, JointParams};

pub(crate) struct ProjectedBodies<'a> {
    pub body_a: &'a mut PhysicsBodySyncState,
    pub body_b: Option<&'a mut PhysicsBodySyncState>,
}

pub(crate) fn project_constraint(
    desc: &ConstraintDesc,
    bodies: ProjectedBodies<'_>,
    step_seconds: Real,
) {
    let ProjectedBodies { body_a, body_b } = bodies;
    match &desc.params {
        JointParams::Fixed => {
            let target_rotation = body_b
                .as_deref()
                .map(|body| body.transform.rotation)
                .unwrap_or(desc.anchor_b.rotation);
            project_linear(desc, body_a, body_b, None, 1.0, step_seconds);
            body_a.transform.rotation = target_rotation;
            body_a.angular_velocity = [0.0; 3];
        }
        JointParams::Distance { min, max, spring } => project_linear(
            desc,
            body_a,
            body_b,
            Some((*min, *max)),
            spring
                .map(|spring| (spring.stiffness * step_seconds).clamp(0.0, 1.0))
                .unwrap_or(1.0),
            step_seconds,
        ),
        JointParams::Hinge { axis, limit, motor } => {
            project_hinge_position(desc, body_a, body_b, step_seconds);
            constrain_angular(body_a, None, *axis, *limit, motor.as_ref());
        }
        JointParams::Slider { axis, limit, motor } => project_slider(
            desc,
            body_a,
            body_b,
            *axis,
            *limit,
            motor.as_ref(),
            step_seconds,
        ),
        JointParams::ConeTwist {
            axis,
            swing_limit,
            twist_limit,
            motor,
        } => {
            project_linear(desc, body_a, body_b, None, 1.0, step_seconds);
            let limit = Some([-*twist_limit, *twist_limit]);
            constrain_angular(body_a, None, *axis, limit, motor.as_ref());
            let angular = Vec3::from_array(body_a.angular_velocity);
            let axis = Vec3::from_array(*axis).normalize_or_zero();
            let twist = axis * angular.dot(axis);
            let swing = angular - twist;
            let swing_limit = swing_limit[0].max(swing_limit[1]);
            body_a.angular_velocity = (twist + swing.clamp_length_max(swing_limit)).to_array();
        }
        JointParams::Generic6Dof {
            axis: _,
            linear,
            angular,
        } => {
            let base = body_b
                .as_deref()
                .map(|body| body.transform.translation)
                .unwrap_or(desc.anchor_b.translation);
            let mut relative = body_a.transform.translation - base;
            for axis in 0..3 {
                let [min, max] = linear[axis].limit.unwrap_or([0.0, 0.0]);
                relative[axis] = relative[axis].clamp(min, max);
                if let Some(drive) = linear[axis].drive {
                    let desired = drive.target_position - relative[axis];
                    relative[axis] += desired * (drive.stiffness * step_seconds).clamp(0.0, 1.0);
                    body_a.linear_velocity[axis] = drive.target_velocity;
                }
            }
            apply_position(body_a, base + relative, step_seconds);
            let mut angular_velocity = Vec3::from_array(body_a.angular_velocity);
            for axis in 0..3 {
                if let Some([min, max]) = angular[axis].limit {
                    angular_velocity[axis] = angular_velocity[axis].clamp(min, max);
                }
                if let Some(drive) = angular[axis].drive {
                    angular_velocity[axis] = drive.target_velocity;
                }
            }
            body_a.angular_velocity = angular_velocity.to_array();
        }
    }
}

fn project_hinge_position(
    desc: &ConstraintDesc,
    body_a: &mut PhysicsBodySyncState,
    body_b: Option<&mut PhysicsBodySyncState>,
    step_seconds: Real,
) {
    let pivot = body_b
        .as_deref()
        .map(|body| body.transform.translation + desc.anchor_b.translation)
        .unwrap_or(desc.anchor_b.translation);
    let radius = desc.anchor_a.translation.length();
    let delta = body_a.transform.translation - pivot;
    let direction = if delta.length_squared() > Real::EPSILON {
        delta.normalize()
    } else {
        -Vec3::Y
    };
    apply_position(body_a, pivot + direction * radius, step_seconds);
}

fn project_linear(
    desc: &ConstraintDesc,
    body_a: &mut PhysicsBodySyncState,
    body_b: Option<&mut PhysicsBodySyncState>,
    limits: Option<(Real, Real)>,
    strength: Real,
    step_seconds: Real,
) {
    let anchor_a = body_a.transform.translation + desc.anchor_a.translation;
    let anchor_b = body_b
        .as_deref()
        .map(|body| body.transform.translation + desc.anchor_b.translation)
        .unwrap_or(desc.anchor_b.translation);
    let delta = anchor_a - anchor_b;
    let target_distance = limits
        .map(|(min, max)| delta.length().clamp(min.max(0.0), max.max(min)))
        .unwrap_or(0.0);
    let target_anchor = anchor_b + delta.normalize_or_zero() * target_distance;
    let correction = (target_anchor - anchor_a) * strength;
    apply_pair_correction(body_a, body_b, correction, step_seconds);
}

fn project_slider(
    desc: &ConstraintDesc,
    body_a: &mut PhysicsBodySyncState,
    body_b: Option<&mut PhysicsBodySyncState>,
    axis: [Real; 3],
    limit: Option<[Real; 2]>,
    motor: Option<&zircon_runtime::core::framework::scene::physics::PhysicsJointDrive>,
    step_seconds: Real,
) {
    let origin = body_b
        .as_deref()
        .map(|body| body.transform.translation + desc.anchor_b.translation)
        .unwrap_or(desc.anchor_b.translation);
    let axis = Vec3::from_array(axis).normalize_or_zero();
    let relative = body_a.transform.translation + desc.anchor_a.translation - origin;
    let mut travel = relative.dot(axis);
    if let Some([min, max]) = limit {
        travel = travel.clamp(min, max);
    }
    if let Some(motor) = motor {
        travel += motor.target_velocity * step_seconds;
    }
    apply_position(
        body_a,
        origin + axis * travel - desc.anchor_a.translation,
        step_seconds,
    );
    body_a.transform.rotation = body_b
        .as_deref()
        .map(|body| body.transform.rotation)
        .unwrap_or(Quat::IDENTITY);
}

fn constrain_angular(
    body_a: &mut PhysicsBodySyncState,
    body_b: Option<&PhysicsBodySyncState>,
    axis: [Real; 3],
    limit: Option<[Real; 2]>,
    motor: Option<&zircon_runtime::core::framework::scene::physics::PhysicsJointDrive>,
) {
    let axis = Vec3::from_array(axis).normalize_or_zero();
    let relative = Vec3::from_array(body_a.angular_velocity)
        - body_b
            .map(|body| Vec3::from_array(body.angular_velocity))
            .unwrap_or(Vec3::ZERO);
    let mut speed = relative.dot(axis);
    if let Some([min, max]) = limit {
        speed = speed.clamp(min, max);
    }
    if let Some(motor) = motor {
        speed = motor.target_velocity;
    }
    body_a.angular_velocity = (axis * speed).to_array();
}

fn apply_pair_correction(
    body_a: &mut PhysicsBodySyncState,
    body_b: Option<&mut PhysicsBodySyncState>,
    correction: Vec3,
    step_seconds: Real,
) {
    let a_dynamic = body_a.body_type != PhysicsBodyType::Static;
    let b_dynamic = body_b
        .as_deref()
        .is_some_and(|body| body.body_type != PhysicsBodyType::Static);
    match (a_dynamic, b_dynamic, body_b) {
        (true, true, Some(body_b)) => {
            apply_position(
                body_a,
                body_a.transform.translation + correction * 0.5,
                step_seconds,
            );
            apply_position(
                body_b,
                body_b.transform.translation - correction * 0.5,
                step_seconds,
            );
        }
        (true, _, _) => apply_position(
            body_a,
            body_a.transform.translation + correction,
            step_seconds,
        ),
        (false, true, Some(body_b)) => apply_position(
            body_b,
            body_b.transform.translation - correction,
            step_seconds,
        ),
        _ => {}
    }
}

fn apply_position(body: &mut PhysicsBodySyncState, target: Vec3, step_seconds: Real) {
    let correction = target - body.transform.translation;
    body.transform.translation = target;
    if step_seconds > Real::EPSILON {
        body.linear_velocity =
            (Vec3::from_array(body.linear_velocity) + correction / step_seconds).to_array();
    }
}
