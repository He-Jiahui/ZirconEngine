use std::hint::black_box;
use std::time::Instant;

use super::*;

const VOLUME_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 21;
const OPERATIONS_PER_SAMPLE: usize = 4;

#[test]
fn runtime99_fog_volume_filter_matches_legacy_projection() {
    let volumes = fixture_volumes(96);
    let render_layers = RenderLayerSet::layer(1);

    assert_eq!(
        collect_gpu_volumes(&volumes, true, Some(&render_layers)),
        legacy_collect_gpu_volumes(&volumes, true, &render_layers)
    );
    assert_eq!(
        collect_gpu_volumes(&volumes, false, Some(&render_layers)),
        legacy_collect_gpu_volumes(&volumes, false, &render_layers)
    );
    assert_eq!(
        collect_gpu_volumes(&volumes, true, None),
        volumes.iter().map(GpuFogVolume::from).collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "release-only fog volume upload benchmark"]
fn runtime99_fog_volume_upload_release_benchmark_evidence() {
    let volumes = fixture_volumes(VOLUME_COUNT);
    let half_layers = RenderLayerSet::layer(1);
    let all_layers = RenderLayerSet::from_layers([1, 2]);

    black_box(time_legacy(&volumes, true, &half_layers));
    black_box(time_optimized(&volumes, true, &half_layers));
    black_box(time_legacy(&volumes, false, &all_layers));
    black_box(time_optimized(&volumes, false, &all_layers));

    let mut legacy_filter_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_filter_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut legacy_disabled_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_disabled_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_filter_samples.push(time_legacy(&volumes, true, &half_layers));
            optimized_filter_samples.push(time_optimized(&volumes, true, &half_layers));
            legacy_disabled_samples.push(time_legacy(&volumes, false, &all_layers));
            optimized_disabled_samples.push(time_optimized(&volumes, false, &all_layers));
        } else {
            optimized_filter_samples.push(time_optimized(&volumes, true, &half_layers));
            legacy_filter_samples.push(time_legacy(&volumes, true, &half_layers));
            optimized_disabled_samples.push(time_optimized(&volumes, false, &all_layers));
            legacy_disabled_samples.push(time_legacy(&volumes, false, &all_layers));
        }
    }

    let legacy_filter_p95_ns = nearest_rank(&legacy_filter_samples, 95);
    let optimized_filter_p95_ns = nearest_rank(&optimized_filter_samples, 95);
    let legacy_disabled_p95_ns = nearest_rank(&legacy_disabled_samples, 95);
    let optimized_disabled_p95_ns = nearest_rank(&optimized_disabled_samples, 95);
    println!(
        "RUNTIME99_FOG_VOLUME_FILTER_PERF volumes=65536 visible=32768 pairs=21 operations_per_sample=4 order=alternating percentile=nearest-rank legacy_volume_clones=32768 optimized_volume_clones=0 legacy_vectors=2 optimized_vectors=1 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        nearest_rank(&legacy_filter_samples, 50),
        legacy_filter_p95_ns,
        nearest_rank(&optimized_filter_samples, 50),
        optimized_filter_p95_ns,
        legacy_filter_samples,
        optimized_filter_samples,
    );
    println!(
        "RUNTIME99_FOG_VOLUME_DISABLED_PERF volumes=65536 pairs=21 operations_per_sample=4 order=alternating percentile=nearest-rank legacy_volume_visits=65536 optimized_volume_visits=0 legacy_volume_clones=65536 optimized_volume_clones=0 legacy_p50_ns={} legacy_p95_ns={} optimized_p50_ns={} optimized_p95_ns={} legacy_samples_ns={:?} optimized_samples_ns={:?}",
        nearest_rank(&legacy_disabled_samples, 50),
        legacy_disabled_p95_ns,
        nearest_rank(&optimized_disabled_samples, 50),
        optimized_disabled_p95_ns,
        legacy_disabled_samples,
        optimized_disabled_samples,
    );

    assert!(
        optimized_filter_p95_ns.saturating_mul(4) <= legacy_filter_p95_ns,
        "fused layer filtering must reduce P95 by at least 75%: legacy={legacy_filter_p95_ns}ns optimized={optimized_filter_p95_ns}ns"
    );
    assert!(
        optimized_disabled_p95_ns.saturating_mul(10) <= legacy_disabled_p95_ns,
        "disabled local volumes must reduce P95 by at least 90%: legacy={legacy_disabled_p95_ns}ns optimized={optimized_disabled_p95_ns}ns"
    );
}

fn fixture_volumes(count: usize) -> Vec<FogVolumeData> {
    (0..count)
        .map(|index| FogVolumeData {
            volume_id: index as u64,
            bounds_min: Vec3::new(index as f32, 0.0, 0.0),
            bounds_max: Vec3::new(index as f32 + 1.0, 1.0, 1.0),
            density: 0.25,
            albedo: Vec3::new(0.25, 0.5, 0.75),
            layer_mask: RenderLayerSet::layer(if index % 2 == 0 { 1 } else { 2 }),
        })
        .collect()
}

fn legacy_collect_gpu_volumes(
    local_volumes: &[FogVolumeData],
    include_local_volumes: bool,
    render_layers: &RenderLayerSet,
) -> Vec<GpuFogVolume> {
    let selected = local_volumes
        .iter()
        .filter(|volume| volume.layer_mask.intersects(render_layers))
        .cloned()
        .collect::<Vec<_>>();
    if !include_local_volumes {
        black_box(selected);
        return Vec::new();
    }
    selected.iter().map(GpuFogVolume::from).collect()
}

fn time_legacy(
    volumes: &[FogVolumeData],
    include_local_volumes: bool,
    render_layers: &RenderLayerSet,
) -> u128 {
    let started = Instant::now();
    for _ in 0..OPERATIONS_PER_SAMPLE {
        let projected = legacy_collect_gpu_volumes(
            black_box(volumes),
            include_local_volumes,
            black_box(render_layers),
        );
        black_box(projected);
    }
    started.elapsed().as_nanos() / OPERATIONS_PER_SAMPLE as u128
}

fn time_optimized(
    volumes: &[FogVolumeData],
    include_local_volumes: bool,
    render_layers: &RenderLayerSet,
) -> u128 {
    let started = Instant::now();
    for _ in 0..OPERATIONS_PER_SAMPLE {
        let projected = collect_gpu_volumes(
            black_box(volumes),
            include_local_volumes,
            Some(black_box(render_layers)),
        );
        black_box(projected);
    }
    started.elapsed().as_nanos() / OPERATIONS_PER_SAMPLE as u128
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
