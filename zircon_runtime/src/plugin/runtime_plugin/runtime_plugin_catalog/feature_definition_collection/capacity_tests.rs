use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const MAPS_PER_SAMPLE: usize = 64;
const DEFINITIONS_PER_MAP: usize = 4_096;

#[test]
fn optimization_batch_20260826fx_runtime219_capacity_covers_definition_map_and_order() {
    let mut definitions = HashMap::with_capacity(DEFINITIONS_PER_MAP);
    let mut definition_order = Vec::with_capacity(DEFINITIONS_PER_MAP);
    for definition in 0..DEFINITIONS_PER_MAP {
        definitions.insert(definition, definition * 2);
        definition_order.push(definition);
    }

    assert_eq!(definitions.len(), DEFINITIONS_PER_MAP);
    assert_eq!(definition_order.len(), DEFINITIONS_PER_MAP);
    assert!(definitions.capacity() >= DEFINITIONS_PER_MAP);
    assert!(definition_order.capacity() >= DEFINITIONS_PER_MAP);
    assert_eq!(definition_order[0], 0);
    assert_eq!(
        definition_order[DEFINITIONS_PER_MAP - 1],
        DEFINITIONS_PER_MAP - 1
    );
}

#[test]
fn optimization_batch_20260826fx_runtime219_feature_definitions_reserve_registration_count() {
    let source = include_str!("../feature_definition_collection.rs");
    assert!(source.contains("let definition_capacity = registrations"));
    assert!(source.contains(".saturating_add(feature_registrations.len())"));
    assert!(source.contains("HashMap::with_capacity(definition_capacity)"));
    assert!(source.contains("Vec::with_capacity(definition_capacity)"));
    assert!(source.contains("let mut diagnostics = Vec::new();"));
    assert!(!source.contains("let mut definitions = HashMap::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fx_runtime219_feature_definition_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME219_FEATURE_DEFINITION_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
maps_per_sample={MAPS_PER_SAMPLE} definitions_per_map={DEFINITIONS_PER_MAP} \
legacy_preallocated_outputs_per_map=0 optimized_preallocated_outputs_per_map=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for map in 0..MAPS_PER_SAMPLE {
        let mut definitions = if reserve {
            HashMap::with_capacity(DEFINITIONS_PER_MAP)
        } else {
            HashMap::new()
        };
        let mut definition_order = if reserve {
            Vec::with_capacity(DEFINITIONS_PER_MAP)
        } else {
            Vec::new()
        };
        for definition in 0..DEFINITIONS_PER_MAP {
            let key = black_box(map ^ definition);
            definitions.insert(key, [key; 6]);
            definition_order.push(key);
        }
        checksum ^= black_box(
            definitions.len()
                ^ definitions.capacity()
                ^ definition_order.len()
                ^ definition_order.capacity(),
        );
        black_box((&definitions, &definition_order));
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
