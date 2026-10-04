use std::hint::black_box;
use std::mem::size_of;
use std::time::Instant;

use super::decode_f32;
use crate::onnx::reader::OnnxReadError;

const BENCH_RAW_BYTES: usize = 4 * 1024 * 1024;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn borrowed_raw_f32_decode_preserves_values_and_alignment_error() {
    let bytes = [
        1.25_f32.to_le_bytes(),
        (-3.5_f32).to_le_bytes(),
        0.0_f32.to_le_bytes(),
    ]
    .concat();

    assert_eq!(decode_f32(&bytes), Ok(vec![1.25, -3.5, 0.0]));
    assert_eq!(
        decode_f32(&bytes[..bytes.len() - 1]),
        Err(OnnxReadError::InvalidFloatTensorData)
    );
}

#[test]
#[ignore = "release-only borrowed ONNX raw F32 decode benchmark"]
fn borrowed_raw_f32_decode_release_benchmark_evidence() {
    let bytes = vec![0_u8; BENCH_RAW_BYTES];
    assert_eq!(legacy_decode(&bytes), decode_f32(&bytes).unwrap());

    let (legacy_samples, optimized_samples) =
        paired_samples(|| measure_legacy(&bytes), || measure_optimized(&bytes));
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Plugins02 task=borrowed_onnx_raw_f32_decode \
sample_pairs={SAMPLE_PAIRS} raw_bytes={BENCH_RAW_BYTES} decoded_f32={} \
legacy_raw_byte_clones=1 optimized_raw_byte_clones=0 \
legacy_temporary_raw_bytes={BENCH_RAW_BYTES} optimized_temporary_raw_bytes=0 \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        BENCH_RAW_BYTES / size_of::<f32>(),
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(9),
        "borrowed raw F32 decode must reduce P95 by at least 10%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_decode(bytes: &[u8]) -> Vec<f32> {
    let owned = bytes.to_vec();
    owned
        .chunks_exact(4)
        .map(|value| f32::from_le_bytes(value.try_into().unwrap()))
        .collect()
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

fn measure_legacy(bytes: &[u8]) -> u128 {
    measure(|| legacy_decode(black_box(bytes)))
}

fn measure_optimized(bytes: &[u8]) -> u128 {
    measure(|| decode_f32(black_box(bytes)).unwrap())
}

fn measure(decode: impl FnOnce() -> Vec<f32>) -> u128 {
    let started = Instant::now();
    let decoded = decode();
    black_box(&decoded);
    let elapsed = started.elapsed().as_nanos().max(1);
    black_box(decoded);
    elapsed
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
