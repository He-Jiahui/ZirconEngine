use std::hint::black_box;
use std::mem::size_of;
use std::time::Instant;

use super::{decode_tensor_ids, read_u16};

const SAMPLE_PAIRS: usize = 21;
const ITERATIONS_PER_SAMPLE: usize = 2_048;
const INPUT_COUNT: usize = 192;
const OUTPUT_COUNT: usize = 63;

#[test]
#[ignore = "release performance evidence"]
fn znn_tensor_id_decode_release_gate() {
    let encoded = encoded_tensor_ids(INPUT_COUNT + OUTPUT_COUNT);
    for _ in 0..128 {
        black_box(decode_tensor_ids(&encoded, INPUT_COUNT, OUTPUT_COUNT).unwrap());
        black_box(decode_tensor_ids_legacy(&encoded, INPUT_COUNT));
    }

    let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples_ns.push(measure_legacy(&encoded));
            optimized_samples_ns.push(measure_optimized(&encoded));
        } else {
            optimized_samples_ns.push(measure_optimized(&encoded));
            legacy_samples_ns.push(measure_legacy(&encoded));
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples_ns, 95);
    assert!(
        u128::from(optimized_p95_ns) * 100 <= u128::from(legacy_p95_ns) * 110,
        "optimized P95 {optimized_p95_ns}ns exceeded the 10% regression ceiling over legacy P95 {legacy_p95_ns}ns"
    );

    println!(
        "PERF-MVP-PLUGINS02-ZNN-BOUNDED-LOADER sample_pairs={SAMPLE_PAIRS} iterations_per_sample={ITERATIONS_PER_SAMPLE} tensor_ids_per_op={} legacy_allocations_per_op=3 optimized_allocations_per_op=2 allocation_reduction_pct=33 legacy_id_writes_per_op={} optimized_id_writes_per_op={} id_write_reduction_pct=50 legacy_samples_ns={} optimized_samples_ns={} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} target_ratio_pct=110",
        INPUT_COUNT + OUTPUT_COUNT,
        (INPUT_COUNT + OUTPUT_COUNT) * 2,
        INPUT_COUNT + OUTPUT_COUNT,
        join_samples(&legacy_samples_ns),
        join_samples(&optimized_samples_ns),
    );
}

fn measure_legacy(encoded: &[u8]) -> u64 {
    let started = Instant::now();
    for _ in 0..ITERATIONS_PER_SAMPLE {
        black_box(decode_tensor_ids_legacy(encoded, INPUT_COUNT));
    }
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn measure_optimized(encoded: &[u8]) -> u64 {
    let started = Instant::now();
    for _ in 0..ITERATIONS_PER_SAMPLE {
        black_box(decode_tensor_ids(encoded, INPUT_COUNT, OUTPUT_COUNT).unwrap());
    }
    u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX)
}

fn decode_tensor_ids_legacy(encoded: &[u8], input_count: usize) -> (Vec<u16>, Vec<u16>) {
    let tensor_ids = encoded
        .chunks_exact(size_of::<u16>())
        .map(|value| read_u16(value, 0).unwrap())
        .collect::<Vec<_>>();
    (
        tensor_ids[..input_count].to_vec(),
        tensor_ids[input_count..].to_vec(),
    )
}

fn encoded_tensor_ids(count: usize) -> Vec<u8> {
    (0..count)
        .flat_map(|index| {
            u16::try_from(index)
                .expect("benchmark tensor id should fit u16")
                .to_le_bytes()
        })
        .collect()
}

fn nearest_rank_percentile(samples: &[u64], percentile: usize) -> u64 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u64]) -> String {
    samples
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
