use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const ENTITY_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn legacy_collect(entities: &[u64]) -> (usize, usize) {
    let mut all = HashSet::new();
    let mut static_entities = HashSet::new();
    let mut dynamic_entities = HashSet::new();
    let mut relevance = Vec::new();
    let mut bounds = Vec::new();
    let mut history = Vec::new();
    for (index, entity) in entities.iter().copied().enumerate() {
        all.insert(entity);
        if index % 2 == 0 {
            static_entities.insert(entity);
        } else {
            dynamic_entities.insert(entity);
        }
        relevance.push(entity);
        bounds.push(entity);
        history.push(entity);
    }
    (
        all.len() + static_entities.len() + dynamic_entities.len(),
        relevance.len() + bounds.len() + history.len(),
    )
}

fn optimized_collect(entities: &[u64]) -> (usize, usize) {
    let entity_capacity = entities.len();
    let mut all = HashSet::with_capacity(entity_capacity);
    let mut static_entities = HashSet::with_capacity(entity_capacity);
    let mut dynamic_entities = HashSet::with_capacity(entity_capacity);
    let mut relevance = Vec::with_capacity(entity_capacity);
    let mut bounds = Vec::with_capacity(entity_capacity);
    let mut history = Vec::with_capacity(entity_capacity);
    for (index, entity) in entities.iter().copied().enumerate() {
        all.insert(entity);
        if index % 2 == 0 {
            static_entities.insert(entity);
        } else {
            dynamic_entities.insert(entity);
        }
        relevance.push(entity);
        bounds.push(entity);
        history.push(entity);
    }
    (
        all.len() + static_entities.len() + dynamic_entities.len(),
        relevance.len() + bounds.len() + history.len(),
    )
}

#[test]
fn optimization_batch_io_runtime625_frame_batching_preallocates_known_collection_bounds() {
    let source = include_str!("../../collect_batching_result.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("frame batching production source");

    assert!(production.contains("let entity_capacity = mesh_indices.len();"));
    assert_eq!(
        production
            .matches("EntitySet::with_capacity(entity_capacity)")
            .count(),
        3
    );
    assert_eq!(
        production
            .matches("Vec::with_capacity(entity_capacity)")
            .count(),
        3
    );
    assert!(!production.contains("HashSet::new()"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_io_runtime625_frame_batching_preallocation_performance_evidence() {
    let entities = (0..ENTITY_COUNT as u64)
        .map(|entity| entity.wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .collect::<Vec<_>>();
    assert_eq!(legacy_collect(&entities), optimized_collect(&entities));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_collect(black_box(&entities)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_collect(black_box(&entities)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_collect(black_box(&entities)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_collect(black_box(&entities)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "RUNTIME625_PREALLOCATED_FRAME_BATCHING_BENCH_V1 entities={ENTITY_COUNT} \
         hash_sets=3 vectors=3 legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} \
         target_ratio_bp=8500"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(8_500),
        "preallocated batching P95 {optimized_p95} ns exceeded 85% of unreserved {legacy_p95} ns"
    );
}
