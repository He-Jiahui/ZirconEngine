use std::hint::black_box;
use std::time::{Duration, Instant};

use super::*;

const PERFORMANCE_POLYGON_COUNT: usize = 32_768;
const PERFORMANCE_SAMPLE_COUNT: usize = 15;

#[test]
fn optimization_batch_cy_runtime401_median_selection_preserves_nearest_queries() {
    let polygons = polygon_fixture(1_024);
    let legacy = legacy_spatial_index(&polygons);
    let selected = PolygonSpatialIndex::new(&polygons);

    assert_eq!(selected.node_count(), legacy.node_count());
    for probe in 0..128 {
        let position = Vec3::new(
            ((probe * 271) % 1_024) as Real + 0.25,
            0.0,
            ((probe * 613 + 17) % 1_024) as Real + 0.75,
        );
        assert_eq!(
            nearest_polygon(&selected, &polygons, position),
            nearest_polygon(&legacy, &polygons, position),
        );
    }
}

#[test]
fn optimization_batch_cy_runtime401_spatial_index_selects_medians_without_full_sort() {
    let production = include_str!("../spatial_index.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");

    assert!(production.contains("select_nth_unstable_by"));
    assert!(!production.contains("polygon_indices.sort_unstable_by"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_cy_runtime401_median_selection_performance_evidence() {
    let polygons = polygon_fixture(PERFORMANCE_POLYGON_COUNT);
    assert_eq!(
        PolygonSpatialIndex::new(&polygons).node_count(),
        legacy_spatial_index(&polygons).node_count(),
    );

    let mut legacy_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
    let mut selected_samples = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
    for sample in 0..PERFORMANCE_SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure(|| {
                black_box(legacy_spatial_index(black_box(&polygons)))
            }));
            selected_samples.push(measure(|| {
                black_box(PolygonSpatialIndex::new(black_box(&polygons)))
            }));
        } else {
            selected_samples.push(measure(|| {
                black_box(PolygonSpatialIndex::new(black_box(&polygons)))
            }));
            legacy_samples.push(measure(|| {
                black_box(legacy_spatial_index(black_box(&polygons)))
            }));
        }
    }

    let legacy_p95 = percentile_95(&mut legacy_samples);
    let selected_p95 = percentile_95(&mut selected_samples);
    println!(
        "RUNTIME401_SPATIAL_INDEX_SELECT_NTH_BENCH_V1 polygons={PERFORMANCE_POLYGON_COUNT} \
             legacy_recursive_sort=true median_select=true legacy_p95_ns={} selected_p95_ns={}",
        legacy_p95.as_nanos(),
        selected_p95.as_nanos(),
    );
    assert!(
        selected_p95.as_nanos() * 100 <= legacy_p95.as_nanos() * 70,
        "median-selection P95 {:?} exceeded 70% of recursive-sort P95 {:?}",
        selected_p95,
        legacy_p95,
    );
}

fn polygon_fixture(count: usize) -> Vec<BakedPolygon> {
    (0..count)
        .map(|index| {
            let x = ((index * 48_271) % count) as Real;
            let z = ((index * 69_621 + 17) % count) as Real;
            let center = Vec3::new(x, 0.0, z);
            BakedPolygon {
                area: 0,
                center,
                min: center - Vec3::splat(0.5),
                max: center + Vec3::splat(0.5),
                edge_keys: Vec::new(),
            }
        })
        .collect()
}

fn nearest_polygon(
    index: &PolygonSpatialIndex,
    polygons: &[BakedPolygon],
    position: Vec3,
) -> Option<usize> {
    let mut best = None;
    let mut work = MeshQueryWork::default();
    index.visit_nearest(polygons, position, u64::MAX, &mut best, &mut work);
    best.map(|(polygon, _)| polygon)
}

fn legacy_spatial_index(polygons: &[BakedPolygon]) -> PolygonSpatialIndex {
    let mut index = PolygonSpatialIndex::default();
    if polygons.is_empty() {
        return index;
    }
    let mut polygon_indices = (0..polygons.len()).collect::<Vec<_>>();
    legacy_build_node(&mut index, &mut polygon_indices, polygons);
    index
}

fn legacy_build_node(
    index: &mut PolygonSpatialIndex,
    polygon_indices: &mut [usize],
    polygons: &[BakedPolygon],
) -> usize {
    let (min, max) = polygon_bounds(polygon_indices, polygons);
    let node_index = index.nodes.len();
    index
        .nodes
        .push(PolygonSpatialIndexNode::leaf(min, max, 0, 0));
    if polygon_indices.len() <= POLYGON_SPATIAL_INDEX_LEAF_SIZE {
        let start = index.polygon_order.len();
        index.polygon_order.extend_from_slice(polygon_indices);
        let end = index.polygon_order.len();
        index.nodes[node_index] = PolygonSpatialIndexNode::leaf(min, max, start, end);
        return node_index;
    }

    let extent = max - min;
    if extent.x >= extent.z {
        polygon_indices.sort_unstable_by(|left, right| {
            polygons[*left]
                .center
                .x
                .total_cmp(&polygons[*right].center.x)
                .then(left.cmp(right))
        });
    } else {
        polygon_indices.sort_unstable_by(|left, right| {
            polygons[*left]
                .center
                .z
                .total_cmp(&polygons[*right].center.z)
                .then(left.cmp(right))
        });
    }
    let middle = polygon_indices.len() / 2;
    let (left_indices, right_indices) = polygon_indices.split_at_mut(middle);
    let left = legacy_build_node(index, left_indices, polygons);
    let right = legacy_build_node(index, right_indices, polygons);
    index.nodes[node_index] = PolygonSpatialIndexNode::branch(min, max, left, right);
    node_index
}

fn measure<T>(run: impl FnOnce() -> T) -> Duration {
    let started = Instant::now();
    black_box(run());
    started.elapsed()
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}
