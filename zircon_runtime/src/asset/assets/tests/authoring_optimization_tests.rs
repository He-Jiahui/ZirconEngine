use std::hint::black_box;
use std::time::Instant;

use super::*;

const LAYER_COUNT: usize = 4_096;
const SAMPLE_PAIRS: usize = 17;

fn reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn fixture() -> TerrainAsset {
    let material = reference("res://materials/terrain.zmaterial");
    let weightmap = reference("res://terrain/weightmap.png");
    TerrainAsset {
        uri: AssetUri::parse("res://terrain/performance.terrain.toml").unwrap(),
        name: "Performance".to_string(),
        width: 1,
        height: 1,
        sample_spacing: 1.0,
        height_scale: 1.0,
        height_samples: vec![0.0],
        layers: (0..LAYER_COUNT)
            .map(|index| TerrainLayerAsset {
                name: format!("Layer {index}"),
                material: Some(material.clone()),
                weightmap: Some(weightmap.clone()),
                strength: 1.0,
            })
            .collect(),
    }
}

fn legacy_direct_references(terrain: &TerrainAsset) -> Vec<AssetReference> {
    terrain
        .layers
        .iter()
        .flat_map(|layer| {
            layer
                .material
                .iter()
                .chain(layer.weightmap.iter())
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect()
}

fn elapsed_micros(run: impl FnOnce()) -> u128 {
    let started = Instant::now();
    run();
    started.elapsed().as_micros()
}

fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * 95).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
#[ignore = "release performance evidence for the managed validation coordinator"]
fn optimization_batch_20260826c_runtime86_terrain_reference_append_performance_evidence() {
    let terrain = fixture();
    let expected_reference_count = LAYER_COUNT * 2;

    for _ in 0..4 {
        assert_eq!(
            black_box(legacy_direct_references(&terrain)).len(),
            expected_reference_count
        );
        assert_eq!(
            black_box(terrain.direct_references()).len(),
            expected_reference_count
        );
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let measure_legacy = || {
            elapsed_micros(|| {
                black_box(legacy_direct_references(black_box(&terrain)));
            })
        };
        let measure_optimized = || {
            elapsed_micros(|| {
                black_box(black_box(&terrain).direct_references());
            })
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
    let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
    println!(
        "RUNTIME86_TERRAIN_REFERENCE_APPEND_BENCH_V1 sample_pairs={} layers={} references={} legacy_temporary_layer_vectors={} optimized_temporary_layer_vectors=0 legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
        SAMPLE_PAIRS,
        LAYER_COUNT,
        expected_reference_count,
        LAYER_COUNT,
        legacy_p95,
        optimized_p95,
        legacy_samples,
        optimized_samples,
    );
    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(75),
        "single-buffer terrain reference append p95 must be at least 25% below per-layer vectors: legacy={legacy_p95}us optimized={optimized_p95}us"
    );
}
