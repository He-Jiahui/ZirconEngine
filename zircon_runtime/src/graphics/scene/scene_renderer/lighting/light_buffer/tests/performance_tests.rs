use std::hint::black_box;
use std::time::Instant;

use super::*;

const LIGHT_COUNT: usize = 32_768;
const VOLUMETRIC_ID_COUNT: usize = 8_192;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime95_frame_metadata_volumetric_index_matches_slice_membership() {
    for ids in [
        Vec::new(),
        vec![2, 2, 7],
        (0..64).map(|id| id * 3).collect::<Vec<_>>(),
    ] {
        let index = VolumetricLightIdIndex::new(&ids);
        for light_id in 0..256 {
            assert_eq!(index.contains(light_id), ids.contains(&light_id));
        }
    }
}

#[test]
#[ignore = "release-only volumetric membership benchmark"]
fn runtime95_frame_metadata_volumetric_release_benchmark_evidence() {
    let lights = (0..LIGHT_COUNT as u64).collect::<Vec<_>>();
    let volumetric_ids = (1_000_000..1_000_000 + VOLUMETRIC_ID_COUNT as u64).collect::<Vec<_>>();

    black_box(time_legacy(&lights, &volumetric_ids));
    black_box(time_optimized(&lights, &volumetric_ids));

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(time_legacy(&lights, &volumetric_ids));
            optimized_samples.push(time_optimized(&lights, &volumetric_ids));
        } else {
            optimized_samples.push(time_optimized(&lights, &volumetric_ids));
            legacy_samples.push(time_legacy(&lights, &volumetric_ids));
        }
    }

    let legacy_p95_ns = nearest_rank(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank(&optimized_samples, 95);
    println!(
        "RUNTIME95_VOLUMETRIC_MEMBERSHIP_PERF lights=32768 volumetric_ids=8192 pairs=21 order=alternating percentile=nearest-rank legacy_comparisons=268435456 optimized_probes=40960 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        nearest_rank(&legacy_samples, 50),
        legacy_p95_ns,
        nearest_rank(&optimized_samples, 50),
        optimized_p95_ns,
        legacy_samples,
        optimized_samples,
    );

    assert!(
        optimized_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "indexed membership must reduce P95 by at least 75%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn time_legacy(lights: &[u64], volumetric_ids: &[u64]) -> u128 {
    let started = Instant::now();
    let matches = lights
        .iter()
        .filter(|light_id| black_box(volumetric_ids).contains(black_box(light_id)))
        .count();
    let elapsed = started.elapsed().as_nanos();
    black_box(matches);
    elapsed
}

fn time_optimized(lights: &[u64], volumetric_ids: &[u64]) -> u128 {
    let started = Instant::now();
    let index = VolumetricLightIdIndex::new(black_box(volumetric_ids));
    let matches = lights
        .iter()
        .filter(|light_id| index.contains(**light_id))
        .count();
    let elapsed = started.elapsed().as_nanos();
    black_box(matches);
    elapsed
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
