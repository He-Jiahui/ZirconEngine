use std::hint::black_box;
use std::time::Instant;

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828ic_editor_round_robin_cursor_reuses_allocation() {
    let mut id = String::with_capacity(16 * 1024);
    id.push_str(&"consumer".repeat(512));
    let next = id.clone();
    let allocation = id.as_ptr();
    let mut cursor = Some(id);

    update_round_robin_cursor(&mut cursor, &next);
    assert_eq!(cursor.as_deref(), Some(next.as_str()));
    assert_eq!(cursor.as_ref().expect("cursor").as_ptr(), allocation);

    update_round_robin_cursor(&mut cursor, "next-consumer");
    assert_eq!(cursor.as_deref(), Some("next-consumer"));
    assert_eq!(cursor.as_ref().expect("cursor").as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260828ic_editor_round_robin_borrows_and_updates_cursor_in_place() {
    let source = include_str!("../round_robin.rs");
    let advance = source
        .split("pub(super) fn advance_round_robin_start")
        .nth(1)
        .and_then(|body| body.split("fn update_round_robin_cursor").next())
        .expect("round-robin advance implementation");
    let update = source
        .split("fn update_round_robin_cursor")
        .nth(1)
        .and_then(|body| body.split("fn next_start_index").next())
        .expect("round-robin cursor update implementation");

    assert!(advance.contains("snapshot.consumer_id.as_str()"));
    assert!(advance.contains("update_round_robin_cursor"));
    assert!(!advance.contains("snapshot.consumer_id.clone()"));
    assert!(update.contains("Some(current) if current.as_str() == next"));
    assert!(update.contains("current.clear()"));
    assert!(update.contains("current.push_str(next)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828ic_editor_reused_round_robin_cursor_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8 * 1024;
    let consumer_id = "round-robin-consumer/".repeat(256);

    let mut warm_cursor = Some(consumer_id.clone());
    legacy_update_round_robin_cursor(&mut warm_cursor, &consumer_id);
    update_round_robin_cursor(&mut warm_cursor, &consumer_id);
    black_box(warm_cursor);

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let mut cursor = Some(consumer_id.clone());
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                legacy_update_round_robin_cursor(black_box(&mut cursor), black_box(&consumer_id));
            }
            black_box(cursor);
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let mut cursor = Some(consumer_id.clone());
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                update_round_robin_cursor(black_box(&mut cursor), black_box(&consumer_id));
            }
            black_box(cursor);
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "EDITOR221_REUSED_ROUND_ROBIN_CURSOR_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_update_round_robin_cursor(cursor: &mut Option<String>, next: &str) {
    *cursor = Some(next.to_owned());
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
