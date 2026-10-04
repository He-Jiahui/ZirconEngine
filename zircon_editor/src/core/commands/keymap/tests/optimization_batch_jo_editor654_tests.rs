use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 17;
const BINDING_COUNT: usize = 16_384;

#[test]
fn optimization_batch_jo_editor654_keymap_signature_index_reserves_binding_upper_bound() {
    let source = include_str!("../../keymap.rs");
    assert!(source.contains("HashMap::<EditorKeyChordSignature, Vec<usize>>::with_capacity"));
    assert!(source.contains("with_capacity(bindings.len())"));
    assert!(!source.contains("HashMap::<EditorKeyChordSignature, Vec<usize>>::new()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_jo_editor654_keymap_signature_index_capacity_bench() {
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
    let unreserved_p95_ns = percentile(&unreserved_samples, 95);
    let reserved_p95_ns = percentile(&reserved_samples, 95);
    println!(
        "EDITOR654_KEYMAP_SIGNATURE_INDEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} binding_count={BINDING_COUNT} unreserved_p95_ns={unreserved_p95_ns} reserved_p95_ns={reserved_p95_ns} unreserved_raw_ns={} reserved_raw_ns={}",
        sample_csv(&unreserved_samples),
        sample_csv(&reserved_samples),
    );
    assert!(reserved_p95_ns <= unreserved_p95_ns * 80 / 100);
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for batch in 0..32 {
        let mut index = if reserve {
            HashMap::<usize, Vec<usize>>::with_capacity(BINDING_COUNT)
        } else {
            HashMap::<usize, Vec<usize>>::new()
        };
        index.extend((0..BINDING_COUNT).map(|key| (key, vec![batch ^ key])));
        checksum ^= index.len();
        black_box(index);
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
