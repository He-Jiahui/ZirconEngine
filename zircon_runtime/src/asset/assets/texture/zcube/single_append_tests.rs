use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::render::{append_rgba16f_texels, encode_rgba16f_texels};

use super::{ZcubeSourceCubemapHeader, ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE};

const CHECKS_PER_SAMPLE: usize = 64;
const SAMPLE_PAIRS: usize = 31;
const TEXEL_COUNT: usize = 8192;

fn legacy_encode(texels: &[[f32; 4]]) -> Vec<u8> {
    let mut bytes = ZcubeSourceCubemapHeader {
        face_size: 64,
        mip_count: 7,
    }
    .encode()
    .to_vec();
    bytes.extend_from_slice(&encode_rgba16f_texels(texels));
    bytes
}

fn candidate_encode(texels: &[[f32; 4]]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(
        ZCUBE_SOURCE_CUBEMAP_HEADER_SIZE + texels.len() * super::RGBA16F_TEXEL_SIZE_BYTES,
    );
    bytes.extend_from_slice(
        &ZcubeSourceCubemapHeader {
            face_size: 64,
            mip_count: 7,
        }
        .encode(),
    );
    append_rgba16f_texels(&mut bytes, texels);
    bytes
}

fn measure(texels: &[[f32; 4]], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut bytes = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        let encoded = if optimized {
            candidate_encode(black_box(texels))
        } else {
            legacy_encode(black_box(texels))
        };
        bytes += encoded.len();
        black_box(encoded);
    }
    black_box(bytes);
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

#[test]
fn optimization_batch_20260829bl_runtime339_single_append_preserves_zcube_bytes() {
    let texels = vec![[0.25, 0.5, 1.0, 1.0]; TEXEL_COUNT];
    assert_eq!(candidate_encode(&texels), legacy_encode(&texels));
}

#[test]
fn optimization_batch_20260829bl_runtime339_zcube_encoding_uses_append_path() {
    let source = include_str!("../zcube.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;

    assert!(production.contains("Vec::with_capacity"));
    assert!(production.contains("append_rgba16f_texels(&mut bytes"));
    assert!(!production.contains("encode_rgba16f_texels(cubemap.source_texels())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bl_runtime339_single_append_zcube_bench() {
    let texels = vec![[0.25, 0.5, 1.0, 1.0]; TEXEL_COUNT];
    let mut baseline_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline_samples.push(measure(&texels, false));
            candidate_samples.push(measure(&texels, true));
        } else {
            candidate_samples.push(measure(&texels, true));
            baseline_samples.push(measure(&texels, false));
        }
    }

    let baseline_p50_ns = percentile(&baseline_samples, 50);
    let candidate_p50_ns = percentile(&candidate_samples, 50);
    let baseline_p95_ns = percentile(&baseline_samples, 95);
    let candidate_p95_ns = percentile(&candidate_samples, 95);
    println!(
        "RUNTIME339_SINGLE_APPEND_ZCUBE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
checks_per_sample={CHECKS_PER_SAMPLE} texels={TEXEL_COUNT} \
baseline_payload_allocations=2 candidate_payload_allocations=1 \
baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} \
baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} \
baseline_raw_ns={} candidate_raw_ns={}",
        sample_csv(&baseline_samples),
        sample_csv(&candidate_samples),
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
