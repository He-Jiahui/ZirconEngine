use zircon_plugin_animation_runtime::{DefaultAnimationManager, LookAtJob, TwoBoneIkJob};
use zircon_runtime::core::framework::animation::{
    AnimationIkCommand, AnimationIkCommandError, AnimationLookAtCommand, AnimationManager,
    AnimationTargetId,
};
use zircon_runtime::core::framework::scene::WorldHandle;
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

#[test]
fn manager_ik_commands_are_validated_and_drained_per_world() {
    let manager = DefaultAnimationManager::default();
    let world = WorldHandle::new(7);
    let other_world = WorldHandle::new(8);
    let bone = AnimationTargetId::from_segments(["Root", "Head"]);
    let command = AnimationIkCommand::LookAt(AnimationLookAtCommand {
        world,
        entity: 41,
        bone,
        target: Vec3::Y,
        axis: Vec3::X,
        clamp_degrees: 35.0,
        weight: 0.75,
    });

    manager.queue_ik_command(3, command.clone()).unwrap();

    assert!(manager.drain_ik_commands(other_world, 3).is_empty());
    assert_eq!(manager.drain_ik_commands(world, 3), vec![command]);
    assert!(manager.drain_ik_commands(world, 3).is_empty());

    let invalid = AnimationIkCommand::LookAt(AnimationLookAtCommand {
        world,
        entity: 41,
        bone,
        target: Vec3::Y,
        axis: Vec3::ZERO,
        clamp_degrees: 35.0,
        weight: 1.0,
    });
    assert_eq!(
        manager.queue_ik_command(3, invalid),
        Err(AnimationIkCommandError::DegenerateAxis { world, entity: 41 })
    );
}
