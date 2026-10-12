use std::hint::black_box;
use std::time::Instant;

use super::{RenderExposureReadbackReport, EXPOSURE_READBACK_EXPECTED_BYTE_LEN};

const CALLS_PER_SAMPLE: usize = 262_144;
const SAMPLE_PAIRS: usize = 31;

#[test]
fn optimization_batch_20260829aw_runtime323_exposure_decode_counts_invalid_words() {
    let bytes = words_to_bytes([f32::NAN, 1.25, f32::INFINITY, -3.5]);

    let report = RenderExposureReadbackReport::from_raw_f32x4_bytes(&bytes);

    assert_eq!(report.invalid_word_count, 2);
    assert_eq!(report.multiplier_bits, f32::NAN.to_bits());
    assert_eq!(report.resolved_ev100_bits, 1.25f32.to_bits());
    assert_eq!(report.average_ev100_bits, f32::INFINITY.to_bits());
    assert_eq!(report.valid_flag_bits, (-3.5f32).to_bits());
}

#[test]
fn optimization_batch_20260829aw_runtime323_short_exposure_decode_zero_fills_missing_words() {
    let bytes = words_to_bytes([1.0, 2.0, 3.0, 4.0]);

    let report = RenderExposureReadbackReport::from_raw_f32x4_bytes(&bytes[..8]);

    assert!(report.invalid_byte_len);
    assert_eq!(report.byte_len, 8);
    assert_eq!(report.invalid_word_count, 0);
    assert_eq!(report.multiplier(), 1.0);
    assert_eq!(report.resolved_ev100(), 2.0);
    assert_eq!(report.average_ev100(), 0.0);
    assert_eq!(report.valid_flag(), 0.0);
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829aw_runtime323_single_pass_exposure_readback_decode_bench() {
    let bytes = words_to_bytes([1.25, f32::NAN, 9.5, 1.0]);
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&bytes, false));
            optimized_samples.push(measure(&bytes, true));
        } else {
            optimized_samples.push(measure(&bytes, true));
            legacy_samples.push(measure(&bytes, false));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME323_SINGLE_PASS_EXPOSURE_READBACK_DECODE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
calls_per_sample={CALLS_PER_SAMPLE} words_per_call=4 legacy_word_passes=2 \
optimized_word_passes=1 legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(bytes: &[u8], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0u32;
    for _ in 0..CALLS_PER_SAMPLE {
        let report = if optimized {
            RenderExposureReadbackReport::from_raw_f32x4_bytes(black_box(bytes))
        } else {
            legacy_from_raw_f32x4_bytes(black_box(bytes))
        };
        checksum ^=
            report.multiplier_bits ^ report.average_ev100_bits ^ report.invalid_word_count as u32;
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn legacy_from_raw_f32x4_bytes(bytes: &[u8]) -> RenderExposureReadbackReport {
    let mut words = [0.0_f32; 4];
    for (word, chunk) in words.iter_mut().zip(bytes.chunks_exact(4)) {
        *word = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    RenderExposureReadbackReport {
        available: true,
        byte_len: bytes.len(),
        expected_byte_len: EXPOSURE_READBACK_EXPECTED_BYTE_LEN,
        invalid_byte_len: bytes.len() != EXPOSURE_READBACK_EXPECTED_BYTE_LEN,
        invalid_word_count: words.iter().filter(|word| !word.is_finite()).count(),
        multiplier_bits: words[0].to_bits(),
        resolved_ev100_bits: words[1].to_bits(),
        average_ev100_bits: words[2].to_bits(),
        valid_flag_bits: words[3].to_bits(),
    }
}

fn words_to_bytes(words: [f32; 4]) -> [u8; EXPOSURE_READBACK_EXPECTED_BYTE_LEN] {
    let mut bytes = [0; EXPOSURE_READBACK_EXPECTED_BYTE_LEN];
    for (index, word) in words.into_iter().enumerate() {
        bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes
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
