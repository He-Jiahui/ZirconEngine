use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const BATCHES_PER_SAMPLE: usize = 48;
const SOURCES_PER_BATCH: usize = 4_096;

#[test]
fn optimization_batch_jk_runtime650_catalog_input_index_reserves_source_count() {
    let source = include_str!("../../full_generation.rs");

    assert!(source.contains("let mut catalog_inputs = HashMap::with_capacity(sources.len());"));
    assert!(!source.contains("let mut catalog_inputs = HashMap::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jk_runtime650_catalog_input_index_capacity_bench() {
    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure(false));
            reserved_samples.push(measure(true));
        } else {
            reserved_samples.push(measure(true));
            unreserved_samples.push(measure(false));
        }
    }

    let unreserved_p50_ns = percentile(&unreserved_samples, 50);
    let reserved_p50_ns = percentile(&reserved_samples, 50);
    let unreserved_p95_ns = percentile(&unreserved_samples, 95);
    let reserved_p95_ns = percentile(&reserved_samples, 95);
    println!(
        "RUNTIME650_CATALOG_INPUT_INDEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
batches_per_sample={BATCHES_PER_SAMPLE} sources_per_batch={SOURCES_PER_BATCH} \
unreserved_p50_ns={unreserved_p50_ns} reserved_p50_ns={reserved_p50_ns} \
unreserved_p95_ns={unreserved_p95_ns} reserved_p95_ns={reserved_p95_ns} \
unreserved_raw_ns={} reserved_raw_ns={}",
        sample_csv(&unreserved_samples),
        sample_csv(&reserved_samples),
    );

    assert!(reserved_p95_ns <= unreserved_p95_ns * 80 / 100);
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for batch in 0..BATCHES_PER_SAMPLE {
        let mut catalog_inputs = if reserve {
            HashMap::with_capacity(SOURCES_PER_BATCH)
        } else {
            HashMap::new()
        };
        for source in 0..SOURCES_PER_BATCH {
            catalog_inputs.insert(black_box(source), [black_box(batch ^ source); 4]);
        }
        checksum ^= catalog_inputs.len()
            ^ catalog_inputs
                .get(&(SOURCES_PER_BATCH - 1))
                .expect("last catalog input")[0];
        black_box(&catalog_inputs);
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
