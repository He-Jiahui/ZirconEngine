use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

use super::dedupe_suggestions;

const INPUT_COUNT: usize = 65_536;
const UNIQUE_COUNT: usize = 32_768;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iv_editor632_preserves_first_suggestion_order() {
    assert_eq!(
        dedupe_suggestions(vec![
            "beta".to_string(),
            "alpha".to_string(),
            "beta".to_string(),
            "gamma".to_string(),
            "alpha".to_string(),
        ]),
        vec!["beta".to_string(), "alpha".to_string(), "gamma".to_string(),]
    );
}

#[test]
fn optimization_batch_iv_editor632_hashes_suggestion_membership() {
    let source = include_str!("../../payload_editing.rs");
    let dedupe_body = source
        .split("pub(super) fn dedupe_suggestions")
        .nth(1)
        .expect("suggestion dedupe remains present")
        .split("pub(super) fn event_source_tag")
        .next()
        .expect("suggestion dedupe remains bounded");

    assert!(dedupe_body.contains("HashSet::with_capacity(items.len())"));
    assert!(!dedupe_body.contains("BTreeSet::new()"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iv_editor632_hash_suggestion_dedupe_benchmark() {
    let items = (0..INPUT_COUNT)
        .map(|index| format!("binding.suggestion.{:08}", index % UNIQUE_COUNT))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_dedupe(&items, false));
        black_box(measure_dedupe(&items, true));
    }

    let mut ordered_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut hash_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            ordered_samples.push(measure_dedupe(&items, false));
            hash_samples.push(measure_dedupe(&items, true));
        } else {
            hash_samples.push(measure_dedupe(&items, true));
            ordered_samples.push(measure_dedupe(&items, false));
        }
    }

    let ordered_p95 = percentile(&ordered_samples, 95);
    let hash_p95 = percentile(&hash_samples, 95);
    let improvement_percent =
        ordered_p95.saturating_sub(hash_p95).saturating_mul(100) / ordered_p95.max(1);
    println!(
        "EDITOR632_HASH_BINDING_SUGGESTION_DEDUPE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} input_count={INPUT_COUNT} unique_count={UNIQUE_COUNT} ordered_ns={} hash_ns={} ordered_p95_ns={ordered_p95} hash_p95_ns={hash_p95} improvement_percent={improvement_percent} threshold_percent=60",
        csv(&ordered_samples),
        csv(&hash_samples),
    );
    assert!(hash_p95 <= ordered_p95 * 40 / 100);
}

fn measure_dedupe(items: &[String], hash: bool) -> u128 {
    let started = Instant::now();
    let output = if hash {
        let mut seen = HashSet::with_capacity(items.len());
        items
            .iter()
            .filter(|item| seen.insert((*item).clone()))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        let mut seen = BTreeSet::new();
        items
            .iter()
            .filter(|item| seen.insert((*item).clone()))
            .cloned()
            .collect::<Vec<_>>()
    };
    black_box(output);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
