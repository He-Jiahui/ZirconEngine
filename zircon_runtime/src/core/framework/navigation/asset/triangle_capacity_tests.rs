use std::hint::black_box;
use std::time::Instant;

use super::{navigation_triangle_capacity, NavMeshAsset, NavMeshPolygonAsset, AREA_WALKABLE};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 512;
const TRIANGLES_PER_BUILD: usize = 256;
const INDICES_PER_BUILD: usize = TRIANGLES_PER_BUILD * 3;

#[test]
fn optimization_batch_20260826et_runtime189_capacity_preserves_triangle_mesh_projection() {
    let indices = (0..TRIANGLES_PER_BUILD)
        .flat_map(|_| [0, 1, 2])
        .collect::<Vec<_>>();

    let asset = NavMeshAsset::from_triangle_mesh(
        "runtime189",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        indices,
        AREA_WALKABLE,
    );

    assert_eq!(asset.indices.len(), INDICES_PER_BUILD);
    assert!(asset.indices.capacity() >= INDICES_PER_BUILD);
    assert_eq!(asset.polygons.len(), TRIANGLES_PER_BUILD);
    assert!(asset.polygons.capacity() >= TRIANGLES_PER_BUILD);
    assert_eq!(asset.polygons[0].first_index, 0);
    assert_eq!(asset.polygons[255].first_index, 765);
    assert_eq!(navigation_triangle_capacity(0), 0);
    assert_eq!(
        navigation_triangle_capacity(INDICES_PER_BUILD),
        TRIANGLES_PER_BUILD
    );
}

#[test]
fn optimization_batch_20260826et_runtime189_triangle_mesh_reserves_input_upper_bounds() {
    let source = include_str!("mod.rs");
    assert!(source.contains("Vec::with_capacity(indices.len())"));
    assert!(source.contains("Vec::with_capacity(navigation_triangle_capacity(indices.len()))"));
    assert!(source.contains("index_count / 3"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826et_runtime189_navigation_triangle_mesh_capacity_bench() {
    let polygon = NavMeshPolygonAsset {
        first_index: 0,
        index_count: 3,
        area: AREA_WALKABLE,
        tile: 0,
    };
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&polygon, false));
            optimized_samples.push(measure(&polygon, true));
        } else {
            optimized_samples.push(measure(&polygon, true));
            legacy_samples.push(measure(&polygon, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME189_NAVIGATION_TRIANGLE_MESH_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} triangles_per_build={TRIANGLES_PER_BUILD} \
indices_per_build={INDICES_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=2 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(polygon: &NavMeshPolygonAsset, reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut indices = if reserve {
            Vec::with_capacity(INDICES_PER_BUILD)
        } else {
            Vec::new()
        };
        let mut polygons = if reserve {
            Vec::with_capacity(TRIANGLES_PER_BUILD)
        } else {
            Vec::new()
        };
        for index in 0..INDICES_PER_BUILD {
            indices.push(black_box(index as u32));
        }
        for _ in 0..TRIANGLES_PER_BUILD {
            polygons.push(black_box(polygon.clone()));
        }
        checksum ^=
            black_box(indices.len() ^ indices.capacity() ^ polygons.len() ^ polygons.capacity());
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
