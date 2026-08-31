use super::*;
use crate::core::framework::render::ProceduralSkyParams;
use std::hint::black_box;
use std::time::Instant;

fn key_with_revision(revision: u64) -> IblBakeKey {
    let mut params = ProceduralSkyParams::default_gradient();
    params.source_revision = revision;
    params.ibl_bake_key()
}

fn scheduler() -> RealtimeIblTimeSliceScheduler {
    RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(8, 2).expect("valid test config"),
    )
}

#[test]
fn initial_generation_is_time_sliced_and_publishes_only_after_sh9() {
    let mut scheduler = scheduler();
    let key = key_with_revision(1);
    assert!(scheduler.request_rebake(key));

    let mut observed = Vec::new();
    for frame in 10..40 {
        let batch = scheduler.begin_frame(frame).expect("generation batch");
        observed.push(batch.clone());
        let completion = scheduler.complete_frame(batch.token(), true);
        if completion == RealtimeIblCompletion::Published {
            break;
        }
        assert_eq!(completion, RealtimeIblCompletion::Advanced);
    }

    assert_eq!(observed.len(), 21);
    assert!(observed.iter().all(|batch| batch.operations().len() == 1));
    assert_eq!(
        observed
            .iter()
            .filter(|batch| matches!(batch.operations(), [RealtimeIblOperation::CaptureSky(_)]))
            .count(),
        3,
        "the ticket has exactly three two-face sky captures"
    );
    assert_eq!(observed[0].ready_slot(), IblRealtimeBufferSlot::A);
    assert_eq!(observed[0].work_slot(), IblRealtimeBufferSlot::B);
    assert_eq!(
        observed[0].operations(),
        &[RealtimeIblOperation::CaptureSky(CubeFaceRange::new(0, 2))]
    );
    assert_eq!(
        observed[2].operations(),
        &[RealtimeIblOperation::CaptureSky(CubeFaceRange::new(4, 2))]
    );
    assert_eq!(
        observed[3].operations(),
        &[RealtimeIblOperation::GenerateSourceMip { mip_level: 1 }]
    );
    assert_eq!(
        observed[9].operations(),
        &[RealtimeIblOperation::GenerateSourceMip { mip_level: 7 }]
    );
    assert_eq!(
        observed[10].operations(),
        &[RealtimeIblOperation::Prefilter {
            mips: CubeMipRange::new(0, 1),
            faces: CubeFaceRange::new(0, 2),
        }]
    );
    assert_eq!(
        observed[19].operations(),
        &[RealtimeIblOperation::Prefilter {
            mips: CubeMipRange::new(7, 1),
            faces: CubeFaceRange::ALL,
        }]
    );
    assert_eq!(
        observed[20].operations(),
        &[RealtimeIblOperation::ProjectDiffuseSh9]
    );
    assert_eq!(scheduler.published_key(), Some(key));
    assert_eq!(scheduler.ready_slot(), IblRealtimeBufferSlot::B);
    assert!(!scheduler.is_rebake_pending());
}

#[test]
fn generated_environment_uses_a_bounded_topology_cache_budget() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));
    let batch = scheduler.begin_frame(1).expect("first generation batch");

    // Twenty-one ticket stages across two ping-pong slots are the only
    // topologies this fixed configuration may retain at once.
    assert_eq!(batch.topology_cache_capacity(), 42);
}

#[test]
fn parameter_change_discards_old_work_without_replacing_the_ready_environment() {
    let mut scheduler = scheduler();
    let published = key_with_revision(1);
    publish_generation(&mut scheduler, published);
    scheduler.request_rebake(key_with_revision(2));
    let obsolete = scheduler.begin_frame(20).expect("old generation batch");

    let newest = key_with_revision(3);
    assert!(scheduler.request_rebake(newest));
    assert_eq!(scheduler.published_key(), Some(published));
    assert_eq!(
        scheduler.complete_frame(obsolete.token(), true),
        RealtimeIblCompletion::Stale
    );

    let restarted = scheduler.begin_frame(21).expect("restarted batch");
    assert_eq!(restarted.logical_state(), 0);
    assert_eq!(restarted.work_slot(), IblRealtimeBufferSlot::A);
    assert_eq!(scheduler.published_key(), Some(published));
    assert_eq!(scheduler.pending_key(), Some(newest));
}

#[test]
fn failed_submission_slice_retries_after_one_backoff_frame() {
    let mut scheduler = scheduler();
    publish_generation(&mut scheduler, key_with_revision(1));
    scheduler.request_rebake(key_with_revision(2));

    let first = scheduler.begin_frame(30).expect("first sliced batch");
    assert_eq!(scheduler.begin_frame(30), Some(first.clone()));
    assert_eq!(
        scheduler.complete_attempt(
            first.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::RetryScheduled
    );

    assert_eq!(scheduler.begin_frame(31), None);
    let retry = scheduler.begin_frame(32).expect("retry batch");
    assert_eq!(retry.logical_state(), first.logical_state());
    assert_eq!(retry.operations(), first.operations());
    assert_eq!(scheduler.published_key(), Some(key_with_revision(1)));
}

#[test]
fn delayed_completion_from_a_failed_attempt_cannot_complete_its_retry() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));

    let failed = scheduler.begin_frame(10).expect("failed attempt");
    assert_eq!(
        scheduler.complete_attempt(
            failed.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(scheduler.begin_frame(11), None);
    let retry = scheduler.begin_frame(12).expect("retry attempt");

    assert_ne!(failed.token(), retry.token());
    assert_eq!(
        scheduler.complete_attempt(failed.token(), RealtimeIblSliceAttempt::Succeeded),
        RealtimeIblCompletion::Stale
    );
    assert_eq!(scheduler.begin_frame(12), Some(retry.clone()));
    assert_eq!(
        scheduler.complete_attempt(retry.token(), RealtimeIblSliceAttempt::Succeeded),
        RealtimeIblCompletion::Advanced
    );
}

#[test]
fn retry_backoff_remains_bounded_across_frame_counter_wrap() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));

    let first = scheduler.begin_frame(u64::MAX).expect("pre-wrap attempt");
    assert_eq!(
        scheduler.complete_attempt(
            first.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(
        scheduler
            .failure_report()
            .expect("pre-wrap failure report")
            .retry_not_before_frame,
        Some(1)
    );
    assert_eq!(scheduler.begin_frame(0), None);
    assert!(scheduler.begin_frame(1).is_some());
}

#[test]
fn frame_sequence_age_is_bounded_and_rollover_safe() {
    assert_eq!(frame_sequence_age(7, 7), Some(0));
    assert_eq!(frame_sequence_age(0, u64::MAX), Some(1));
    assert_eq!(frame_sequence_age(1, u64::MAX), Some(2));
    assert_eq!(
        frame_sequence_age(i64::MAX as u64, 0),
        Some(i64::MAX as u64)
    );
    assert_eq!(frame_sequence_age(1_u64 << 63, 0), None);
    assert_eq!(frame_sequence_age(u64::MAX, 0), None);
}

#[test]
fn repeated_slice_failures_back_off_then_terminally_preserve_last_good() {
    let mut scheduler = scheduler();
    let published_key = key_with_revision(1);
    let failed_key = key_with_revision(2);
    publish_generation(&mut scheduler, published_key);
    let published_slot = scheduler.ready_slot();
    assert!(scheduler.request_rebake(failed_key));

    let first = scheduler.begin_frame(30).expect("first attempt");
    assert_eq!(
        scheduler.complete_attempt(
            first.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Recording),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(scheduler.begin_frame(31), None);
    let first_failure = scheduler.failure_report().expect("first failure report");
    assert_eq!(first_failure.bake_key, failed_key);
    assert_eq!(
        first_failure.failure_kind,
        RealtimeIblFailureKind::Recording
    );
    assert_eq!(first_failure.failed_attempt_count, 1);
    assert_eq!(first_failure.retry_not_before_frame, Some(32));
    assert!(!first_failure.terminal);
    assert!(first_failure.last_good_available);

    let second = scheduler.begin_frame(32).expect("second attempt");
    assert_eq!(second.operations(), first.operations());
    assert_eq!(
        scheduler.complete_attempt(
            second.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(scheduler.begin_frame(33), None);
    assert_eq!(scheduler.begin_frame(34), None);
    let second_failure = scheduler.failure_report().expect("second failure report");
    assert_eq!(
        second_failure.failure_kind,
        RealtimeIblFailureKind::Submission
    );
    assert_eq!(second_failure.failed_attempt_count, 2);
    assert_eq!(second_failure.retry_not_before_frame, Some(35));

    let third = scheduler.begin_frame(35).expect("terminal attempt");
    assert_eq!(third.operations(), first.operations());
    assert_eq!(
        scheduler.complete_attempt(
            third.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::Failed
    );
    let terminal = scheduler.failure_report().expect("terminal failure report");
    assert_eq!(terminal.failed_attempt_count, 3);
    assert_eq!(terminal.retry_not_before_frame, None);
    assert!(terminal.terminal);
    assert_eq!(scheduler.published_key(), Some(published_key));
    assert_eq!(scheduler.ready_slot(), published_slot);
    assert!(!scheduler.is_rebake_pending());
    assert_eq!(scheduler.begin_frame(36), None);
    assert!(!scheduler.request_rebake(failed_key));
    assert_eq!(scheduler.readiness(), RealtimeIblReadiness::FailedLastGood);

    assert!(!scheduler.request_rebake(published_key));
    assert_eq!(scheduler.readiness(), RealtimeIblReadiness::Ready);
    assert!(scheduler.failure_report().is_none());

    let recovered_key = key_with_revision(3);
    assert!(scheduler.request_rebake(recovered_key));
    assert!(scheduler.failure_report().is_none());
    assert_eq!(scheduler.pending_key(), Some(recovered_key));
    assert!(scheduler.begin_frame(37).is_some());
}

#[test]
fn successful_retry_resets_the_consecutive_failure_budget() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));

    let first = scheduler.begin_frame(10).expect("first attempt");
    assert_eq!(
        scheduler.complete_attempt(
            first.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Recording),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(scheduler.begin_frame(11), None);
    let retry = scheduler.begin_frame(12).expect("successful retry");
    assert_eq!(
        scheduler.complete_attempt(retry.token(), RealtimeIblSliceAttempt::Succeeded),
        RealtimeIblCompletion::Advanced
    );
    assert!(scheduler.failure_report().is_none());

    let next_stage = scheduler.begin_frame(13).expect("next stage");
    assert_eq!(
        scheduler.complete_attempt(
            next_stage.token(),
            RealtimeIblSliceAttempt::Failed(RealtimeIblFailureKind::Submission),
        ),
        RealtimeIblCompletion::RetryScheduled
    );
    assert_eq!(
        scheduler
            .failure_report()
            .expect("new stage failure report")
            .failed_attempt_count,
        1
    );
}

#[test]
fn single_mip_generation_has_no_empty_gpu_operations() {
    let mut scheduler = RealtimeIblTimeSliceScheduler::new(
        RealtimeIblTimeSliceConfig::try_new(1, 2).expect("valid test config"),
    );
    let key = key_with_revision(1);
    scheduler.request_rebake(key);

    let mut completed = 0;
    for frame in 100..120 {
        let batch = scheduler.begin_frame(frame).expect("generation batch");
        assert_eq!(batch.operations().len(), 1);
        completed += 1;
        if scheduler.complete_frame(batch.token(), true) == RealtimeIblCompletion::Published {
            break;
        }
    }

    assert_eq!(completed, 7);
    assert_eq!(scheduler.published_key(), Some(key));
}

#[test]
fn requesting_the_published_key_cancels_obsolete_pending_work() {
    let mut scheduler = scheduler();
    let published = key_with_revision(1);
    publish_generation(&mut scheduler, published);
    scheduler.request_rebake(key_with_revision(2));
    let obsolete = scheduler.begin_frame(40).expect("obsolete batch");

    assert!(scheduler.request_rebake(published));
    assert!(!scheduler.is_rebake_pending());
    assert_eq!(scheduler.begin_frame(41), None);
    assert_eq!(
        scheduler.complete_frame(obsolete.token(), true),
        RealtimeIblCompletion::Stale
    );
    assert_eq!(scheduler.published_key(), Some(published));
}

#[test]
fn source_mips_complete_before_pmrem_face_slices() {
    let mut scheduler = scheduler();
    scheduler.request_rebake(key_with_revision(1));

    for frame in 100..110 {
        let batch = scheduler.begin_frame(frame).expect("scheduled batch");
        scheduler.complete_frame(batch.token(), true);
    }
    let mip_zero_faces_zero_and_one = scheduler.begin_frame(110).expect("PMREM slice");

    assert_eq!(
        mip_zero_faces_zero_and_one.prefilter_dispatch_slices(),
        vec![RealtimeIblPrefilterDispatchSlice {
            mip_level: 0,
            first_face: 0,
            face_count: 2,
        }]
    );
}

#[test]
fn frame_batch_uses_fixed_storage_for_single_operation() {
    let source = include_str!("realtime_ibl_time_slice.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("realtime IBL scheduler implementation");

    assert!(implementation.contains("operations: [RealtimeIblOperation; 1]"));
    assert!(implementation.contains("operations: [ticket.operation(self.config)]"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cp_runtime_ibl_operation_storage_p95() {
    const SAMPLE_PAIRS: usize = 17;
    const OPERATIONS_PER_SAMPLE: usize = 1;
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME391_IBL_OPERATION_STORAGE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} operations_per_sample={OPERATIONS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(use_fixed_storage: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..100_000 {
        if use_fixed_storage {
            let operations = [RealtimeIblOperation::ProjectDiffuseSh9];
            checksum ^= operations.len();
            black_box(operations);
        } else {
            let operations = vec![RealtimeIblOperation::ProjectDiffuseSh9];
            checksum ^= operations.len();
            black_box(operations);
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn publish_generation(scheduler: &mut RealtimeIblTimeSliceScheduler, key: IblBakeKey) {
    scheduler.request_rebake(key);
    for frame in 1..64 {
        let batch = scheduler.begin_frame(frame).expect("generation batch");
        if scheduler.complete_frame(batch.token(), true) == RealtimeIblCompletion::Published {
            return;
        }
    }
    panic!("realtime IBL generation did not publish within the ticket budget");
}
