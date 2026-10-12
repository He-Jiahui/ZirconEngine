use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const SLOTS_PER_SAMPLE: usize = 1_024;

#[test]
fn dynamic_batch_reserves_input_and_retry_collections() {
    let source = include_str!("../dynamic_batch.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    assert!(implementation.contains("Vec::with_capacity(slots.len())"));
    assert!(implementation.contains("Vec::<BatchGroup>::with_capacity(pending.len())"));
    assert!(implementation.contains("HashMap::<BatchKey, usize>::with_capacity(pending.len())"));
    assert!(implementation.contains("Vec::with_capacity(group.entries.len())"));
    assert!(implementation.contains("HashMap::with_capacity(batch.glyphs.len())"));
    assert!(implementation.contains("Vec::with_capacity(pending.len())"));
    assert!(!implementation.contains("let mut pending = Vec::new()"));
}

#[test]
fn dynamic_batch_keeps_group_generation_before_retry_scan() {
    let source = include_str!("../dynamic_batch.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("production implementation");
    let generation = implementation
        .find("generate_batch(group.params, &glyph_ids)")
        .expect("batch generation");
    let retry = implementation
        .find("let mut unresolved = Vec::with_capacity(pending.len())")
        .expect("retry collection");
    assert!(generation < retry);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830by_runtime_dynamic_batch_capacity_p95() {
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(false));
            optimized.push(measure(true));
        } else {
            optimized.push(measure(true));
            legacy.push(measure(false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME377_DYNAMIC_BATCH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} slots_per_sample={SLOTS_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy),
        sample_csv(&optimized),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..128 {
        let mut pending = if optimized {
            Vec::with_capacity(SLOTS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut groups = if optimized {
            Vec::with_capacity(SLOTS_PER_SAMPLE)
        } else {
            Vec::new()
        };
        let mut results = if optimized {
            std::collections::HashMap::with_capacity(SLOTS_PER_SAMPLE)
        } else {
            std::collections::HashMap::new()
        };
        for index in 0..SLOTS_PER_SAMPLE {
            pending.push(index);
            groups.push(index);
            results.insert(index, index);
        }
        let mut unresolved = if optimized {
            Vec::with_capacity(pending.len())
        } else {
            Vec::new()
        };
        unresolved.extend(pending.into_iter().filter(|index| index % 2 == 0));
        checksum ^= groups.len() ^ results.len() ^ unresolved.len();
    }
    std::hint::black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
