use super::*;
use crate::core::CoreRuntime;
use crate::scene::{
    NodeKind, WorldReplacementCommit, WorldReplacementError, WorldReplacementReason,
    WorldReplacementRejected, WorldReplacementRequest, WorldReplacementTimePolicy,
};
use std::time::Duration;

fn replacement_world(nodes: usize) -> World {
    let mut world = World::empty();
    for _ in 0..nodes {
        world.spawn_node(NodeKind::Empty).unwrap();
    }
    world
}

fn replacement_node_ids(world: &World) -> Vec<EntityId> {
    world.node_records().into_iter().map(|node| node.id).collect()
}

fn replacement_level(id: u64, world: World, policy: TimePolicy) -> LevelSystem {
    let level = LevelSystem::new(
        WorldHandle(id),
        Arc::new(Mutex::new(world)),
        LevelMetadata::default(),
    );
    level.initialize_time_policy(policy).expect("valid fixture policy");
    level
}

fn replacement_policy() -> TimePolicy {
    TimePolicy::default()
        .with_fixed_timestep(Duration::from_millis(10))
        .with_virtual_relative_speed(1.5)
}

fn replacement_observed(level: &LevelSystem) -> (u64, u64, WorldTimeState, Vec<EntityId>) {
    (
        level.world_generation(),
        level.capture_world_replacement_epoch(),
        level.world_time(),
        level.with_world(replacement_node_ids),
    )
}

fn replacement_committed(
    result: Result<WorldReplacementCommit, WorldReplacementRejected>,
) -> WorldReplacementCommit {
    result.expect("fixture replacement should commit")
}

fn replacement_rejected(
    result: Result<WorldReplacementCommit, WorldReplacementRejected>,
) -> WorldReplacementRejected {
    result.expect_err("fixture replacement should be rejected")
}

#[test]
fn legacy_preserve_retains_time_debt_and_the_old_active_step_settlement_path() {
    let outer = CoreRuntime::new();
    let level = replacement_level(1, replacement_world(1), replacement_policy());
    level
        .advance_world_time(outer.advance_time_by(Duration::from_millis(25), 2))
        .unwrap();
    let old_generation = level.world_generation();
    let old_step = level.begin_fixed_step(old_generation).unwrap();
    level.pause_virtual_time();
    let before = level.world_time();
    assert!(before.fixed_time().overstep() > Duration::ZERO);
    assert!(before.virtual_time().elapsed() > Duration::ZERO);
    assert!(before.virtual_time().is_paused());
    assert_eq!(before.virtual_time().relative_speed_f64(), 1.5);

    level.replace(replacement_world(2));
    assert_eq!(level.world_time(), before);
    assert!(level.world_generation() > old_generation);
    assert!(matches!(
        level.commit_fixed_step(old_generation, &old_step),
        Err(WorldFixedStepError::WorldGenerationChanged { .. })
    ));
    level.abort_fixed_step(old_step).unwrap();

    let generation = level.world_generation();
    let step = level.begin_fixed_step(generation).unwrap();
    level.commit_fixed_step(generation, &step).unwrap();
    let before_second = level.world_time();
    level.replace_world_and_reset_runtime_state(replacement_world(3));
    assert_eq!(level.world_time(), before_second);
    let generation = level.world_generation();
    let step = level.begin_fixed_step(generation).unwrap();
    level.commit_fixed_step(generation, &step).unwrap();
}

#[test]
fn explicit_preserve_retains_the_actual_retired_world_and_full_time_state() {
    let outer = CoreRuntime::new();
    let original = replacement_world(1);
    let original_ids = replacement_node_ids(&original);
    let level = replacement_level(2, original, replacement_policy());
    level
        .advance_world_time(outer.advance_time_by(Duration::from_millis(25), 0))
        .unwrap();
    level.pause_virtual_time();
    level.request_single_step().unwrap();
    let before = level.world_time();
    assert!(before.fixed_time().overstep() > Duration::ZERO);
    let incoming = replacement_world(2);
    let incoming_ids = replacement_node_ids(&incoming);
    let expected = level.capture_world_replacement_expectation();
    let (receipt, retired) = replacement_committed(level.replace_world_with_time_policy(
        &expected,
        WorldReplacementRequest::preserve(incoming),
        WorldReplacementReason::ExplicitHostRequest,
    ))
    .into_parts();

    assert_eq!(receipt.time_policy(), WorldReplacementTimePolicy::Preserve);
    assert_eq!(receipt.before(), before);
    assert_eq!(receipt.after(), before);
    assert_eq!(level.world_time(), before);
    assert_eq!(receipt.preserved_debt(), before.fixed_time().overstep());
    assert_eq!(retired.time(), before);
    assert_eq!(retired.identity(), receipt.previous());
    assert_eq!(replacement_node_ids(retired.world()), original_ids);
    assert_eq!(level.with_world(replacement_node_ids), incoming_ids);
    assert!(receipt.current().world_generation() > receipt.previous().world_generation());
    assert_eq!(
        receipt.current().replacement_epoch(),
        receipt.previous().replacement_epoch() + 1,
    );
    level.request_single_step().unwrap();
}

#[test]
fn reset_and_restore_rebase_domains_without_rewinding_the_live_outer_frame() {
    let outer = CoreRuntime::new();
    let original = replacement_world(1);
    let original_ids = replacement_node_ids(&original);
    let level = replacement_level(3, original, replacement_policy());
    level
        .advance_world_time(outer.advance_time_by(Duration::from_millis(25), 2))
        .unwrap();
    for _ in 0..2 {
        let generation = level.world_generation();
        let step = level.begin_fixed_step(generation).unwrap();
        level.commit_fixed_step(generation, &step).unwrap();
    }
    level.pause_virtual_time();
    let saved_time = level.world_time();
    assert!(saved_time.virtual_time().elapsed() > Duration::ZERO);
    assert!(saved_time.fixed_time().elapsed() > Duration::ZERO);
    assert!(saved_time.fixed_time().overstep() > Duration::ZERO);

    let reset_world = replacement_world(3);
    let reset_ids = replacement_node_ids(&reset_world);
    let reset_policy = TimePolicy::default()
        .with_fixed_timestep(Duration::from_millis(8))
        .with_virtual_relative_speed(0.5);
    let expected = level.capture_world_replacement_expectation();
    let (reset_receipt, saved) = replacement_committed(level.replace_world_with_time_policy(
        &expected,
        WorldReplacementRequest::reset(reset_world, reset_policy, false),
        WorldReplacementReason::SceneTravel,
    ))
    .into_parts();
    let reset = reset_receipt.after();
    assert_eq!(reset_receipt.time_policy(), WorldReplacementTimePolicy::Reset);
    assert_eq!(reset_receipt.before(), saved_time);
    assert_eq!(reset_receipt.discarded_debt(), saved_time.fixed_time().overstep());
    assert_eq!(saved.time(), saved_time);
    assert_eq!(saved.identity(), reset_receipt.previous());
    assert_eq!(replacement_node_ids(saved.world()), original_ids);
    assert_eq!(level.with_world(replacement_node_ids), reset_ids);
    assert_eq!(reset.virtual_time().delta(), Duration::ZERO);
    assert_eq!(reset.virtual_time().elapsed(), Duration::ZERO);
    assert_eq!(reset.virtual_time().frame_index(), 0);
    assert_eq!(reset.fixed_time().delta(), Duration::ZERO);
    assert_eq!(reset.fixed_time().elapsed(), Duration::ZERO);
    assert_eq!(reset.fixed_time().frame_index(), 0);
    assert_eq!(reset.fixed_time().overstep(), Duration::ZERO);
    assert_eq!(reset.fixed_time().timestep(), Duration::from_millis(8));
    assert_eq!(reset.virtual_time().relative_speed_f64(), 0.5);
    assert!(!reset.virtual_time().is_paused());
    assert_eq!(reset.last_outer_frame_index(), saved_time.last_outer_frame_index());
    assert_eq!(reset.policy_generation(), saved_time.policy_generation() + 1);
    assert_eq!(
        reset.virtual_time().clock_domain_stamp().epoch(),
        saved_time.virtual_time().clock_domain_stamp().epoch() + 1,
    );
    assert_eq!(
        reset.fixed_time().clock_domain_stamp().epoch(),
        saved_time.fixed_time().clock_domain_stamp().epoch() + 1,
    );
    assert_eq!(
        reset.virtual_time().clock_domain_stamp().source_generation(),
        saved_time.virtual_time().clock_domain_stamp().source_generation(),
    );
    assert_eq!(
        reset.fixed_time().clock_domain_stamp().source_generation(),
        saved_time.fixed_time().clock_domain_stamp().source_generation(),
    );

    let live_outer = outer.advance_time_by(Duration::from_millis(40), 0);
    level.advance_world_time(live_outer).unwrap();
    let live_time = level.world_time();
    assert_ne!(live_time.last_outer_frame_index(), saved_time.last_outer_frame_index());
    let saved_identity = saved.identity();
    let expected = level.capture_world_replacement_expectation();
    let (restore_receipt, retired_reset_world) =
        replacement_committed(level.replace_world_with_time_policy(
            &expected,
            WorldReplacementRequest::restore(saved),
            WorldReplacementReason::CheckpointRestore,
        ))
        .into_parts();
    let restored = restore_receipt.after();
    assert_eq!(restore_receipt.time_policy(), WorldReplacementTimePolicy::Restore);
    assert_eq!(restore_receipt.restored_checkpoint(), Some(saved_identity));
    assert_eq!(restore_receipt.before(), live_time);
    assert_eq!(restore_receipt.restored_debt(), saved_time.fixed_time().overstep());
    assert_eq!(retired_reset_world.time(), live_time);
    assert_eq!(retired_reset_world.identity(), restore_receipt.previous());
    assert_eq!(replacement_node_ids(retired_reset_world.world()), reset_ids);
    assert_eq!(level.with_world(replacement_node_ids), original_ids);
    assert_eq!(restored, level.world_time());
    assert_eq!(restored.virtual_time().elapsed(), saved_time.virtual_time().elapsed());
    assert_eq!(restored.virtual_time().frame_index(), saved_time.virtual_time().frame_index());
    assert_eq!(restored.virtual_time().delta(), Duration::ZERO);
    assert_eq!(restored.virtual_time().is_paused(), saved_time.virtual_time().is_paused());
    assert_eq!(
        restored.virtual_time().relative_speed_f64(),
        saved_time.virtual_time().relative_speed_f64(),
    );
    assert_eq!(restored.virtual_time().max_delta(), saved_time.virtual_time().max_delta());
    assert_eq!(restored.fixed_time().elapsed(), saved_time.fixed_time().elapsed());
    assert_eq!(restored.fixed_time().frame_index(), saved_time.fixed_time().frame_index());
    assert_eq!(restored.fixed_time().delta(), Duration::ZERO);
    assert_eq!(restored.fixed_time().timestep(), saved_time.fixed_time().timestep());
    assert_eq!(restored.fixed_time().overstep(), saved_time.fixed_time().overstep());
    assert_eq!(restored.last_outer_frame_index(), live_time.last_outer_frame_index());
    assert_eq!(restored.policy_generation(), live_time.policy_generation() + 1);
    let restored_virtual_stamp = restored.virtual_time().clock_domain_stamp();
    let restored_fixed_stamp = restored.fixed_time().clock_domain_stamp();
    let live_virtual_stamp = live_time.virtual_time().clock_domain_stamp();
    let live_fixed_stamp = live_time.fixed_time().clock_domain_stamp();
    assert_eq!(
        restored_virtual_stamp.epoch(),
        live_virtual_stamp.epoch().max(saved_time.virtual_time().clock_domain_stamp().epoch()) + 1,
    );
    assert_eq!(
        restored_fixed_stamp.epoch(),
        live_fixed_stamp.epoch().max(saved_time.fixed_time().clock_domain_stamp().epoch()) + 1,
    );
    assert_eq!(restored_virtual_stamp.source_generation(), live_virtual_stamp.source_generation());
    assert_eq!(restored_fixed_stamp.source_generation(), live_fixed_stamp.source_generation());
    assert_eq!(restored_virtual_stamp.id(), live_virtual_stamp.id());
    assert_eq!(restored_fixed_stamp.id(), live_fixed_stamp.id());
    assert_eq!(restored_virtual_stamp.unit(), live_virtual_stamp.unit());
    assert_eq!(restored_fixed_stamp.unit(), live_fixed_stamp.unit());

    let before_duplicate = level.world_time();
    assert!(matches!(
        level.advance_world_time(live_outer),
        Err(WorldTimeAdvanceError::DuplicateOuterFrame { .. })
    ));
    assert_eq!(level.world_time(), before_duplicate);
}

#[test]
fn rejected_owner_stale_and_invalid_requests_are_failure_atomic_and_retryable() {
    // Identical routing handles must not substitute for the actual Level owner.
    let a = replacement_level(4, replacement_world(1), replacement_policy());
    let b = replacement_level(4, replacement_world(2), replacement_policy());
    let incoming = replacement_world(3);
    let incoming_ids = replacement_node_ids(&incoming);
    let expected_a = a.capture_world_replacement_expectation();
    let before_b = replacement_observed(&b);
    let rejected = replacement_rejected(b.replace_world_with_time_policy(
        &expected_a,
        WorldReplacementRequest::preserve(incoming),
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(rejected.error(), WorldReplacementError::TargetOwnerChanged));
    assert_eq!(replacement_observed(&b), before_b);
    let (_, request) = rejected.into_parts();
    let (_, checkpoint_a) = replacement_committed(a.replace_world_with_time_policy(
        &expected_a,
        request,
        WorldReplacementReason::ExplicitHostRequest,
    ))
    .into_parts();
    assert_eq!(a.with_world(replacement_node_ids), incoming_ids);

    let checkpoint_identity = checkpoint_a.identity();
    let checkpoint_time = checkpoint_a.time();
    let checkpoint_ids = replacement_node_ids(checkpoint_a.world());
    let before_a = replacement_observed(&a);
    let before_b = replacement_observed(&b);
    let expected_b = b.capture_world_replacement_expectation();
    let rejected = replacement_rejected(b.replace_world_with_time_policy(
        &expected_b,
        WorldReplacementRequest::restore(checkpoint_a),
        WorldReplacementReason::CheckpointRestore,
    ));
    assert!(matches!(rejected.error(), WorldReplacementError::CheckpointOwnerChanged));
    assert_eq!(replacement_observed(&a), before_a);
    assert_eq!(replacement_observed(&b), before_b);
    let (_, request) = rejected.into_parts();
    let checkpoint_a = match request {
        WorldReplacementRequest::Restore { checkpoint } => checkpoint,
        _ => panic!("rejection must return the original Restore request"),
    };
    assert_eq!(checkpoint_a.identity(), checkpoint_identity);
    assert_eq!(checkpoint_a.time(), checkpoint_time);
    assert_eq!(replacement_node_ids(checkpoint_a.world()), checkpoint_ids);
    let expected_a = a.capture_world_replacement_expectation();
    replacement_committed(a.replace_world_with_time_policy(
        &expected_a,
        WorldReplacementRequest::restore(checkpoint_a),
        WorldReplacementReason::CheckpointRestore,
    ));
    assert_eq!(a.with_world(replacement_node_ids), checkpoint_ids);

    let stale = b.capture_world_replacement_expectation();
    b.replace(replacement_world(4));
    let incoming = replacement_world(5);
    let incoming_ids = replacement_node_ids(&incoming);
    let before_b = replacement_observed(&b);
    let rejected = replacement_rejected(b.replace_world_with_time_policy(
        &stale,
        WorldReplacementRequest::preserve(incoming),
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(rejected.error(), WorldReplacementError::TargetChanged { .. }));
    assert_eq!(replacement_observed(&b), before_b);
    let (_, request) = rejected.into_parts();
    let expected_b = b.capture_world_replacement_expectation();
    replacement_committed(b.replace_world_with_time_policy(
        &expected_b,
        request,
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert_eq!(b.with_world(replacement_node_ids), incoming_ids);

    let invalid = TimePolicy::default().with_fixed_timestep(Duration::ZERO);
    let expected_b = b.capture_world_replacement_expectation();
    let before_b = replacement_observed(&b);
    let rejected = replacement_rejected(b.replace_world_with_time_policy(
        &expected_b,
        WorldReplacementRequest::reset(replacement_world(6), invalid, false),
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(rejected.error(), WorldReplacementError::InvalidTimePolicy(_)));
    assert_eq!(replacement_observed(&b), before_b);
    let (_, original_request) = rejected.into_parts();
    // The returned owned request remains invalid and is not consumed on retry.
    let rejected = replacement_rejected(b.replace_world_with_time_policy(
        &expected_b,
        original_request,
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(rejected.error(), WorldReplacementError::InvalidTimePolicy(_)));
    assert_eq!(replacement_observed(&b), before_b);
    let (_, _still_owned_request) = rejected.into_parts();
}

#[test]
fn quiet_rejections_preserve_the_request_and_original_fixed_step_transactions() {
    let outer = CoreRuntime::new();
    let level = replacement_level(5, replacement_world(1), replacement_policy());
    level
        .advance_world_time(outer.advance_time_by(Duration::from_millis(25), 2))
        .unwrap();
    let generation = level.world_generation();
    let first = level.begin_fixed_step(generation).unwrap();
    let incoming = replacement_world(2);
    let incoming_ids = replacement_node_ids(&incoming);
    let expected = level.capture_world_replacement_expectation();
    let before = replacement_observed(&level);
    let rejected = replacement_rejected(level.replace_world_with_time_policy(
        &expected,
        WorldReplacementRequest::reset(incoming, TimePolicy::default(), false),
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(rejected.error(), WorldReplacementError::FixedStepActive { .. }));
    assert_eq!(replacement_observed(&level), before);
    let (_, request) = rejected.into_parts();
    level.commit_fixed_step(generation, &first).unwrap();

    let expected = level.capture_world_replacement_expectation();
    let before = replacement_observed(&level);
    let rejected = replacement_rejected(level.replace_world_with_time_policy(
        &expected,
        request,
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(
        rejected.error(),
        WorldReplacementError::FixedStepBudgetPending { .. }
    ));
    assert_eq!(replacement_observed(&level), before);
    let (_, request) = rejected.into_parts();
    let second = level.begin_fixed_step(generation).unwrap();
    level.commit_fixed_step(generation, &second).unwrap();
    let expected = level.capture_world_replacement_expectation();
    replacement_committed(level.replace_world_with_time_policy(
        &expected,
        request,
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert_eq!(level.with_world(replacement_node_ids), incoming_ids);
}

#[test]
fn epoch_and_incoming_generation_overflow_return_the_original_request() {
    let saturated = replacement_level(6, replacement_world(1), replacement_policy());
    saturated.world_replacement_epoch.store(u64::MAX, Ordering::Release);
    let expected = saturated.capture_world_replacement_expectation();
    let before = replacement_observed(&saturated);
    let incoming = replacement_world(2);
    let incoming_ids = replacement_node_ids(&incoming);
    let rejected = replacement_rejected(saturated.replace_world_with_time_policy(
        &expected,
        WorldReplacementRequest::preserve(incoming),
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(
        rejected.error(),
        WorldReplacementError::ReplacementEpochExhausted { current } if *current == u64::MAX
    ));
    assert_eq!(replacement_observed(&saturated), before);
    let (_, request) = rejected.into_parts();
    // Keep the exhausted owner unchanged; only the returned incoming World moves.
    let retry_target = replacement_level(7, replacement_world(1), replacement_policy());
    let expected = retry_target.capture_world_replacement_expectation();
    replacement_committed(retry_target.replace_world_with_time_policy(
        &expected,
        request,
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert_eq!(retry_target.with_world(replacement_node_ids), incoming_ids);
    assert_eq!(replacement_observed(&saturated), before);

    let mut incoming = replacement_world(3);
    incoming.advance_world_generation_after(u64::MAX - 1);
    assert_eq!(incoming.world_generation(), u64::MAX);
    let incoming_ids = replacement_node_ids(&incoming);
    let before = replacement_observed(&retry_target);
    let expected = retry_target.capture_world_replacement_expectation();
    let rejected = replacement_rejected(retry_target.replace_world_with_time_policy(
        &expected,
        WorldReplacementRequest::preserve(incoming),
        WorldReplacementReason::ExplicitHostRequest,
    ));
    assert!(matches!(
        rejected.error(),
        WorldReplacementError::WorldGenerationExhausted { .. }
    ));
    assert_eq!(replacement_observed(&retry_target), before);
    let (_, request) = rejected.into_parts();
    match request {
        WorldReplacementRequest::Preserve { world } => {
            assert_eq!(world.world_generation(), u64::MAX);
            assert_eq!(replacement_node_ids(&world), incoming_ids);

            // The old entry must still replace a MAX-generation World using
            // its original saturating arithmetic, rather than newly panicking.
            let time = retry_target.world_time();
            let epoch = retry_target.capture_world_replacement_epoch();
            retry_target.replace(world);
            assert_eq!(retry_target.world_generation(), u64::MAX);
            assert_eq!(retry_target.world_time(), time);
            assert_eq!(retry_target.with_world(replacement_node_ids), incoming_ids);
            assert_eq!(retry_target.capture_world_replacement_epoch(), epoch + 1);
        }
        _ => panic!("generation failure must return the original Preserve request"),
    }
}

#[test]
fn frame_begin_keeps_time_and_schedule_owner_paired_across_a_real_reset() {
    let outer = CoreRuntime::new();
    let sequential = replacement_level(8, replacement_world(1), replacement_policy());
    let (snapshot, generation, execution) = sequential
        .begin_world_frame(outer.advance_time_by(Duration::from_millis(5), 0))
        .unwrap();
    assert!(sequential.with_world(|world| execution.is_bound_to(world)));
    let expected = sequential.capture_world_replacement_expectation();
    let commit = replacement_committed(sequential.replace_world_with_time_policy(
        &expected,
        WorldReplacementRequest::reset(replacement_world(2), TimePolicy::default(), false),
        WorldReplacementReason::ExplicitHostRequest,
    ));
    let receipt = commit.receipt();
    assert_eq!(generation, receipt.previous().world_generation());
    assert_eq!(snapshot.virtual_elapsed(), receipt.before().virtual_time().elapsed());
    assert_eq!(snapshot.virtual_delta(), receipt.before().virtual_time().delta());
    assert_eq!(Some(snapshot.outer_frame_index()), receipt.before().last_outer_frame_index());
    assert!(!sequential.with_world(|world| execution.is_bound_to(world)));
    assert!(sequential.with_world_mut_if_schedule_execution(&execution, |_| ()).is_none());

    let outer = CoreRuntime::new();
    let level = Arc::new(replacement_level(9, replacement_world(1), replacement_policy()));
    level
        .advance_world_time(outer.advance_time_by(Duration::from_millis(7), 0))
        .unwrap();
    let frame = outer.advance_time_by(Duration::from_millis(5), 0);
    let expected = level.capture_world_replacement_expectation();
    let incoming = replacement_world(3);
    let incoming_ids = replacement_node_ids(&incoming);
    let reset_policy = TimePolicy::default()
        .with_fixed_timestep(Duration::from_millis(8))
        .with_virtual_relative_speed(0.5);
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let frame_level = Arc::clone(&level);
    let frame_barrier = Arc::clone(&barrier);
    let reader = std::thread::spawn(move || {
        frame_barrier.wait();
        frame_level.begin_world_frame(frame).unwrap()
    });
    let reset_level = Arc::clone(&level);
    let reset_barrier = Arc::clone(&barrier);
    let writer = std::thread::spawn(move || {
        reset_barrier.wait();
        reset_level.replace_world_with_time_policy(
            &expected,
            WorldReplacementRequest::reset(incoming, reset_policy, false),
            WorldReplacementReason::ExplicitHostRequest,
        )
    });
    barrier.wait();
    let (snapshot, generation, execution) = reader.join().expect("frame thread");
    let commit = replacement_committed(writer.join().expect("replacement thread"));
    let receipt = commit.receipt();
    let live = level.world_time();
    assert_eq!(level.with_world(replacement_node_ids), incoming_ids);
    assert_eq!(
        receipt.after().virtual_time().clock_domain_stamp().epoch(),
        receipt.before().virtual_time().clock_domain_stamp().epoch() + 1,
    );
    assert_eq!(
        receipt.after().fixed_time().clock_domain_stamp().epoch(),
        receipt.before().fixed_time().clock_domain_stamp().epoch() + 1,
    );
    if generation == receipt.previous().world_generation() {
        // Frame first: time, generation and execution retire together.
        assert_eq!(snapshot.virtual_elapsed(), receipt.before().virtual_time().elapsed());
        assert_eq!(snapshot.virtual_delta(), receipt.before().virtual_time().delta());
        assert_eq!(Some(snapshot.outer_frame_index()), receipt.before().last_outer_frame_index());
        assert_eq!(live, receipt.after());
        assert!(!level.with_world(|world| execution.is_bound_to(world)));
    } else {
        // Reset first: all three results belong to the new World.
        assert_eq!(generation, receipt.current().world_generation());
        assert_eq!(snapshot.virtual_elapsed(), live.virtual_time().elapsed());
        assert_eq!(snapshot.virtual_delta(), live.virtual_time().delta());
        assert_eq!(Some(snapshot.outer_frame_index()), live.last_outer_frame_index());
        assert!(live.virtual_time().elapsed() > receipt.after().virtual_time().elapsed());
        assert!(level.with_world(|world| execution.is_bound_to(world)));
    }
}
