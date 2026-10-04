use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 13;
const HASHES_PER_SAMPLE: usize = 8_192;

#[test]
fn optimization_batch_20260830ex_runtime563_batches_artifact_hash_header() {
    let production = include_str!("../source_cubemap_artifact.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("BAKE_ARTIFACT_PAYLOAD_HASH_HEADER_BYTES"));
    assert!(production.contains("hasher.update(&header)"));
    assert!(!production.contains("update_u32_array_hash"));

    let descriptor = benchmark_descriptor();
    let payload = benchmark_payload();
    assert_eq!(
        legacy_artifact_hash(descriptor, &payload),
        optimized_artifact_hash(descriptor, &payload)
    );
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_20260830ex_runtime563_artifact_hash_header_benchmark() {
    let descriptor = benchmark_descriptor();
    let payload = benchmark_payload();
    for _ in 0..3 {
        black_box(measure_artifact_hash(descriptor, &payload, false));
        black_box(measure_artifact_hash(descriptor, &payload, true));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure_artifact_hash(descriptor, &payload, false));
            optimized_samples.push(measure_artifact_hash(descriptor, &payload, true));
        } else {
            optimized_samples.push(measure_artifact_hash(descriptor, &payload, true));
            legacy_samples.push(measure_artifact_hash(descriptor, &payload, false));
        }
    }

    let legacy_p95 = nearest_rank_p95(&legacy_samples);
    let optimized_p95 = nearest_rank_p95(&optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "RUNTIME563_ARTIFACT_HASH_HEADER_BATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} hashes_per_sample={HASHES_PER_SAMPLE} payload_bytes={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=20",
        payload.len()
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(80) / 100,
        "batched artifact hash header must reduce P95 by at least 20%"
    );
}

fn benchmark_descriptor() -> IblBakeArtifactDescriptor {
    IblBakeArtifactDescriptor::current(
        IblBakeKey {
            source_kind: 7,
            source_revision: 19,
            horizon_color: [1, 2, 3, 4],
            zenith_color: [5, 6, 7, 8],
            ground_color: [9, 10, 11, 12],
            source_hash: [13, 14, 15, 16],
        },
        128,
        8,
        IblBakeArtifactContents::PMREM_SH9_IEM,
    )
}

fn benchmark_payload() -> Vec<u8> {
    (0..4_096).map(|index| index as u8).collect()
}

fn optimized_artifact_hash(descriptor: IblBakeArtifactDescriptor, payload: &[u8]) -> blake3::Hash {
    let header = bake_artifact_payload_hash_header(descriptor);
    let mut hasher = blake3::Hasher::new();
    hasher.update(&header);
    hasher.update(payload);
    hasher.finalize()
}

fn legacy_artifact_hash(descriptor: IblBakeArtifactDescriptor, payload: &[u8]) -> blake3::Hash {
    let bake_key = descriptor.bake_key();
    let mut hasher = blake3::Hasher::new();
    hasher.update(&bake_key.source_kind.to_le_bytes());
    hasher.update(&bake_key.source_revision.to_le_bytes());
    for values in [
        bake_key.horizon_color,
        bake_key.zenith_color,
        bake_key.ground_color,
        bake_key.source_hash,
    ] {
        for value in values {
            hasher.update(&value.to_le_bytes());
        }
    }
    hasher.update(&descriptor.algorithm_version().to_le_bytes());
    hasher.update(&(descriptor.producer() as u32).to_le_bytes());
    hasher.update(&descriptor.source_face_size().to_le_bytes());
    hasher.update(&descriptor.source_mip_count().to_le_bytes());
    hasher.update(&descriptor.face_size().to_le_bytes());
    hasher.update(&descriptor.mip_count().to_le_bytes());
    hasher.update(&descriptor.contents().bits().to_le_bytes());
    hasher.update(payload);
    hasher.finalize()
}

fn measure_artifact_hash(
    descriptor: IblBakeArtifactDescriptor,
    payload: &[u8],
    optimized: bool,
) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_u8;
    for index in 0..HASHES_PER_SAMPLE {
        let hash = if optimized {
            optimized_artifact_hash(descriptor, black_box(payload))
        } else {
            legacy_artifact_hash(descriptor, black_box(payload))
        };
        checksum ^= hash.as_bytes()[index & 31];
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
