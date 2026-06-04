use zircon_runtime::core::framework::physics::PhysicsColliderSyncState;
use zircon_runtime::core::math::{Real, Vec3};

pub(super) fn trigger_point(
    left: &PhysicsColliderSyncState,
    right: &PhysicsColliderSyncState,
) -> [Real; 3] {
    finite_midpoint(left.transform.translation, right.transform.translation)
        .unwrap_or(Vec3::ZERO)
        .to_array()
}

fn finite_midpoint(left: Vec3, right: Vec3) -> Option<Vec3> {
    let midpoint = Vec3::new(
        midpoint_component(left.x, right.x)?,
        midpoint_component(left.y, right.y)?,
        midpoint_component(left.z, right.z)?,
    );
    (midpoint.x.is_finite() && midpoint.y.is_finite() && midpoint.z.is_finite()).then_some(midpoint)
}

fn midpoint_component(left: Real, right: Real) -> Option<Real> {
    let midpoint = (f64::from(left) + f64::from(right)) * 0.5;
    midpoint.is_finite().then_some(midpoint as Real)
}
