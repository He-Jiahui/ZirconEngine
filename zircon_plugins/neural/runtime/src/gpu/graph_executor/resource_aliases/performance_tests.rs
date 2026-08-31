use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::ResourceAliases;

const BENCH_TENSOR_COUNT: usize = 4_096;
const CHECKS_PER_SAMPLE: usize = 32;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn dense_resource_aliases_canonicalize_chains_without_resource_name_clones() {
    let mut aliases = ResourceAliases::new(5);

    assert!(aliases.alias(2, 1));
    assert!(aliases.alias(3, 2));
    assert_eq!(aliases.resolve(1), 1);
    assert_eq!(aliases.resolve(2), 1);
    assert_eq!(aliases.resolve(3), 1);
    assert!(!aliases.alias(5, 1));
    assert_eq!(aliases.resolve(5), 5);
}

#[test]
#[ignore = "release-only dense resource alias benchmark"]
fn dense_resource_aliases_release_benchmark_evidence() {
    assert_eq!(legacy_alias_checksum(), optimized_alias_checksum());

    let (legacy_samples, optimized_samples) =
        paired_samples(measure_legacy_aliases, measure_optimized_aliases);
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task=dense_resource_aliases \
sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} \
tensor_count={BENCH_TENSOR_COUNT} legacy_lookup=btree_log_n optimized_lookup=dense_o_1 \
legacy_alias_payload=owned_resource_name optimized_alias_payload=canonical_tensor_id \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(2) <= legacy_p95_ns,
        "dense resource aliases must reduce P95 by at least 50%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_alias_checksum() -> usize {
    let mut aliases = BTreeMap::<u16, String>::new();
    for output in 1..BENCH_TENSOR_COUNT as u16 {
        let source = output - 1;
        let resource = aliases
            .get(&source)
            .cloned()
            .unwrap_or_else(|| format!("nn.tensor.{source}"));
        aliases.insert(output, resource);
    }
    (1..BENCH_TENSOR_COUNT as u16)
        .map(|tensor_id| aliases.get(&tensor_id).map_or(0, String::len))
        .sum()
}

fn optimized_alias_checksum() -> usize {
    let mut aliases = ResourceAliases::new(BENCH_TENSOR_COUNT);
    for output in 1..BENCH_TENSOR_COUNT as u16 {
        assert!(aliases.alias(output, output - 1));
    }
    (1..BENCH_TENSOR_COUNT as u16)
        .map(|tensor_id| format!("nn.tensor.{}", aliases.resolve(tensor_id)).len())
        .sum()
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure_legacy_aliases() -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(legacy_alias_checksum());
    }
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized_aliases() -> u128 {
    let started = Instant::now();
    for _ in 0..CHECKS_PER_SAMPLE {
        black_box(optimized_alias_checksum());
    }
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
