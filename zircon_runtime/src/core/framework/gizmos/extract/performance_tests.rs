use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::gizmos::{
    GizmoAsset, GizmoBuffer, GizmoConfig, GizmoOverlayExtractRequest, RetainedGizmo,
};
use crate::core::framework::render::SceneGizmoKind;
use crate::core::math::{Quat, Transform, Vec3, Vec4};

use super::{estimated_request_line_count, extract_gizmo_overlay};

const BENCH_SOURCE_COUNT: usize = 100_000;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn runtime49_gizmo_request_estimate_matches_output_and_skips_disabled_sources() {
    let mut immediate = GizmoBuffer::new();
    immediate
        .line(Vec3::ZERO, Vec3::X, color())
        .circle(Vec3::ZERO, Vec3::Z, 1.0, color());

    let mut disabled = GizmoBuffer::new();
    disabled.sphere(Vec3::ZERO, 1.0, color());
    disabled.config_mut().enabled = false;

    let mut retained_buffer = GizmoBuffer::new();
    retained_buffer.sphere(Vec3::ZERO, 1.0, color());
    let retained = RetainedGizmo::new(GizmoAsset::from_buffer(&retained_buffer));

    let mut disabled_config = GizmoConfig::default();
    disabled_config.enabled = false;
    let disabled_retained =
        RetainedGizmo::new(GizmoAsset::from_buffer(&retained_buffer)).with_config(disabled_config);

    let request = GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera)
        .with_buffer(&immediate)
        .with_buffer(&disabled)
        .with_retained(&retained)
        .with_retained(&disabled_retained);
    let estimated = estimated_request_line_count(&request);

    let overlay = extract_gizmo_overlay(request).expect("enabled gizmos should produce lines");

    assert_eq!(estimated, 1 + 32 + 96);
    assert_eq!(overlay.lines.len(), estimated);
    assert!(overlay.lines.capacity() >= estimated);
}

#[test]
fn runtime49_retained_aabb_transforms_all_corners_before_building_edges() {
    let mut buffer = GizmoBuffer::new();
    buffer.aabb(Vec3::ZERO, Vec3::new(2.0, 1.0, 1.0), color());
    let retained = RetainedGizmo::new(GizmoAsset::from_buffer(&buffer)).with_transform(
        Transform::from_translation(Vec3::new(10.0, 20.0, 30.0))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::new(2.0, 3.0, 4.0)),
    );

    let overlay = extract_gizmo_overlay(
        GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera).with_retained(&retained),
    )
    .expect("retained AABB should produce edges");

    assert_eq!(overlay.lines.len(), 12);
    assert_vec3_close(overlay.lines[0].start, Vec3::new(10.0, 20.0, 30.0));
    assert_vec3_close(overlay.lines[0].end, Vec3::new(10.0, 24.0, 30.0));
    assert_vec3_close(overlay.lines[1].end, Vec3::new(7.0, 24.0, 30.0));
    assert_vec3_close(overlay.lines[4].start, Vec3::new(10.0, 20.0, 34.0));
    assert_vec3_close(overlay.lines[10].end, Vec3::new(7.0, 24.0, 34.0));
}

#[test]
#[ignore = "release-only gizmo multi-source reservation benchmark"]
fn runtime49_gizmo_extract_single_reservation_release_benchmark_evidence() {
    let sources = vec![[7usize]; BENCH_SOURCE_COUNT];
    let legacy = reservation_profile(&sources, ReservationMode::PerSource);
    let optimized = reservation_profile(&sources, ReservationMode::WholeRequest);
    assert_eq!(legacy.checksum, optimized.checksum);
    assert_eq!(legacy.reserve_calls, BENCH_SOURCE_COUNT);
    assert_eq!(optimized.reserve_calls, 1);
    assert!(legacy.capacity_growths > 1);
    assert_eq!(optimized.capacity_growths, 1);

    let (legacy_samples, optimized_samples) = paired_samples(
        || measure(&sources, ReservationMode::PerSource),
        || measure(&sources, ReservationMode::WholeRequest),
    );
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);

    println!(
        "PERF_RESULT plan=Runtime49 task=gizmo_extract_single_reservation \
sample_pairs={SAMPLE_PAIRS} source_count={BENCH_SOURCE_COUNT} commands_per_source=1 \
legacy_reserve_calls={} optimized_reserve_calls={} \
legacy_capacity_growths={} optimized_capacity_growths={} \
pair_order=alternating_legacy_even legacy_first_pairs=11 optimized_first_pairs=10 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        legacy.reserve_calls,
        optimized.reserve_calls,
        legacy.capacity_growths,
        optimized.capacity_growths,
        raw(&legacy_samples),
        raw(&optimized_samples),
    );
}

#[derive(Clone, Copy)]
enum ReservationMode {
    PerSource,
    WholeRequest,
}

struct ReservationProfile {
    checksum: usize,
    reserve_calls: usize,
    capacity_growths: usize,
}

fn reservation_profile(sources: &[[usize; 1]], mode: ReservationMode) -> ReservationProfile {
    let mut output = Vec::new();
    let mut reserve_calls = 0;
    let mut capacity_growths = 0;
    if matches!(mode, ReservationMode::WholeRequest) {
        let required = sources
            .iter()
            .map(|source| source.len())
            .fold(0usize, usize::saturating_add);
        let previous_capacity = output.capacity();
        output.reserve(required);
        reserve_calls += 1;
        capacity_growths += usize::from(output.capacity() != previous_capacity);
    }
    for source in sources {
        if matches!(mode, ReservationMode::PerSource) {
            let previous_capacity = output.capacity();
            output.reserve(source.len());
            reserve_calls += 1;
            capacity_growths += usize::from(output.capacity() != previous_capacity);
        }
        output.extend_from_slice(source);
    }
    ReservationProfile {
        checksum: output.iter().copied().fold(0usize, usize::wrapping_add),
        reserve_calls,
        capacity_growths,
    }
}

fn paired_samples(
    mut measure_legacy: impl FnMut() -> u128,
    mut measure_optimized: impl FnMut() -> u128,
) -> (Vec<u128>, Vec<u128>) {
    for _ in 0..4 {
        black_box(measure_legacy());
        black_box(measure_optimized());
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(sources: &[[usize; 1]], mode: ReservationMode) -> u128 {
    let started = Instant::now();
    black_box(reservation_profile(black_box(sources), mode).checksum);
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

fn color() -> Vec4 {
    Vec4::ONE
}

fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    for axis in 0..3 {
        assert!(
            (actual[axis] - expected[axis]).abs() <= 1.0e-5,
            "axis {axis}: expected {}, got {}",
            expected[axis],
            actual[axis]
        );
    }
}
