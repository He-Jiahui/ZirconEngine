use zircon_runtime::core::framework::physics::PhysicsBodyType;
use zircon_runtime::core::math::{Real, Vec3};

use crate::constraint::JointParams;

use super::runtime::BodyRecord;
use crate::backend::{BodyHandle, ConstraintDesc};

pub(super) fn solve_constraint(
    desc: &ConstraintDesc,
    body_a: &mut BodyRecord,
    body_b: Option<&mut BodyRecord>,
    step_seconds: Real,
) {
    match desc.params {
        JointParams::Fixed => solve_fixed(desc, body_a, body_b, step_seconds),
        JointParams::Distance { min, max, spring } => {
            solve_distance(desc, body_a, body_b, min, max, spring, step_seconds)
        }
        JointParams::Hinge { .. }
        | JointParams::Slider { .. }
        | JointParams::ConeTwist { .. }
        | JointParams::Generic6Dof { .. } => {}
    }
}

fn solve_fixed(
    desc: &ConstraintDesc,
    body_a: &mut BodyRecord,
    body_b: Option<&mut BodyRecord>,
    step_seconds: Real,
) {
    let target = body_b
        .as_deref()
        .map(|body| body.desc.body.transform.translation + desc.anchor_b.translation)
        .unwrap_or(desc.anchor_b.translation);
    project_anchor(desc, body_a, body_b, target, 1.0, step_seconds);
}

fn solve_distance(
    desc: &ConstraintDesc,
    body_a: &mut BodyRecord,
    body_b: Option<&mut BodyRecord>,
    min: Real,
    max: Real,
    spring: Option<crate::constraint::JointSpring>,
    step_seconds: Real,
) {
    let anchor_a = body_a.desc.body.transform.translation + desc.anchor_a.translation;
    let anchor_b = body_b
        .as_deref()
        .map(|body| body.desc.body.transform.translation + desc.anchor_b.translation)
        .unwrap_or(desc.anchor_b.translation);
    let delta = anchor_a - anchor_b;
    let distance = delta.length();
    let target_distance = distance.clamp(min.max(0.0), max.max(min));
    if (distance - target_distance).abs() <= Real::EPSILON {
        return;
    }
    let target = anchor_b + delta.normalize_or_zero() * target_distance;
    let strength = spring
        .map(|spring| (spring.stiffness * step_seconds).clamp(0.0, 1.0))
        .unwrap_or(1.0);
    project_anchor(desc, body_a, body_b, target, strength, step_seconds);
}

fn project_anchor(
    desc: &ConstraintDesc,
    body_a: &mut BodyRecord,
    mut body_b: Option<&mut BodyRecord>,
    target_anchor_a: Vec3,
    strength: Real,
    step_seconds: Real,
) {
    let current = body_a.desc.body.transform.translation + desc.anchor_a.translation;
    let correction = (target_anchor_a - current) * strength;
    let a_dynamic = body_a.desc.body.body_type != PhysicsBodyType::Static;
    let b_dynamic = body_b
        .as_deref()
        .is_some_and(|body| body.desc.body.body_type != PhysicsBodyType::Static);
    let (correction_a, correction_b) = match (a_dynamic, b_dynamic) {
        (true, true) => (correction * 0.5, -correction * 0.5),
        (true, false) => (correction, Vec3::ZERO),
        (false, true) => (Vec3::ZERO, -correction),
        (false, false) => return,
    };
    apply_position_correction(body_a, correction_a, step_seconds);
    if let Some(body_b) = body_b.as_deref_mut() {
        apply_position_correction(body_b, correction_b, step_seconds);
    }
}

fn apply_position_correction(body: &mut BodyRecord, correction: Vec3, step_seconds: Real) {
    body.desc.body.transform.translation += correction;
    body.desc.collider.transform = body.desc.body.transform;
    if step_seconds > Real::EPSILON {
        body.desc.body.linear_velocity = (Vec3::from_array(body.desc.body.linear_velocity)
            + correction / step_seconds)
            .to_array();
    }
    body.active = true;
}

pub(super) fn references_body(desc: &ConstraintDesc, body: BodyHandle) -> bool {
    desc.handles().any(|candidate| candidate == body)
}
