use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use std::{hint::black_box, time::Instant};

use crate::core::framework::scene::SCENE_MODULE_NAME;
use crate::core::{ClockDiscontinuity, CoreError, CoreRuntime, TimePolicyTransaction};
use crate::plugin::RuntimeExtensionRegistry;
use crate::scene::{
    LevelTickError, SystemStage, World, WorldTimeAdvanceError, WorldTimeControlError,
    create_default_level, module_descriptor,
};

#[test]
fn worlds_derive_virtual_and_fixed_time_independently_from_one_outer_frame() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    runtime
        .apply_time_policy(TimePolicyTransaction::new(
            runtime
                .time_policy()
                .with_fixed_timestep(Duration::from_millis(10)),
        ))
        .expect("default policy for newly created Worlds should be valid");
    let paused = create_default_level(&runtime.handle()).unwrap();
    let running = create_default_level(&runtime.handle()).unwrap();

    paused.pause_virtual_time();
    let outer = runtime.advance_time_by(Duration::from_millis(25), 8);
    paused.tick(&runtime.handle(), outer).unwrap();
    running.tick(&runtime.handle(), outer).unwrap();

    let paused_time = paused.world_time();
    let running_time = running.world_time();
    assert!(paused_time.virtual_time().is_paused());
    assert_eq!(paused_time.virtual_time().elapsed(), Duration::ZERO);
    assert_eq!(paused_time.fixed_time().elapsed(), Duration::ZERO);
    assert_eq!(
        running_time.virtual_time().elapsed(),
        Duration::from_millis(25)
    );
    assert_eq!(
        running_time.fixed_time().elapsed(),
        Duration::from_millis(20)
    );
    assert_eq!(
        running_time.fixed_time().overstep(),
        Duration::from_millis(5)
    );
}

#[test]
fn paused_world_single_step_advances_one_fixed_tick_then_stays_paused() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level
        .apply_time_policy(TimePolicyTransaction::new(
            level
                .time_policy()
                .with_fixed_timestep(Duration::from_millis(10)),
        ))
        .expect("World-local fixed policy should be valid");
    let fixed_update_calls = Arc::new(AtomicUsize::new(0));
    let calls_for_system = Arc::clone(&fixed_update_calls);
    let virtual_update_calls = Arc::new(AtomicUsize::new(0));
    let virtual_calls_for_system = Arc::clone(&virtual_update_calls);
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime22.single-step")
        .unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "runtime22.single-step.fixed-update",
            SystemStage::FixedUpdate,
            move || {
                let calls = Arc::clone(&calls_for_system);
                move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "runtime22.single-step.virtual-update",
            SystemStage::Update,
            move || {
                let calls = Arc::clone(&virtual_calls_for_system);
                move |_| {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    let plan = registry.world_runtime_extension_plan().unwrap();
    level
        .with_world_mut(|world| plan.apply_to_world(world))
        .unwrap();
    level.pause_virtual_time();

    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(16), 8),
        )
        .expect("paused frame should be accepted");
    assert_eq!(level.world_time().fixed_time().frame_index(), 0);
    assert_eq!(fixed_update_calls.load(Ordering::SeqCst), 0);
    assert_eq!(virtual_update_calls.load(Ordering::SeqCst), 0);

    level
        .request_single_step()
        .expect("paused World should accept one single-step request");
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(16), 8),
        )
        .expect("single-step frame should be accepted");
    let stepped = level.world_time();
    assert!(stepped.virtual_time().is_paused());
    assert_eq!(stepped.virtual_time().elapsed(), Duration::ZERO);
    assert_eq!(stepped.fixed_time().frame_index(), 1);
    assert_eq!(stepped.fixed_time().elapsed(), Duration::from_millis(10));
    assert_eq!(fixed_update_calls.load(Ordering::SeqCst), 1);
    assert_eq!(virtual_update_calls.load(Ordering::SeqCst), 0);

    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(16), 8),
        )
        .expect("paused frame after single-step should be accepted");
    assert_eq!(level.world_time().fixed_time().frame_index(), 1);
    assert_eq!(fixed_update_calls.load(Ordering::SeqCst), 1);
    assert_eq!(virtual_update_calls.load(Ordering::SeqCst), 0);
}

#[test]
fn paused_single_step_consumes_only_one_preexisting_debt_step() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level
        .apply_time_policy(TimePolicyTransaction::new(
            level
                .time_policy()
                .with_fixed_timestep(Duration::from_millis(10)),
        ))
        .unwrap();

    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(25), 0),
        )
        .expect("zero fixed budget should retain all accumulated debt");
    assert_eq!(
        level.world_time().fixed_time().overstep(),
        Duration::from_millis(25)
    );
    level.pause_virtual_time();
    level.request_single_step().unwrap();

    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::ZERO, 8),
        )
        .expect("single-step should consume exactly one existing debt step");
    let stepped = level.world_time().fixed_time();
    assert_eq!(stepped.frame_index(), 1);
    assert_eq!(stepped.elapsed(), Duration::from_millis(10));
    assert_eq!(stepped.overstep(), Duration::from_millis(15));

    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::ZERO, 8),
        )
        .expect("remaining debt must stay parked while the World remains paused");
    let parked = level.world_time().fixed_time();
    assert_eq!(parked.frame_index(), 1);
    assert_eq!(parked.overstep(), Duration::from_millis(15));
}

#[test]
fn single_step_rejects_unpaused_and_duplicate_requests() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();

    assert_eq!(
        level.request_single_step(),
        Err(WorldTimeControlError::SingleStepRequiresPause)
    );
    level.pause_virtual_time();
    level
        .request_single_step()
        .expect("first request should be accepted");
    assert_eq!(
        level.request_single_step(),
        Err(WorldTimeControlError::SingleStepAlreadyRequested)
    );

    level.unpause_virtual_time();
    level.pause_virtual_time();
    level
        .request_single_step()
        .expect("resume should cancel an unconsumed single-step request");
}

#[test]
fn world_replacement_cancels_unconsumed_single_step() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level.pause_virtual_time();
    level.request_single_step().unwrap();

    level.replace_world_and_reset_runtime_state(World::empty());
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(16), 8),
        )
        .expect("replacement should discard control requests owned by the retired World");

    assert_eq!(level.world_time().fixed_time().frame_index(), 0);
}

#[test]
fn failed_single_step_retries_without_duplicating_fixed_debt() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level
        .apply_time_policy(TimePolicyTransaction::new(
            level
                .time_policy()
                .with_fixed_timestep(Duration::from_millis(10)),
        ))
        .unwrap();
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_system = Arc::clone(&attempts);
    let mut registry = RuntimeExtensionRegistry::default();
    let owner = registry
        .intern_plugin_module("runtime22.single-step-retry")
        .unwrap();
    registry
        .register_runtime_scene_system(
            owner,
            "runtime22.single-step-retry.fixed-update",
            SystemStage::FixedUpdate,
            move || {
                let attempts = Arc::clone(&attempts_for_system);
                move |_| {
                    if attempts.fetch_add(1, Ordering::SeqCst) == 0 {
                        return Err(CoreError::Initialization(
                            "runtime22-single-step".to_string(),
                            "injected retry failure".to_string(),
                        ));
                    }
                    Ok(())
                }
            },
        )
        .register()
        .unwrap();
    let plan = registry.world_runtime_extension_plan().unwrap();
    level
        .with_world_mut(|world| plan.apply_to_world(world))
        .unwrap();
    level.pause_virtual_time();
    level.request_single_step().unwrap();

    let error = level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(16), 8),
        )
        .expect_err("first single-step attempt should fail");
    let receipt = error
        .fixed_step_receipt()
        .expect("failed single-step must retain transaction evidence");
    assert_eq!(receipt.tick().tick_index(), 1);
    assert_eq!(receipt.committed_steps(), 0);
    assert_eq!(receipt.remaining_debt(), Duration::from_millis(10));
    let failed = level.world_time().fixed_time();
    assert_eq!(failed.frame_index(), 0);
    assert_eq!(failed.overstep(), Duration::from_millis(10));

    level.request_single_step().unwrap();
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(16), 8),
        )
        .expect("explicit retry should commit the original fixed debt");
    let retried = level.world_time().fixed_time();
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert_eq!(retried.frame_index(), 1);
    assert_eq!(retried.elapsed(), Duration::from_millis(10));
    assert_eq!(retried.overstep(), Duration::ZERO);
}

#[test]
fn rejected_outer_frame_does_not_consume_pending_single_step() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level.pause_virtual_time();
    let first = runtime.advance_time_by(Duration::from_millis(16), 8);
    level.tick(&runtime.handle(), first).unwrap();
    level.request_single_step().unwrap();

    assert!(matches!(
        level.tick(&runtime.handle(), first),
        Err(LevelTickError::WorldTime(
            WorldTimeAdvanceError::DuplicateOuterFrame { frame_index: 1 }
        ))
    ));
    level
        .tick(
            &runtime.handle(),
            runtime.advance_time_by(Duration::from_millis(16), 8),
        )
        .expect("successor frame should retain and execute the pending single-step");

    assert_eq!(level.world_time().fixed_time().frame_index(), 1);
}

#[test]
fn world_fixed_debt_uses_the_outer_budget_not_another_clock_step_count() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    level
        .apply_time_policy(TimePolicyTransaction::new(
            level
                .time_policy()
                .with_fixed_timestep(Duration::from_millis(1)),
        ))
        .expect("World-local fixed policy should be valid");

    let outer = runtime.advance_time_by(Duration::from_millis(25), 4);
    assert_eq!(outer.fixed_step_budget(), 4);
    level.tick(&runtime.handle(), outer).unwrap();

    assert_eq!(level.world_time().fixed_time().frame_index(), 4);
    assert_eq!(
        level.world_time().fixed_time().overstep(),
        Duration::from_millis(21)
    );
}

#[test]
fn world_rejects_duplicate_outer_frame_before_mutating_time() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let outer = runtime.advance_time_by(Duration::from_millis(25), 4);

    level.tick(&runtime.handle(), outer).unwrap();
    let before = level.world_time();
    assert_eq!(before.last_outer_frame_index(), Some(1));
    let error = level
        .tick(&runtime.handle(), outer)
        .expect_err("one Level must consume an outer frame at most once");

    assert!(matches!(
        error,
        LevelTickError::WorldTime(WorldTimeAdvanceError::DuplicateOuterFrame { frame_index: 1 })
    ));
    assert_eq!(level.world_time(), before);
}

#[test]
fn world_rejects_decreasing_outer_frame_before_mutating_time() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let first = runtime.advance_time_by(Duration::from_millis(11), 4);
    let second = runtime.advance_time_by(Duration::from_millis(17), 4);

    level.tick(&runtime.handle(), second).unwrap();
    let before = level.world_time();
    assert_eq!(before.last_outer_frame_index(), Some(2));
    let error = level
        .tick(&runtime.handle(), first)
        .expect_err("a Level must reject an older outer frame after a newer one");

    assert!(matches!(
        error,
        LevelTickError::WorldTime(WorldTimeAdvanceError::OutOfOrderOuterFrame {
            last_consumed: 2,
            submitted: 1,
        })
    ));
    assert_eq!(level.world_time(), before);
}

#[test]
fn world_rejects_skipped_outer_frame_without_discontinuity() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let first = runtime.advance_time_by(Duration::from_millis(11), 4);
    level.tick(&runtime.handle(), first).unwrap();
    let _skipped = runtime.advance_time_by(Duration::from_millis(17), 4);
    let third = runtime.advance_time_by(Duration::from_millis(23), 4);
    let before = level.world_time();

    let error = level
        .tick(&runtime.handle(), third)
        .expect_err("a missing outer frame requires an explicit discontinuity");

    assert!(matches!(
        error,
        LevelTickError::WorldTime(WorldTimeAdvanceError::SkippedOuterFrames {
            last_consumed: 1,
            submitted: 3,
        })
    ));
    assert_eq!(level.world_time(), before);
}

#[test]
fn world_accepts_skipped_outer_frames_with_explicit_discontinuity() {
    let runtime = CoreRuntime::new();
    runtime.register_module(module_descriptor()).unwrap();
    runtime.activate_module(SCENE_MODULE_NAME).unwrap();
    let level = create_default_level(&runtime.handle()).unwrap();
    let first = runtime.advance_time_by(Duration::from_millis(11), 4);
    level.tick(&runtime.handle(), first).unwrap();
    let _skipped = runtime.advance_time_by(Duration::from_millis(17), 4);

    runtime.submit_clock_discontinuity(ClockDiscontinuity::WindowSurfaceRecreated);
    let rebased = runtime.tick_time(4);
    assert!(rebased.discontinuity().is_some());
    level
        .tick(&runtime.handle(), rebased)
        .expect("an explicit lifecycle discontinuity may rebase a skipped Level frame");

    assert_eq!(
        level.world_time().last_outer_frame_index(),
        Some(rebased.outer_frame_index())
    );
}

#[test]
#[ignore = "release-only performance acceptance"]
fn runtime22_performance_outer_frame_snapshot_projection_profile() {
    assert!(
        !cfg!(debug_assertions),
        "outer-frame projection performance evidence must run in release mode"
    );
    let runtime = CoreRuntime::new();
    let snapshot = runtime.advance_time_by(Duration::from_millis(16), 8);
    const WARMUPS: usize = 10;
    const SAMPLES: usize = 31;

    let mut largest_legacy_p95 = Duration::ZERO;
    let mut largest_snapshot_p95 = Duration::ZERO;
    for context_count in [1_usize, 64, 1024] {
        for sample in 0..WARMUPS {
            if sample % 2 == 0 {
                let _ = legacy_projection_sample(&runtime, context_count);
                let _ = snapshot_projection_sample(snapshot, context_count);
            } else {
                let _ = snapshot_projection_sample(snapshot, context_count);
                let _ = legacy_projection_sample(&runtime, context_count);
            }
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        let mut snapshot_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            if sample % 2 == 0 {
                legacy_samples.push(legacy_projection_sample(&runtime, context_count));
                snapshot_samples.push(snapshot_projection_sample(snapshot, context_count));
            } else {
                snapshot_samples.push(snapshot_projection_sample(snapshot, context_count));
                legacy_samples.push(legacy_projection_sample(&runtime, context_count));
            }
        }

        let legacy_p50 = nearest_rank(&mut legacy_samples, 50);
        let legacy_p95 = nearest_rank(&mut legacy_samples, 95);
        let snapshot_p50 = nearest_rank(&mut snapshot_samples, 50);
        let snapshot_p95 = nearest_rank(&mut snapshot_samples, 95);
        println!(
            "PERF_RESULT runtime22.outer_frame_snapshot profile=release contexts={context_count} samples={SAMPLES} legacy_p50_ns={} legacy_p95_ns={} snapshot_p50_ns={} snapshot_p95_ns={} core_lock_reads_before={context_count} core_lock_reads_after=0",
            legacy_p50.as_nanos(),
            legacy_p95.as_nanos(),
            snapshot_p50.as_nanos(),
            snapshot_p95.as_nanos(),
        );
        if context_count == 1024 {
            largest_legacy_p95 = legacy_p95;
            largest_snapshot_p95 = snapshot_p95;
        }
    }

    assert!(
        largest_snapshot_p95.as_nanos().saturating_mul(4)
            <= largest_legacy_p95.as_nanos().saturating_mul(3),
        "1024 immutable snapshot projections must reduce P95 by at least 25%: legacy={largest_legacy_p95:?}, snapshot={largest_snapshot_p95:?}"
    );
}

fn legacy_projection_sample(runtime: &CoreRuntime, context_count: usize) -> Duration {
    let started = Instant::now();
    for _ in 0..context_count {
        black_box(runtime.real_time().elapsed());
    }
    started.elapsed()
}

fn snapshot_projection_sample(
    snapshot: crate::core::FrameTimeSnapshot,
    context_count: usize,
) -> Duration {
    let started = Instant::now();
    for _ in 0..context_count {
        black_box(black_box(snapshot).real_elapsed());
    }
    started.elapsed()
}

fn nearest_rank(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let rank = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1);
    samples[rank]
}
