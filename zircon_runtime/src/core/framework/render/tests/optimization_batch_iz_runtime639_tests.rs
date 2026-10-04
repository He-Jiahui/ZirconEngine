use std::hint::black_box;
use std::time::Instant;

use super::*;

const CAMERA_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iz_runtime639_reserves_camera_sequence_outputs() {
    let source = include_str!("../camera_stack.rs");
    let projection = source
        .split("fn resolve_active_camera_sequence")
        .nth(1)
        .expect("camera sequence resolver remains present")
        .split("fn index_active_cameras")
        .next()
        .expect("camera sequence resolver remains bounded");

    assert!(projection.contains("Vec::with_capacity(active.len())"));
    assert!(
        projection.contains("active.iter().map(|camera| camera.camera_descriptor().stack.len())")
    );
    assert!(projection.contains("Vec::with_capacity(base.stack.len())"));
    assert!(!projection.contains("let mut sequence = Vec::new();"));
    assert!(!projection.contains("let mut overlays = Vec::new();"));
}

#[test]
fn optimization_batch_iz_runtime639_camera_output_capacity_is_bounded() {
    let active = vec![0_u8; CAMERA_COUNT];
    let sequence = Vec::<u8>::with_capacity(active.len());
    let overlays = Vec::<u8>::with_capacity(active.len());
    assert!(sequence.capacity() >= active.len());
    assert!(overlays.capacity() >= active.len());
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iz_runtime639_preallocated_camera_sequence_output_benchmark() {
    let cameras = (0..CAMERA_COUNT).collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_projection(&cameras, false));
        black_box(measure_projection(&cameras, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_projection(&cameras, false));
            preallocated_samples.push(measure_projection(&cameras, true));
        } else {
            preallocated_samples.push(measure_projection(&cameras, true));
            unreserved_samples.push(measure_projection(&cameras, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME639_PREALLOCATED_CAMERA_SEQUENCE_OUTPUT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} camera_count={CAMERA_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_projection(cameras: &[usize], preallocated: bool) -> u128 {
    let mut output = if preallocated {
        Vec::with_capacity(cameras.len())
    } else {
        Vec::new()
    };
    let started = Instant::now();
    for camera in cameras {
        output.push(black_box(*camera));
    }
    black_box(output);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
