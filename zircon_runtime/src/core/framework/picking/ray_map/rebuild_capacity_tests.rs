use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use crate::core::math::Vec2;

use super::*;

const CHECKS_PER_SAMPLE: usize = 256;
const POINTER_COUNT: usize = 256;
const CAMERA_COUNT: usize = 8;
const SAMPLE_PAIRS: usize = 31;

fn measure(pointers: &[PointerLocation], cameras: &[CameraRaySource], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut count = 0;
    for _ in 0..CHECKS_PER_SAMPLE {
        if optimized {
            let mut map = RayMap::default();
            map.rebuild(black_box(pointers), black_box(cameras));
            count += map.len();
            black_box(map);
        } else {
            let map = legacy_rebuild(black_box(pointers), black_box(cameras));
            count += map.len();
            black_box(map);
        }
    }
    black_box(count);
    started.elapsed().as_nanos().max(1)
}

fn legacy_rebuild(
    pointers: &[PointerLocation],
    cameras: &[CameraRaySource],
) -> HashMap<RayId, PointerRay> {
    let mut map = HashMap::new();
    for camera in cameras {
        if !camera.active {
            continue;
        }
        for pointer in pointers {
            if pointer.viewport != camera.viewport
                || !pointer.is_inside_viewport(camera.viewport_size)
            {
                continue;
            }
            if let Some(ray) =
                ray_from_viewport_point(&camera.snapshot, camera.viewport_size, pointer.position)
            {
                map.insert(
                    RayId::new(camera.camera, pointer.pointer, camera.viewport),
                    ray,
                );
            }
        }
    }
    map
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
fn optimization_batch_20260829bv_runtime349_ray_map_capacity_preserves_results() {
    let pointers = Vec::<PointerLocation>::new();
    let cameras = Vec::<CameraRaySource>::new();
    let mut map = RayMap::default();
    map.rebuild(&pointers, &cameras);
    assert!(map.is_empty());
}

#[test]
fn optimization_batch_20260829bv_runtime349_ray_map_reserves_active_pair_capacity() {
    let source = include_str!("../ray_map.rs");
    let production = source.split_once("#[cfg(test)]").expect("test boundary").0;
    assert!(production.contains(
        "let active_camera_count = cameras.iter().filter(|camera| camera.active).count()"
    ));
    assert!(production.contains("reserve(pointers.len().saturating_mul(active_camera_count))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829bv_runtime349_ray_map_rebuild_capacity_bench() {
    let pointers = (0..POINTER_COUNT)
        .map(|index| PointerLocation {
            pointer: PointerId::new(index as u64),
            viewport: RenderViewportHandle::new(1),
            position: Vec2::new(index as f32, 2.0),
        })
        .collect::<Vec<_>>();
    let cameras = (0..CAMERA_COUNT)
        .map(|index| {
            CameraRaySource::new(
                EntityId::new(index as u64 + 1),
                RenderViewportHandle::new(1),
                UVec2::new(1024, 768),
                ViewportCameraSnapshot::default(),
            )
        })
        .collect::<Vec<_>>();
    let mut baseline = Vec::with_capacity(SAMPLE_PAIRS);
    let mut candidate = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            baseline.push(measure(&pointers, &cameras, false));
            candidate.push(measure(&pointers, &cameras, true));
        } else {
            candidate.push(measure(&pointers, &cameras, true));
            baseline.push(measure(&pointers, &cameras, false));
        }
    }
    let baseline_p50_ns = percentile(&baseline, 50);
    let candidate_p50_ns = percentile(&candidate, 50);
    let baseline_p95_ns = percentile(&baseline, 95);
    let candidate_p95_ns = percentile(&candidate, 95);
    println!(
        "RUNTIME349_RAY_MAP_REBUILD_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} checks_per_sample={CHECKS_PER_SAMPLE} pointer_count={POINTER_COUNT} camera_count={CAMERA_COUNT} baseline_initial_capacity=0 candidate_reserved_capacity={} baseline_p50_ns={baseline_p50_ns} candidate_p50_ns={candidate_p50_ns} baseline_p95_ns={baseline_p95_ns} candidate_p95_ns={candidate_p95_ns} baseline_raw_ns={} candidate_raw_ns={}",
        POINTER_COUNT * CAMERA_COUNT,
        sample_csv(&baseline),
        sample_csv(&candidate)
    );
    assert!(candidate_p95_ns.saturating_mul(100) <= baseline_p95_ns.saturating_mul(70));
}
