use std::hint::black_box;
use std::time::Instant;

use super::chip_slot_variant;

const CHECKS_PER_SAMPLE: usize = 8192;
const SAMPLE_PAIRS: usize = 31;
const VARIANT_BYTES: usize = 4096;

fn split_variant(value: &str) -> impl Iterator<Item = &str> {
    value.split(|character: char| {
        character.is_ascii_whitespace() || matches!(character, ',' | '/' | '|' | ':' | ';')
    })
}

fn legacy_contains(value: &str, expected: &str) -> bool {
    split_variant(value).any(|part| part.eq_ignore_ascii_case(expected))
}

fn legacy_starts_with(value: &str, expected_prefix: &str) -> bool {
    split_variant(value).any(|part| {
        part.get(..expected_prefix.len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(expected_prefix))
    })
}

fn legacy_chip_slot_variant(value: &str) -> bool {
    legacy_contains(value, "muiChipSlot")
        || legacy_contains(value, "ChipSlot")
        || legacy_contains(value, "chipSlot")
        || legacy_starts_with(value, "chipSlot")
}

fn measure(value: &str, optimized: bool) -> u128 {
    let started = Instant::now();
    let mut matches = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        matches += usize::from(if optimized {
            chip_slot_variant(black_box(value))
        } else {
            legacy_chip_slot_variant(black_box(value))
        });
    }
    black_box(matches);
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

#[test]
fn optimization_batch_20260829bk_editor283_single_scan_chip_slots_preserve_results() {
    for value in [
        "muiChipSlot",
        "CHIPSLOT",
        "chipSlotIcon",
        "filled,chipSlotAvatar",
        "filled / MuiChipSlot",
        "preChipSlot",
        "chip",
        "",
        "\u{4f8b}",
    ] {
        assert_eq!(
            chip_slot_variant(value),
            legacy_chip_slot_variant(value),
            "{value:?}"
        );
    }
}

#[test]
fn optimization_batch_20260829bk_editor283_chip_slot_uses_one_token_scan() {
    let source = include_str!("../identity.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    let slot = production
        .split_once("fn is_chip_slot_node")
        .expect("slot function")
        .1
        .split_once("fn chip_is_small")
        .expect("next function")
        .0;

    assert!(slot.contains("chip_slot_variant(&node.component_variant)"));
    assert!(!slot.contains("component_variant_contains"));
    assert_eq!(production.matches("fn chip_slot_variant").count(), 1);
    assert!(!production.contains("fn component_variant_token_starts_with"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bk_editor283_single_scan_chip_slot_bench() {
    let value = "x".repeat(VARIANT_BYTES);
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&value, false));
            candidate_samples.push(measure(&value, true));
        } else {
            candidate_samples.push(measure(&value, true));
            baseline_samples.push(measure(&value, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "EDITOR283_SINGLE_SCAN_CHIP_SLOT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} variant_bytes={VARIANT_BYTES} \
baseline_variant_scans=4 candidate_variant_scans=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
