use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use super::retained_slot_id_index;

const PERFORMANCE_MARKER: &str = "RUNTIME138_CAPTURE_RETENTION_HASH_MEMBERSHIP_BENCH_V1";

#[test]
fn optimization_batch_20260826cu_runtime138_hash_index_preserves_membership() {
    let retained = vec![
        "slot-c".to_string(),
        "slot-a".to_string(),
        "slot-c".to_string(),
        "slot-b".to_string(),
    ];

    let index = retained_slot_id_index(&retained);

    assert_eq!(index.len(), 3);
    assert!(index.contains("slot-a"));
    assert!(index.contains("slot-b"));
    assert!(index.contains("slot-c"));
    assert!(!index.contains("slot-d"));
}

#[test]
fn optimization_batch_20260826cu_runtime138_hash_membership_keeps_canonical_output_order() {
    let retained = vec![
        "slot-c".to_string(),
        "slot-a".to_string(),
        "slot-b".to_string(),
    ];
    let index = retained_slot_id_index(&retained);
    let mut projected = ["slot-d", "slot-b", "slot-c", "slot-a"]
        .into_iter()
        .filter(|slot_id| index.contains(slot_id))
        .collect::<Vec<_>>();

    projected.sort_unstable();

    assert_eq!(projected, ["slot-a", "slot-b", "slot-c"]);
}

#[test]
#[ignore = "release-only capture retention membership performance gate"]
fn optimization_batch_20260826cu_runtime138_hash_membership_performance_evidence() {
    const SLOT_COUNT: usize = 8_192;
    const PROBE_ROUNDS: usize = 12;
    const SAMPLE_COUNT: usize = 15;

    assert_eq!(
        PERFORMANCE_MARKER,
        "RUNTIME138_CAPTURE_RETENTION_HASH_MEMBERSHIP_BENCH_V1"
    );
    let retained = (0..SLOT_COUNT)
        .map(|index| format!("capture-slot-{index:08}"))
        .collect::<Vec<_>>();
    let probes = (0..SLOT_COUNT)
        .map(|index| format!("capture-slot-{:08}", index * 2))
        .collect::<Vec<_>>();

    for _ in 0..3 {
        black_box(legacy_membership_work(&retained, &probes, PROBE_ROUNDS));
        black_box(hash_membership_work(&retained, &probes, PROBE_ROUNDS));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                black_box(legacy_membership_work(&retained, &probes, PROBE_ROUNDS));
            }));
            optimized_samples.push(measure(|| {
                black_box(hash_membership_work(&retained, &probes, PROBE_ROUNDS));
            }));
        } else {
            optimized_samples.push(measure(|| {
                black_box(hash_membership_work(&retained, &probes, PROBE_ROUNDS));
            }));
            legacy_samples.push(measure(|| {
                black_box(legacy_membership_work(&retained, &probes, PROBE_ROUNDS));
            }));
        }
    }

    let legacy_p50_ns = percentile_ns(&mut legacy_samples, 50);
    let legacy_p95_ns = percentile_ns(&mut legacy_samples, 95);
    let optimized_p50_ns = percentile_ns(&mut optimized_samples, 50);
    let optimized_p95_ns = percentile_ns(&mut optimized_samples, 95);
    println!(
        "{PERFORMANCE_MARKER} legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} slots={SLOT_COUNT} probes_per_round={SLOT_COUNT} probe_rounds={PROBE_ROUNDS} samples={SAMPLE_COUNT} tree_index_entries={SLOT_COUNT} hash_index_entries={SLOT_COUNT}"
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "hash membership P95 {optimized_p95_ns}ns must be at most 70% of tree membership P95 {legacy_p95_ns}ns"
    );
}

fn legacy_membership_work(retained: &[String], probes: &[String], rounds: usize) -> usize {
    let index = retained.iter().map(String::as_str).collect::<BTreeSet<_>>();
    (0..rounds)
        .map(|_| {
            probes
                .iter()
                .filter(|slot_id| index.contains(slot_id.as_str()))
                .count()
        })
        .sum()
}

fn hash_membership_work(retained: &[String], probes: &[String], rounds: usize) -> usize {
    let index = retained_slot_id_index(retained);
    (0..rounds)
        .map(|_| {
            probes
                .iter()
                .filter(|slot_id| index.contains(slot_id.as_str()))
                .count()
        })
        .sum()
}

fn measure(run: impl FnOnce()) -> Duration {
    let started = Instant::now();
    run();
    started.elapsed()
}

fn percentile_ns(samples: &mut [Duration], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)].as_nanos()
}
