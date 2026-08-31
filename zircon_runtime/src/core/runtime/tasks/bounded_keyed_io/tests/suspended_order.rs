use std::collections::VecDeque;
use std::hint::black_box;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::super::lane::merge_ordered;
use super::{blocked_scheduler, BoundedKeyedIoLane, BoundedKeyedIoLimits};
use crate::core::runtime::tasks::bounded_keyed_io::{
    BoundedKeyedIoTerminal, BoundedKeyedIoWaitResult, BoundedKeyedIoWork,
    BoundedKeyedIoWorkDeadline, GlobalAdmissionEpoch,
};

#[test]
fn optimization_wave_20260825wx_runtime134_suspended_order_tracks_activation_and_release() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 8), scheduler);
    let front = lane
        .try_admit(
            "front",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap()
        .activate();
    let mut suspended = (0..3)
        .map(|generation| {
            lane.try_admit(
                "held",
                generation,
                1,
                BoundedKeyedIoWorkDeadline::none(),
                Box::new(|| Ok(())),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    assert!(lane.suspended_order_index_matches_for_tests());
    assert!(lane.front_is_runnable_for_tests());

    let activated = suspended.pop().unwrap().activate();
    assert!(lane.suspended_order_index_matches_for_tests());
    assert!(lane.front_is_runnable_for_tests());

    suspended.clear();
    assert!(lane.suspended_order_index_matches_for_tests());
    assert!(lane.front_is_runnable_for_tests());

    release_tx.send(()).unwrap();
    assert!(matches!(
        front.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    assert!(matches!(
        activated.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    blocker.wait();
}

#[test]
fn optimization_wave_20260825wx_runtime134_suspended_minimum_preserves_ticket_ordering() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(4, 4), scheduler);
    let earlier = lane
        .try_admit(
            "earlier",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap();
    let later = lane
        .try_admit(
            "later",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap()
        .activate();

    assert!(lane.suspended_order_index_matches_for_tests());
    assert!(!lane.front_is_runnable_for_tests());

    drop(earlier);
    assert!(lane.suspended_order_index_matches_for_tests());
    assert!(lane.front_is_runnable_for_tests());

    release_tx.send(()).unwrap();
    assert!(matches!(
        later.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    blocker.wait();
}

#[test]
fn optimization_wave_20260825wx_runtime134_shutdown_merges_pinned_admissions_in_order() {
    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(BoundedKeyedIoLimits::new(8, 8), scheduler);
    let calls = Arc::new(Mutex::new(Vec::new()));
    let work = |value| -> BoundedKeyedIoWork {
        let calls = Arc::clone(&calls);
        Box::new(move || {
            calls.lock().unwrap().push(value);
            Ok(())
        })
    };

    let pinned_first = lane
        .try_admit(
            "pinned-first",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            work(1),
        )
        .unwrap();
    let queued_first = lane
        .try_admit(
            "queued-first",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            work(2),
        )
        .unwrap()
        .activate();
    let first_fence = lane
        .submit_fence(1, BoundedKeyedIoWorkDeadline::none(), work(10))
        .unwrap();
    let pinned_second = lane
        .try_admit(
            "pinned-second",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            work(3),
        )
        .unwrap();
    let queued_second = lane
        .try_admit(
            "queued-second",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            work(4),
        )
        .unwrap()
        .activate();
    let second_fence = lane
        .submit_fence(1, BoundedKeyedIoWorkDeadline::none(), work(20))
        .unwrap();

    let guard = lane.shutdown();
    assert!(lane.suspended_order_index_matches_for_tests());
    release_tx.send(()).unwrap();
    assert!(guard.wait_until(Instant::now() + Duration::from_secs(2)));
    blocker.wait();
    assert_eq!(*calls.lock().unwrap(), vec![1, 2, 10, 3, 4, 20]);
    assert_eq!(
        pinned_first.ticket().terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(
        pinned_second.ticket().terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(
        queued_first.terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(
        queued_second.terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(
        first_fence.ticket().terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
    assert_eq!(
        second_fence.ticket().terminal(),
        Some(BoundedKeyedIoTerminal::Succeeded)
    );
}

#[test]
fn optimization_wave_20260825wx_runtime134_front_readiness_uses_ordered_minimum() {
    let source = include_str!("../lane.rs");
    let function = function_source(source, "fn front_is_runnable(state: &LaneState)");
    assert!(function.contains("suspended_order.first().copied()"));
    assert!(!function.contains("suspended.iter().any"));
    assert_eq!(source.matches("state.suspended.remove(").count(), 1);
    assert!(source.contains("fn remove_suspended_entry"));
    let shutdown = source
        .split("pub fn shutdown(&self)")
        .nth(1)
        .and_then(|source| source.split("\n    }\n}\n\nimpl LaneInner").next())
        .unwrap();
    assert!(shutdown.contains("merge_ordered_queue"));
    assert!(!shutdown.contains("insert_ordered"));
}

#[test]
#[ignore = "release-only Runtime134 suspended-admission readiness performance gate"]
fn optimization_wave_20260825wx_runtime134_suspended_order_release_benchmark() {
    const SUSPENDED_COUNT: usize = 50_000;
    const PROBES_PER_SAMPLE: usize = 32;
    const SAMPLE_PAIRS: usize = 15;

    let (scheduler, release_tx, blocker) = blocked_scheduler();
    let lane = BoundedKeyedIoLane::new(
        BoundedKeyedIoLimits::new(SUSPENDED_COUNT + 1, SUSPENDED_COUNT + 1),
        scheduler,
    );
    let front = lane
        .try_admit(
            "front",
            1,
            1,
            BoundedKeyedIoWorkDeadline::none(),
            Box::new(|| Ok(())),
        )
        .unwrap()
        .activate();
    let suspended = (0..SUSPENDED_COUNT)
        .map(|generation| {
            lane.try_admit(
                format!("held-{generation}"),
                generation as u64,
                1,
                BoundedKeyedIoWorkDeadline::none(),
                Box::new(|| Ok(())),
            )
            .unwrap()
        })
        .collect::<Vec<_>>();
    let snapshot = lane.front_readiness_snapshot_for_tests().unwrap();
    assert_eq!(snapshot.3.len(), SUSPENDED_COUNT);
    assert_eq!(
        legacy_front_is_runnable(&snapshot),
        lane.front_is_runnable_for_tests()
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            measure(PROBES_PER_SAMPLE, || {
                black_box(legacy_front_is_runnable(black_box(&snapshot)))
            })
        };
        let measure_optimized = || {
            measure(PROBES_PER_SAMPLE, || {
                black_box(lane.front_is_runnable_for_tests())
            })
        };
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME134_SUSPENDED_ADMISSION_ORDER_INDEX_BENCH_V1 suspended_count={SUSPENDED_COUNT} probes_per_sample={PROBES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_visits_per_sample={} optimized_min_lookups_per_sample={PROBES_PER_SAMPLE} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SUSPENDED_COUNT * PROBES_PER_SAMPLE,
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        nanos(&legacy_samples),
        nanos(&optimized_samples),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(10) <= legacy_p95.as_nanos(),
        "optimized P95 {optimized_p95:?} must be at most 10% of legacy P95 {legacy_p95:?}"
    );
    assert!(
        optimized_p95 <= Duration::from_millis(1),
        "optimized P95 {optimized_p95:?} must stay within 1 ms"
    );

    drop(suspended);
    release_tx.send(()).unwrap();
    assert!(matches!(
        front.wait_until(Instant::now() + Duration::from_secs(2)),
        BoundedKeyedIoWaitResult::Terminal(BoundedKeyedIoTerminal::Succeeded)
    ));
    blocker.wait();
}

#[test]
#[ignore = "release-only Runtime134 suspended shutdown merge performance gate"]
fn optimization_wave_20260825wx_runtime134_shutdown_merge_release_benchmark() {
    const QUEUED_COUNT: usize = 8_192;
    const SUSPENDED_COUNT: usize = 8_192;
    const SAMPLE_PAIRS: usize = 15;

    let queued = (0..QUEUED_COUNT)
        .map(|index| (0_u64, (index as u64 + 1) * 2, false))
        .collect::<VecDeque<_>>();
    let suspended = (0..SUSPENDED_COUNT)
        .map(|index| (0_u64, index as u64 * 2 + 1, false))
        .collect::<VecDeque<_>>();
    let mut legacy_expected = queued.clone();
    for entry in suspended.iter().copied() {
        legacy_insert_ordered(&mut legacy_expected, entry);
    }
    let mut optimized_actual = queued.clone();
    merge_ordered(
        &mut optimized_actual,
        suspended.clone(),
        synthetic_entry_precedes_or_equals,
    );
    assert_eq!(optimized_actual, legacy_expected);

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_shutdown_merge_legacy(&queued, &suspended));
            optimized_samples.push(measure_shutdown_merge_optimized(&queued, &suspended));
        } else {
            optimized_samples.push(measure_shutdown_merge_optimized(&queued, &suspended));
            legacy_samples.push(measure_shutdown_merge_legacy(&queued, &suspended));
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let optimized_p50 = nearest_rank(&optimized_samples, 50);
    let optimized_p95 = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME134_SUSPENDED_SHUTDOWN_MERGE_BENCH_V1 queued_count={QUEUED_COUNT} suspended_count={SUSPENDED_COUNT} sample_pairs={SAMPLE_PAIRS} legacy_order_comparisons={} optimized_order_comparisons_upper={} optimized_linear_visits={} legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        SUSPENDED_COUNT.saturating_mul(SUSPENDED_COUNT),
        QUEUED_COUNT + SUSPENDED_COUNT - 1,
        QUEUED_COUNT + SUSPENDED_COUNT,
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
        nanos(&legacy_samples),
        nanos(&optimized_samples),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(10) <= legacy_p95.as_nanos(),
        "optimized P95 {optimized_p95:?} must be at most 10% of legacy P95 {legacy_p95:?}"
    );
    assert!(
        optimized_p95 <= Duration::from_millis(5),
        "optimized P95 {optimized_p95:?} must stay within 5 ms"
    );
}

type ReadinessSnapshot = (
    GlobalAdmissionEpoch,
    u64,
    bool,
    Vec<(GlobalAdmissionEpoch, u64)>,
);

fn legacy_front_is_runnable(snapshot: &ReadinessSnapshot) -> bool {
    let (front_epoch, front_ticket_id, front_is_fence, suspended) = snapshot;
    !suspended.iter().any(|(epoch, ticket_id)| {
        epoch < front_epoch
            || (epoch == front_epoch && (*front_is_fence || ticket_id < front_ticket_id))
    })
}

type SyntheticEntry = (u64, u64, bool);

fn synthetic_entry_precedes_or_equals(left: &SyntheticEntry, right: &SyntheticEntry) -> bool {
    left.0 < right.0
        || (left.0 == right.0 && ((!left.2 && right.2) || (left.2 == right.2 && left.1 <= right.1)))
}

fn legacy_insert_ordered(queue: &mut VecDeque<SyntheticEntry>, entry: SyntheticEntry) {
    let insertion = queue
        .iter()
        .position(|queued| !synthetic_entry_precedes_or_equals(queued, &entry))
        .unwrap_or(queue.len());
    queue.insert(insertion, entry);
}

fn measure_shutdown_merge_legacy(
    queued: &VecDeque<SyntheticEntry>,
    suspended: &VecDeque<SyntheticEntry>,
) -> Duration {
    let mut merged = queued.clone();
    let incoming = suspended.clone();
    let started = Instant::now();
    for entry in incoming {
        legacy_insert_ordered(black_box(&mut merged), black_box(entry));
    }
    black_box(merged);
    started.elapsed()
}

fn measure_shutdown_merge_optimized(
    queued: &VecDeque<SyntheticEntry>,
    suspended: &VecDeque<SyntheticEntry>,
) -> Duration {
    let mut merged = queued.clone();
    let incoming = suspended.clone();
    let started = Instant::now();
    merge_ordered(
        black_box(&mut merged),
        black_box(incoming),
        synthetic_entry_precedes_or_equals,
    );
    black_box(merged);
    started.elapsed()
}

fn measure(mut probes: usize, mut probe: impl FnMut() -> bool) -> Duration {
    let started = Instant::now();
    while probes > 0 {
        assert!(probe());
        probes -= 1;
    }
    started.elapsed()
}

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = percentile.saturating_mul(sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn nanos(samples: &[Duration]) -> Vec<u128> {
    samples.iter().map(Duration::as_nanos).collect()
}

fn function_source<'a>(source: &'a str, signature: &str) -> &'a str {
    let start = source.find(signature).unwrap();
    let rest = &source[start..];
    let end = rest.find("\n}\n").unwrap() + 3;
    &rest[..end]
}
