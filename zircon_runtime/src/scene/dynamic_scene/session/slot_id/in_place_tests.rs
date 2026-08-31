use std::hint::black_box;
use std::time::Instant;

use super::{normalize_slot_id, trim_slot_id_in_place};
use crate::scene::dynamic_scene::session::RuntimeSessionArchiveError;

const SAMPLE_PAIRS: usize = 21;
const SLOT_IDS_PER_SAMPLE: usize = 16_384;

#[test]
fn optimization_batch_20260826de_runtime148_slot_id_preserves_trim_and_validation() {
    assert_eq!(
        normalize_slot_id(" \tcheckpoint-42\n ".to_string()).unwrap(),
        "checkpoint-42"
    );
    assert!(matches!(
        normalize_slot_id(" \t\n ".to_string()),
        Err(RuntimeSessionArchiveError::EmptySlotId)
    ));
}

#[test]
fn optimization_batch_20260826de_runtime148_slot_id_reuses_owned_buffer() {
    let mut slot_id = String::with_capacity(128);
    slot_id.push_str("  imported-checkpoint  ");
    let allocation = slot_id.as_ptr();
    let capacity = slot_id.capacity();

    trim_slot_id_in_place(&mut slot_id);

    assert_eq!(slot_id, "imported-checkpoint");
    assert_eq!(slot_id.as_ptr(), allocation);
    assert_eq!(slot_id.capacity(), capacity);

    let source = include_str!("../slot_id.rs");
    assert!(source.contains("trim_slot_id_in_place(&mut slot_id)"));
    assert!(!source.contains("slot_id.trim().to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826de_runtime148_slot_id_in_place_normalization_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy = fixture_slot_ids();
        let optimized = legacy.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy, legacy_normalize_slot_id));
            optimized_samples.push(measure(optimized, normalize_slot_id));
        } else {
            optimized_samples.push(measure(optimized, normalize_slot_id));
            legacy_samples.push(measure(legacy, legacy_normalize_slot_id));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME148_SLOT_ID_IN_PLACE_NORMALIZATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
slot_ids_per_sample={SLOT_IDS_PER_SAMPLE} \
legacy_trim_allocations_per_sample={SLOT_IDS_PER_SAMPLE} optimized_trim_allocations_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place slot normalization P95 {optimized_p95_ns}ns must be at most 70% of copied normalization P95 {legacy_p95_ns}ns"
    );
}

fn fixture_slot_ids() -> Vec<String> {
    (0..SLOT_IDS_PER_SAMPLE)
        .map(|index| format!("  checkpoint-{index:05}  "))
        .collect()
}

fn legacy_normalize_slot_id(slot_id: String) -> Result<String, RuntimeSessionArchiveError> {
    let slot_id = slot_id.trim().to_string();
    if slot_id.trim().is_empty() {
        return Err(RuntimeSessionArchiveError::EmptySlotId);
    }
    Ok(slot_id)
}

fn measure(
    slot_ids: Vec<String>,
    normalize: fn(String) -> Result<String, RuntimeSessionArchiveError>,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for slot_id in slot_ids {
        checksum ^= black_box(normalize(black_box(slot_id)).unwrap()).len();
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
