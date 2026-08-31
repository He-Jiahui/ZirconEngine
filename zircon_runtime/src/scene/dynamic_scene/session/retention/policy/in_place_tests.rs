use std::hint::black_box;
use std::time::Instant;

use super::{normalize_protected_slot_ids, trim_slot_id_in_place};

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 64;
const SLOT_IDS_PER_BATCH: usize = 256;

#[test]
fn optimization_batch_20260826dd_runtime147_retention_slots_preserve_canonical_order() {
    let mut slot_ids = vec![
        "  autosave-2  ".to_string(),
        "\tautosave-1\n".to_string(),
        "autosave-2".to_string(),
        "   ".to_string(),
        "存档 ".to_string(),
    ];

    normalize_protected_slot_ids(&mut slot_ids);

    assert_eq!(slot_ids, ["autosave-1", "autosave-2", "存档"]);
}

#[test]
fn optimization_batch_20260826dd_runtime147_retention_slot_trim_reuses_owned_buffer() {
    let mut slot_id = String::with_capacity(128);
    slot_id.push_str("  protected-checkpoint  ");
    let allocation = slot_id.as_ptr();
    let capacity = slot_id.capacity();

    trim_slot_id_in_place(&mut slot_id);

    assert_eq!(slot_id, "protected-checkpoint");
    assert_eq!(slot_id.as_ptr(), allocation);
    assert_eq!(slot_id.capacity(), capacity);

    let source = include_str!("../policy.rs");
    assert!(source.contains("trim_slot_id_in_place(slot_id)"));
    assert!(!source.contains("*slot_id = slot_id.trim().to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dd_runtime147_retention_slot_in_place_trim_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy = fixture_batches();
        let optimized = legacy.clone();
        let measure_legacy = || measure(legacy, legacy_normalize_protected_slot_ids);
        let measure_optimized = || measure(optimized, normalize_protected_slot_ids);
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME147_RETENTION_SLOT_IN_PLACE_TRIM_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} slot_ids_per_batch={SLOT_IDS_PER_BATCH} \
legacy_trim_allocations_per_sample={} optimized_trim_allocations_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BATCHES_PER_SAMPLE * SLOT_IDS_PER_BATCH,
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place protected-slot trim P95 {optimized_p95_ns}ns must be at most 70% of copied trim P95 {legacy_p95_ns}ns"
    );
}

fn fixture_batches() -> Vec<Vec<String>> {
    (0..BATCHES_PER_SAMPLE)
        .map(|batch| {
            (0..SLOT_IDS_PER_BATCH)
                .map(|slot| format!("  checkpoint-{:03}-{:03}  ", batch % 8, slot % 64))
                .collect()
        })
        .collect()
}

fn legacy_normalize_protected_slot_ids(slot_ids: &mut Vec<String>) {
    for slot_id in slot_ids.iter_mut() {
        *slot_id = slot_id.trim().to_string();
    }
    slot_ids.retain(|slot_id| !slot_id.is_empty());
    slot_ids.sort();
    slot_ids.dedup();
}

fn measure(mut batches: Vec<Vec<String>>, normalize: fn(&mut Vec<String>)) -> u128 {
    let started = Instant::now();
    for slot_ids in &mut batches {
        normalize(black_box(slot_ids));
    }
    black_box(batches);
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
