use std::time::Duration;

use crate::core::{CoreRuntime, TimePolicy, TimePolicyError, TimePolicyTransaction};

use super::WorldTimeController;

#[test]
fn worlds_keep_pause_scale_and_fixed_debt_independent() {
    let outer = CoreRuntime::new();
    let policy = TimePolicy::default().with_fixed_timestep(Duration::from_millis(10));
    let mut paused = WorldTimeController::new(policy).expect("valid policy");
    let mut scaled = WorldTimeController::new(policy.with_virtual_relative_speed(0.5))
        .expect("valid scaled policy");

    paused.pause_virtual_time();
    let paused_frame = paused
        .advance(outer.advance_time_by(Duration::from_millis(25), 8))
        .expect("first paused frame should be accepted");
    let scaled_frame = scaled
        .advance(outer.advance_time_by(Duration::from_millis(25), 8))
        .expect("first scaled frame should be accepted");

    assert!(paused_frame.virtual_time_paused());
    assert_eq!(paused_frame.virtual_delta(), Duration::ZERO);
    assert_eq!(paused_frame.fixed_step_plan().step_count, 0);
    assert_eq!(scaled_frame.virtual_delta(), Duration::from_micros(12_500));
    assert_eq!(scaled_frame.fixed_step_plan().step_count, 1);
    assert_eq!(
        scaled_frame.fixed_step_plan().remaining_overstep,
        Duration::from_micros(2_500)
    );
}

#[test]
fn policy_transactions_advance_only_the_changed_clock_domain_epoch() {
    let mut controller = WorldTimeController::default();
    let before = controller.state();
    let receipt = controller
        .apply_time_policy(crate::core::TimePolicyTransaction::new(
            controller
                .time_policy()
                .with_fixed_timestep(Duration::from_millis(10)),
        ))
        .expect("valid policy");
    let after = controller.state();

    assert!(receipt.changed());
    assert_eq!(receipt.generation(), 1);
    assert_eq!(
        after.virtual_time().clock_domain_stamp().epoch(),
        before.virtual_time().clock_domain_stamp().epoch()
    );
    assert_eq!(
        after.fixed_time().clock_domain_stamp().epoch(),
        before
            .fixed_time()
            .clock_domain_stamp()
            .epoch()
            .saturating_add(1)
    );
}

#[test]
fn fixed_steps_stay_uncommitted_until_each_explicit_commit() {
    let outer = CoreRuntime::new();
    let policy = TimePolicy::default().with_fixed_timestep(Duration::from_millis(10));
    let mut controller = WorldTimeController::new(policy).expect("valid policy");

    let frame = controller
        .advance(outer.advance_time_by(Duration::from_millis(25), 8))
        .expect("first frame should be accepted");
    assert_eq!(frame.fixed_step_plan().step_count, 2);
    assert_eq!(controller.state().fixed_time().elapsed(), Duration::ZERO);
    assert_eq!(
        controller.state().fixed_time().overstep(),
        Duration::from_millis(25)
    );

    let first = controller
        .begin_fixed_step(17)
        .expect("first fixed step should begin");
    assert_eq!(first.id().world_generation(), 17);
    assert_eq!(first.id().fixed_epoch(), 0);
    assert_eq!(first.id().tick_index(), 1);
    assert_eq!(first.elapsed(), Duration::from_millis(10));
    assert_eq!(controller.state().fixed_time().elapsed(), Duration::ZERO);

    controller
        .commit_fixed_step(&first)
        .expect("first fixed step should commit");
    assert_eq!(
        controller.state().fixed_time().elapsed(),
        Duration::from_millis(10)
    );
    assert_eq!(
        controller.state().fixed_time().overstep(),
        Duration::from_millis(15)
    );

    let second = controller
        .begin_fixed_step(17)
        .expect("second fixed step should begin");
    assert_eq!(second.id().tick_index(), 2);
    assert_eq!(second.elapsed(), Duration::from_millis(20));
    controller
        .commit_fixed_step(&second)
        .expect("second fixed step should commit");

    let committed = controller.state().fixed_time();
    assert_eq!(committed.frame_index(), 2);
    assert_eq!(committed.elapsed(), Duration::from_millis(20));
    assert_eq!(committed.overstep(), Duration::from_millis(5));
}

#[test]
fn aborting_a_fixed_step_preserves_debt_and_does_not_skip_its_tick_identity() {
    let outer = CoreRuntime::new();
    let policy = TimePolicy::default().with_fixed_timestep(Duration::from_millis(10));
    let mut controller = WorldTimeController::new(policy).expect("valid policy");

    controller
        .advance(outer.advance_time_by(Duration::from_millis(20), 8))
        .expect("first frame should be accepted");
    let failed = controller
        .begin_fixed_step(3)
        .expect("fixed step should begin");
    assert_eq!(failed.id().tick_index(), 1);
    controller
        .abort_fixed_step(failed)
        .expect("fixed step should abort");

    let after_abort = controller.state().fixed_time();
    assert_eq!(after_abort.frame_index(), 0);
    assert_eq!(after_abort.elapsed(), Duration::ZERO);
    assert_eq!(after_abort.overstep(), Duration::from_millis(20));

    let retry = controller
        .begin_fixed_step(3)
        .expect("aborted fixed step should be retryable");
    assert_eq!(retry.id().tick_index(), 1);
}

#[test]
fn time_policy_changes_reject_while_a_fixed_step_is_active() {
    let outer = CoreRuntime::new();
    let policy = TimePolicy::default().with_fixed_timestep(Duration::from_millis(10));
    let mut controller = WorldTimeController::new(policy).expect("valid policy");

    controller
        .advance(outer.advance_time_by(Duration::from_millis(10), 8))
        .expect("first frame should be accepted");
    let active = controller
        .begin_fixed_step(1)
        .expect("fixed step should begin");
    assert_eq!(
        controller.apply_time_policy(TimePolicyTransaction::new(
            policy.with_fixed_timestep(Duration::from_millis(5)),
        )),
        Err(TimePolicyError::FixedStepActive)
    );
    controller
        .abort_fixed_step(active)
        .expect("fixed step should abort");
}

#[test]
fn fixed_timestep_policy_changes_reject_while_fixed_debt_is_pending() {
    let outer = CoreRuntime::new();
    let policy = TimePolicy::default().with_fixed_timestep(Duration::from_millis(10));
    let mut controller = WorldTimeController::new(policy).expect("valid policy");

    controller
        .advance(outer.advance_time_by(Duration::from_millis(15), 8))
        .expect("first frame should be accepted");
    let before = controller.state();
    assert_eq!(
        controller.apply_time_policy(TimePolicyTransaction::new(
            policy.with_fixed_timestep(Duration::from_millis(5)),
        )),
        Err(TimePolicyError::FixedStepDebtPending {
            remaining: Duration::from_millis(15),
        })
    );
    assert_eq!(controller.state(), before);

    controller
        .apply_time_policy(TimePolicyTransaction::new(
            policy.with_virtual_relative_speed(0.5),
        ))
        .expect("virtual policy changes do not reinterpret fixed debt");
}

#[test]
fn fixed_interpolation_observes_only_committed_steps_and_actual_remaining_debt() {
    let outer = CoreRuntime::new();
    let policy = TimePolicy::default().with_fixed_timestep(Duration::from_millis(10));
    let mut controller = WorldTimeController::new(policy).expect("valid policy");

    controller
        .advance(outer.advance_time_by(Duration::from_millis(25), 8))
        .expect("first frame should be accepted");
    let before_commit = controller.fixed_interpolation_context();
    assert_eq!(before_commit.previous().simulation_tick(), None);
    assert_eq!(before_commit.previous().elapsed(), Duration::ZERO);
    assert_eq!(before_commit.current().simulation_tick(), None);
    assert_eq!(before_commit.current().elapsed(), Duration::ZERO);
    assert_eq!(before_commit.remaining_debt(), Duration::from_millis(25));
    assert_eq!(before_commit.fraction(), 0.5);

    let first = controller
        .begin_fixed_step(17)
        .expect("first fixed step should begin");
    controller
        .commit_fixed_step(&first)
        .expect("first fixed step should commit");
    let after_first_commit = controller.fixed_interpolation_context();
    assert_eq!(after_first_commit.previous().simulation_tick(), None);
    assert_eq!(
        after_first_commit
            .current()
            .simulation_tick()
            .map(|tick| tick.tick_index()),
        Some(1)
    );
    assert_eq!(
        after_first_commit.current().elapsed(),
        Duration::from_millis(10)
    );
    assert_eq!(
        after_first_commit.remaining_debt(),
        Duration::from_millis(15)
    );
    assert_eq!(after_first_commit.fraction(), 0.5);

    let failed = controller
        .begin_fixed_step(17)
        .expect("second fixed step should begin");
    controller
        .abort_fixed_step(failed)
        .expect("fixed step should abort");
    let after_abort = controller.fixed_interpolation_context();
    assert_eq!(after_abort, after_first_commit);

    let second = controller
        .begin_fixed_step(17)
        .expect("aborted fixed step should be retryable");
    controller
        .commit_fixed_step(&second)
        .expect("second fixed step should commit");
    let after_second_commit = controller.fixed_interpolation_context();
    assert_eq!(
        after_second_commit
            .previous()
            .simulation_tick()
            .map(|tick| tick.tick_index()),
        Some(1)
    );
    assert_eq!(
        after_second_commit
            .current()
            .simulation_tick()
            .map(|tick| tick.tick_index()),
        Some(2)
    );
    assert_eq!(
        after_second_commit.remaining_debt(),
        Duration::from_millis(5)
    );
    assert_eq!(after_second_commit.fraction(), 0.5);
}
