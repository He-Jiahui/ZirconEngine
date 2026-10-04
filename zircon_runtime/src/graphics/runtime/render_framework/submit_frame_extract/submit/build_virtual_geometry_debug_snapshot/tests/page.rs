use super::unique_entity_count;

#[test]
fn page_inspections_use_prebuilt_page_and_resident_indices() {
    let source = include_str!("../page.rs");

    assert!(source.contains("build_page_size_index"));
    assert!(source.contains("resident_by_page_id"));
    assert!(!source.contains(concat!("extract", ".pages", ".iter()", ".find")));
    assert!(!source.contains(concat!("resident_page_inspections", ".iter()", ".find")));
}

#[test]
fn optimization_batch_20260830cu_unique_entity_hash_count_preserves_duplicate_semantics() {
    let entities = [9, 2, 9, 4, 2, 7, 4, 7, 11];

    assert_eq!(unique_entity_count(entities.into_iter(), entities.len()), 5);
    assert_eq!(unique_entity_count(std::iter::empty(), 0), 0);
}

#[test]
fn optimization_batch_20260830cu_unique_entity_hash_count_source_contract() {
    let source = include_str!("../page.rs");
    let unique_count = source
        .split("fn unique_extract_entity_count")
        .nth(1)
        .expect("unique entity count implementation")
        .split("pub(super) fn build_resident_page_inspections")
        .next()
        .expect("bounded unique entity count implementation");

    assert!(source.contains("use std::collections::{BTreeMap, HashSet};"));
    assert!(unique_count.contains("HashSet::with_capacity(capacity)"));
    assert!(!unique_count.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence; run through the validation coordinator"]
fn optimization_batch_20260830cu_runtime_unique_entity_hash_count_p95() {
    fn measure(entities: &[u64], count: impl Fn(&[u64]) -> usize) -> u128 {
        let started = std::time::Instant::now();
        for _ in 0..16 {
            std::hint::black_box(count(std::hint::black_box(entities)));
        }
        started.elapsed().as_nanos()
    }

    let entities = (0..65_536_u64)
        .map(|index| index.wrapping_mul(2_654_435_761) % 32_768)
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(17);
    let mut optimized_samples = Vec::with_capacity(17);
    for sample_index in 0..17 {
        let legacy = |values: &[u64]| {
            values
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
        };
        let optimized =
            |values: &[u64]| unique_entity_count(values.iter().copied(), values.len()) as usize;
        if sample_index % 2 == 0 {
            legacy_samples.push(measure(&entities, legacy));
            optimized_samples.push(measure(&entities, optimized));
        } else {
            optimized_samples.push(measure(&entities, optimized));
            legacy_samples.push(measure(&entities, legacy));
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[16];
    let optimized_p95 = optimized_samples[16];
    println!(
        "RUNTIME397_UNIQUE_ENTITY_HASH_COUNT_BENCH_V1 entities={} legacy_p95_ns={} optimized_p95_ns={} target_ratio_bp=7000",
        entities.len(),
        legacy_p95,
        optimized_p95,
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(7_000),
        "hash unique-count P95 {optimized_p95} ns exceeded 70% of tree unique-count {legacy_p95} ns"
    );
}
