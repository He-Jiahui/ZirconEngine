use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const ENTITY_COUNT: usize = 8_192;
const OPERATIONS_PER_SAMPLE: usize = 16;

#[test]
fn optimization_batch_20260826ha_runtime247_preserves_first_duplicate_request_order() {
    let mut entities = std::array::from_fn::<_, 128, _>(|index| index as EntityId + 1_000);
    entities[30] = entities[5];
    entities[60] = entities[2];

    assert_eq!(first_duplicate_entity(&entities), Some(1_005));
    assert_eq!(
        UniqueEntityArray::new(entities),
        Err(QueryEntityError::DuplicateEntity(1_005))
    );
}

#[test]
fn optimization_batch_20260826ha_runtime247_routes_large_arrays_through_hash_scan() {
    let source = include_str!("../unique_entities.rs");
    let start = source
        .find("pub(crate) fn first_duplicate_entity<const N: usize>")
        .expect("first_duplicate_entity function");
    let end = source[start..]
        .find("\nfn first_duplicate_entity_sorted")
        .map(|offset| start + offset)
        .expect("legacy sorted helper boundary");
    let body = &source[start..end];

    assert!(body.contains("first_duplicate_entity_hashed(entities)"));
    assert!(!body.contains("first_duplicate_entity_sorted(entities"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826ha_runtime247_unique_entity_hash_scan_release_benchmark() {
    let entities = std::array::from_fn::<_, ENTITY_COUNT, _>(|index| {
        (index.wrapping_mul(2_654_435_761) & (ENTITY_COUNT - 1)) as EntityId + 1
    });
    assert_eq!(legacy_first_duplicate(&entities), None);
    assert_eq!(first_duplicate_entity(&entities), None);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_first_duplicate(black_box(&entities)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(first_duplicate_entity(black_box(&entities)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME247_UNIQUE_ENTITY_HASH_SCAN_BENCH_V1 entities={ENTITY_COUNT} \
         operations_per_sample={OPERATIONS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} \
         legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} \
         legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_first_duplicate<const N: usize>(entities: &[EntityId; N]) -> Option<EntityId> {
    let mut indexed: [(EntityId, usize); N] = std::array::from_fn(|index| (entities[index], index));
    indexed.sort_unstable();
    indexed
        .windows(2)
        .filter(|pair| pair[0].0 == pair[1].0)
        .map(|pair| (pair[1].1, pair[1].0))
        .min_by_key(|(index, _)| *index)
        .map(|(_, entity)| entity)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
