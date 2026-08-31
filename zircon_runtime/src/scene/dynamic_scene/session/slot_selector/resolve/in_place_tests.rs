use std::hint::black_box;
use std::time::Instant;

use super::RuntimeSessionSlotSelector;

const SAMPLE_PAIRS: usize = 21;
const SELECTORS_PER_SAMPLE: usize = 16_384;

#[test]
fn optimization_batch_20260826df_runtime149_slot_selectors_preserve_trimmed_values() {
    assert!(matches!(
        RuntimeSessionSlotSelector::slot_id("  checkpoint-7  "),
        RuntimeSessionSlotSelector::SlotId { slot_id } if slot_id == "checkpoint-7"
    ));
    assert!(matches!(
        RuntimeSessionSlotSelector::latest_updated_with_tag(" \tcombat\n"),
        RuntimeSessionSlotSelector::LatestUpdatedWithTag { tag } if tag == "combat"
    ));
    assert!(matches!(
        RuntimeSessionSlotSelector::oldest_updated_with_tag("  世界  "),
        RuntimeSessionSlotSelector::OldestUpdatedWithTag { tag } if tag == "世界"
    ));
}

#[test]
fn optimization_batch_20260826df_runtime149_slot_selector_reuses_owned_buffer() {
    let mut slot_id = String::with_capacity(128);
    slot_id.push_str("  retained-checkpoint  ");
    let allocation = slot_id.as_ptr();
    let capacity = slot_id.capacity();

    let RuntimeSessionSlotSelector::SlotId { slot_id } =
        RuntimeSessionSlotSelector::slot_id(slot_id)
    else {
        panic!("slot selector constructor must preserve its variant");
    };

    assert_eq!(slot_id, "retained-checkpoint");
    assert_eq!(slot_id.as_ptr(), allocation);
    assert_eq!(slot_id.capacity(), capacity);

    let source = include_str!("../resolve.rs");
    assert_eq!(source.matches("normalize_selector_value(").count(), 4);
    assert!(!source.contains("into().trim().to_string()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826df_runtime149_slot_selector_in_place_normalization_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy = fixture_slot_ids();
        let optimized = legacy.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy, legacy_slot_selector));
            optimized_samples.push(measure(optimized, RuntimeSessionSlotSelector::slot_id));
        } else {
            optimized_samples.push(measure(optimized, RuntimeSessionSlotSelector::slot_id));
            legacy_samples.push(measure(legacy, legacy_slot_selector));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME149_SLOT_SELECTOR_IN_PLACE_NORMALIZATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
selectors_per_sample={SELECTORS_PER_SAMPLE} \
legacy_trim_allocations_per_sample={SELECTORS_PER_SAMPLE} optimized_trim_allocations_per_sample=0 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place selector normalization P95 {optimized_p95_ns}ns must be at most 70% of copied normalization P95 {legacy_p95_ns}ns"
    );
}

fn fixture_slot_ids() -> Vec<String> {
    (0..SELECTORS_PER_SAMPLE)
        .map(|index| format!("  checkpoint-{index:05}  "))
        .collect()
}

fn legacy_slot_selector(slot_id: String) -> RuntimeSessionSlotSelector {
    RuntimeSessionSlotSelector::SlotId {
        slot_id: slot_id.trim().to_string(),
    }
}

fn measure(slot_ids: Vec<String>, construct: fn(String) -> RuntimeSessionSlotSelector) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for slot_id in slot_ids {
        let RuntimeSessionSlotSelector::SlotId { slot_id } =
            black_box(construct(black_box(slot_id)))
        else {
            unreachable!();
        };
        checksum ^= slot_id.len();
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
