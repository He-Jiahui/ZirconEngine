use std::hint::black_box;
use std::time::Instant;

use crate::asset::{AssetReference, AssetUri};

use super::collect_unique_references;

const BENCH_REFERENCE_COUNT: usize = 100_000;
const UNIQUE_REFERENCE_COUNT: usize = 64;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime04_model_reference_index_preserves_first_seen_full_reference_order() {
    let alpha = reference(0);
    let beta = reference(1);
    let gamma = reference(2);
    let references = [
        alpha.clone(),
        beta.clone(),
        alpha.clone(),
        gamma.clone(),
        beta,
    ];

    let unique = collect_unique_references(references.iter());

    assert_eq!(unique, [alpha, reference(1), gamma]);
}

#[test]
#[ignore = "release-only model dependency dedup benchmark"]
fn runtime04_model_reference_index_release_benchmark_evidence() {
    let unique = (0..UNIQUE_REFERENCE_COUNT)
        .map(reference)
        .collect::<Vec<_>>();
    let references = (0..BENCH_REFERENCE_COUNT)
        .map(|index| unique[index % unique.len()].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_unique(&references),
        collect_unique_references(references.iter())
    );

    let (legacy_samples, indexed_samples) = paired_samples(
        || measure_legacy(&references),
        || measure_indexed(&references),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let indexed_p50_ns = percentile(&indexed_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let indexed_p95_ns = percentile(&indexed_samples, 95);

    println!(
        "PERF_RESULT plan=Runtime04 task=model_direct_reference_index \
sample_pairs={SAMPLE_PAIRS} reference_count={BENCH_REFERENCE_COUNT} unique_references={UNIQUE_REFERENCE_COUNT} \
legacy_dedup=ordered_vec_contains optimized_dedup=borrowed_hash_set_first_seen \
pair_order=alternating_legacy_even legacy_first_pairs=11 indexed_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} indexed_p50_ns={indexed_p50_ns} \
legacy_p95_ns={legacy_p95_ns} indexed_p95_ns={indexed_p95_ns} \
legacy_raw_ns={} indexed_raw_ns={}",
        raw(&legacy_samples),
        raw(&indexed_samples),
    );

    assert!(
        indexed_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "indexed model reference dedup must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns indexed={indexed_p95_ns}ns"
    );
}

fn reference(index: usize) -> AssetReference {
    AssetReference::from_locator(
        AssetUri::parse(&format!("res://models/mesh_{index:04}.zmesh"))
            .expect("benchmark asset URI should be valid"),
    )
}

fn legacy_unique(references: &[AssetReference]) -> Vec<AssetReference> {
    let mut unique = Vec::new();
    for reference in references {
        if !unique.contains(reference) {
            unique.push(reference.clone());
        }
    }
    unique
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_indexed: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_indexed());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut indexed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            indexed_samples.push(measure_indexed());
        } else {
            indexed_samples.push(measure_indexed());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, indexed_samples)
}

fn measure_legacy(references: &[AssetReference]) -> u128 {
    let started = Instant::now();
    black_box(legacy_unique(black_box(references)));
    started.elapsed().as_nanos().max(1)
}

fn measure_indexed(references: &[AssetReference]) -> u128 {
    let started = Instant::now();
    black_box(collect_unique_references(black_box(references.iter())));
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn raw(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
