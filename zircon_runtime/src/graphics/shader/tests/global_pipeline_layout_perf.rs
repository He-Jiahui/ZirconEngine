use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const RESOURCES_PER_SAMPLE: usize = 512;

#[test]
fn shader_layout_projection_uses_preallocated_hash_lookups() {
    let source = include_str!("../global_pipeline_layout.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("global shader layout implementation");

    assert!(implementation.contains("use std::collections::{HashMap, HashSet}"));
    assert!(implementation.contains("HashMap::with_capacity(resource_types.len())"));
    assert!(implementation.contains("HashSet::with_capacity(bindings.len())"));
    assert!(!implementation.contains("BTreeMap::new()"));
    assert!(!implementation.contains("collect::<BTreeSet<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260830cm_runtime390_shader_layout_hash_lookup_p95() {
    let names = (0..RESOURCES_PER_SAMPLE)
        .map(|index| format!("shader_resource_{index:04}_shared_prefix"))
        .collect::<Vec<_>>();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&names, false));
            optimized.push(measure(&names, true));
        } else {
            optimized.push(measure(&names, true));
            legacy.push(measure(&names, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME390_SHADER_LAYOUT_HASH_LOOKUP_BENCH_V1 sample_pairs={SAMPLE_PAIRS} resources_per_sample={RESOURCES_PER_SAMPLE} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(names: &[String], use_hash: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..64 {
        if use_hash {
            let mut by_name = HashMap::with_capacity(names.len());
            let mut declared = HashSet::with_capacity(names.len());
            for (index, name) in black_box(names).iter().enumerate() {
                by_name.insert(name.as_str(), index);
                declared.insert(name.as_str());
            }
            checksum ^= names
                .iter()
                .map(|name| by_name[name.as_str()] + usize::from(declared.contains(name.as_str())))
                .sum::<usize>();
        } else {
            let mut by_name = BTreeMap::new();
            let mut declared = BTreeSet::new();
            for (index, name) in black_box(names).iter().enumerate() {
                by_name.insert(name.as_str(), index);
                declared.insert(name.as_str());
            }
            checksum ^= names
                .iter()
                .map(|name| by_name[name.as_str()] + usize::from(declared.contains(name.as_str())))
                .sum::<usize>();
        }
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], p: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * p).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
