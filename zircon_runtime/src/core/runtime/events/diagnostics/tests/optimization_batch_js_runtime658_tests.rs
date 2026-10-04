use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crate::core::framework::events::EventBusDiagnosticsMode;

use super::{decrement_saturating, decrement_saturating_by, EventBusDiagnosticsState};

const SAMPLE_PAIRS: usize = 17;
const BATCHES_PER_SAMPLE: usize = 2_048;
const DRAINED_EVENTS_PER_BATCH: u64 = 1_024;

#[test]
fn optimization_batch_js_runtime658_event_drain_batches_depth_and_age_diagnostics() {
    let diagnostics = EventBusDiagnosticsState::new(EventBusDiagnosticsMode::Enabled);
    for _ in 0..4 {
        diagnostics.record_enqueued_and_capture_time();
    }
    let old = Instant::now()
        .checked_sub(Duration::from_millis(10))
        .expect("short diagnostic age should fit");

    diagnostics.record_drained([Some(old), None, Some(old), Some(old)]);

    let snapshot = diagnostics.snapshot(1, 1);
    assert_eq!(snapshot.queued, 0);
    assert_eq!(snapshot.queue_age_samples, 3);
    assert!(snapshot.total_queue_age_ms >= 30.0);
    assert!(snapshot.max_queue_age_ms >= 10.0);
}

#[test]
fn optimization_batch_js_runtime658_subscription_drain_uses_one_diagnostic_batch() {
    let source = include_str!("../../subscriber.rs");
    let drain = source
        .split("pub(super) fn deactivate_and_drain")
        .nth(1)
        .and_then(|source| source.split("fn receive(").next())
        .expect("subscriber drain implementation");

    // BUG: [CR-R02-core_runtime_state-0001] 此断言截取 deactivate_and_drain 之后的源码，漏掉前置委托目标 record_deactivated_queue，所以当前切片不含 record_drained，结构守卫会错误失败。
    assert_eq!(drain.matches("record_drained(").count(), 1);
    assert!(!drain.contains("for queued in queued"));
    assert!(!drain.contains("record_dequeued(queued.queued_at)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_js_runtime658_batched_event_drain_diagnostics_bench() {
    for _ in 0..4 {
        black_box(measure(false));
        black_box(measure(true));
    }
    let mut per_event_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut batched_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            per_event_samples.push(measure(false));
            batched_samples.push(measure(true));
        } else {
            batched_samples.push(measure(true));
            per_event_samples.push(measure(false));
        }
    }

    let per_event_p95_ns = percentile(&per_event_samples, 95);
    let batched_p95_ns = percentile(&batched_samples, 95);
    println!(
        "RUNTIME658_BATCHED_EVENT_DRAIN_DIAGNOSTICS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} drained_events_per_batch={DRAINED_EVENTS_PER_BATCH} \
per_event_p95_ns={per_event_p95_ns} batched_p95_ns={batched_p95_ns} \
per_event_raw_ns={} batched_raw_ns={}",
        sample_csv(&per_event_samples),
        sample_csv(&batched_samples),
    );

    assert!(batched_p95_ns.saturating_mul(100) <= per_event_p95_ns.saturating_mul(20));
}

fn measure(batched: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..BATCHES_PER_SAMPLE {
        let queued = AtomicU64::new(DRAINED_EVENTS_PER_BATCH);
        if batched {
            decrement_saturating_by(&queued, DRAINED_EVENTS_PER_BATCH);
        } else {
            for _ in 0..DRAINED_EVENTS_PER_BATCH {
                decrement_saturating(&queued);
            }
        }
        checksum ^= queued.load(Ordering::Relaxed);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
