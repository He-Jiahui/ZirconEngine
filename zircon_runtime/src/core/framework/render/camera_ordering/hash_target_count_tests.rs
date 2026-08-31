use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::Instant;

use super::*;
use crate::core::math::UVec2;

const SAMPLE_PAIRS: usize = 21;
const CAMERA_COUNT: usize = 65_536;
const TARGET_COUNT: usize = 4_096;

fn camera(
    entity: EntityId,
    order: i32,
    target: RenderCameraTarget,
    hdr: bool,
) -> RenderCameraOrderInput {
    let mut descriptor =
        CameraRenderDescriptor::from_camera_payload(Some(entity), Default::default());
    descriptor.render_order = order;
    descriptor.target = target;
    descriptor.camera.hdr = hdr;
    RenderCameraOrderInput::from_descriptor(entity, descriptor)
}

#[test]
fn runtime37_batch_camera_hash_counts_preserve_order_and_indices() {
    let surface = RenderCameraTarget::PrimarySurface;
    let headless = RenderCameraTarget::Headless {
        size: UVec2::new(640, 480),
    };
    let report = sort_render_cameras([
        camera(40, 2, surface.clone(), false),
        camera(30, 2, headless.clone(), true),
        camera(10, -1, headless.clone(), false),
        camera(20, 0, headless.clone(), true),
        camera(50, 0, surface.clone(), false),
        camera(60, 2, surface, false),
    ]);

    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.entity)
            .collect::<Vec<_>>(),
        vec![10, 50, 20, 40, 60, 30]
    );
    assert_eq!(
        report
            .cameras
            .iter()
            .map(|camera| camera.sorted_camera_index_for_target)
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1, 2, 1]
    );
    assert_eq!(
        report.ambiguities,
        vec![RenderCameraOrderAmbiguity {
            order: 2,
            target: RenderCameraTargetOrderKey::PrimarySurface,
        }]
    );
}

#[test]
fn runtime37_batch_camera_target_counts_are_hash_private() {
    let source = include_str!("../camera_ordering.rs");
    let implementation = source
        .split("pub fn sort_render_cameras")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("camera ordering implementation");

    assert!(implementation.contains("let mut target_counts = HashMap::new()"));
    assert!(implementation.contains("let mut ambiguities = BTreeSet::new()"));
    assert!(!implementation.contains("let mut target_counts = BTreeMap::new()"));
}

fn target_keys() -> Vec<(RenderCameraTargetOrderKey, bool)> {
    (0..CAMERA_COUNT)
        .map(|index| {
            let target = index % TARGET_COUNT;
            (
                RenderCameraTargetOrderKey::Headless {
                    width: target as u32 + 1,
                    height: (target as u32).wrapping_mul(17) + 1,
                },
                index % 2 == 0,
            )
        })
        .collect()
}

fn measure_legacy(keys: &[(RenderCameraTargetOrderKey, bool)]) -> u128 {
    let started = Instant::now();
    let mut counts = BTreeMap::new();
    let mut checksum = 0usize;
    for key in keys {
        let count = counts.entry(key.clone()).or_insert(0usize);
        checksum = checksum.wrapping_add(*count);
        *count += 1;
    }
    black_box((counts.len(), checksum));
    started.elapsed().as_nanos().max(1)
}

fn measure_optimized(keys: &[(RenderCameraTargetOrderKey, bool)]) -> u128 {
    let started = Instant::now();
    let mut counts = HashMap::new();
    let mut checksum = 0usize;
    for key in keys {
        let count = counts.entry(key.clone()).or_insert(0usize);
        checksum = checksum.wrapping_add(*count);
        *count += 1;
    }
    black_box((counts.len(), checksum));
    started.elapsed().as_nanos().max(1)
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

#[test]
#[ignore = "release-only camera target count benchmark"]
fn runtime37_batch_camera_target_hash_counts_release_benchmark() {
    let keys = target_keys();
    for _ in 0..4 {
        black_box(measure_legacy(&keys));
        black_box(measure_optimized(&keys));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy(&keys));
            optimized_samples.push(measure_optimized(&keys));
        } else {
            optimized_samples.push(measure_optimized(&keys));
            legacy_samples.push(measure_legacy(&keys));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME37_CAMERA_TARGET_HASH_COUNTS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
camera_count={CAMERA_COUNT} target_count={TARGET_COUNT} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        raw(&legacy_samples),
        raw(&optimized_samples),
    );

    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "hash target counts must reduce P95 by at least 30%: \
legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}
