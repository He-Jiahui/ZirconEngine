use zircon_plugin_animation_runtime::{LookAtJob, TwoBoneIkJob};
use zircon_runtime::core::math::{Quat, Vec3};

#[test]
fn two_bone_ik_reaches_target_within_epsilon() {
    let job = TwoBoneIkJob::new(Vec3::new(1.2, 0.8, 0.0))
        .with_pole(Vec3::Z)
        .with_weight(1.0);
    let solved = job
        .solve_positions(Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0))
        .unwrap();
    assert!((solved.tip - Vec3::new(1.2, 0.8, 0.0)).length() <= 1.0e-5);
    assert!(((solved.mid - solved.root).length() - 1.0).abs() <= 1.0e-5);
    assert!(((solved.tip - solved.mid).length() - 1.0).abs() <= 1.0e-5);
}

#[test]
fn two_bone_ik_clamps_unreachable_target() {
    let solved = TwoBoneIkJob::new(Vec3::new(5.0, 0.0, 0.0))
        .solve_positions(Vec3::ZERO, Vec3::X, Vec3::new(2.0, 0.0, 0.0))
        .unwrap();
    assert!((solved.tip - Vec3::new(2.0, 0.0, 0.0)).length() <= 1.0e-5);
}

#[test]
fn look_at_clamps_to_limit() {
    let job = LookAtJob::new(Vec3::Y, Vec3::X)
        .with_clamp_degrees(30.0)
        .with_weight(1.0);
    let solved = job.solve_rotation(Quat::IDENTITY).unwrap();
    let angle = solved.angle_between(Quat::IDENTITY).to_degrees();
    assert!((angle - 30.0).abs() <= 1.0e-4);
}
