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

#[test]
fn retained_axis_transforms_both_endpoints() {
    use crate::core::framework::gizmos::GizmoAxis;

    let mut buffer = GizmoBuffer::new();
    buffer.axis(Vec3::ZERO, GizmoAxis::X, 2.0, color());
    for (scale_x, end_y) in [(2.0, 24.0), (-2.0, 16.0)] {
        let retained = RetainedGizmo::new(GizmoAsset::from_buffer(&buffer)).with_transform(
            Transform::from_translation(Vec3::new(10.0, 20.0, 30.0))
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(scale_x, 3.0, 4.0)),
        );
        let overlay = extract_gizmo_overlay(
            GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera).with_retained(&retained),
        )
        .unwrap();
        assert_eq!(overlay.lines.len(), 1);
        assert_vec3_close(overlay.lines[0].start, Vec3::new(10.0, 20.0, 30.0));
        assert_vec3_close(overlay.lines[0].end, Vec3::new(10.0, end_y, 30.0));
        assert_eq!(overlay.lines[0].color, color());
    }
}

#[test]
fn retained_circle_and_sphere_preserve_affine_geometry() {
    let transform = Transform::from_translation(Vec3::new(10.0, 20.0, 30.0))
        .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
        .with_scale(Vec3::new(2.0, 3.0, 4.0));
    let mut circle = GizmoBuffer::new();
    circle.circle(Vec3::ZERO, Vec3::Z, 1.0, color());
    let mut sphere = GizmoBuffer::new();
    sphere.sphere(Vec3::ZERO, 1.0, color());
    for (buffer, count) in [(&circle, 32), (&sphere, 96)] {
        let asset = GizmoAsset::from_buffer(buffer);
        let identity = RetainedGizmo::new(asset.clone());
        let immediate = extract_gizmo_overlay(
            GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera).with_buffer(buffer),
        )
        .unwrap();
        let identity_overlay = extract_gizmo_overlay(
            GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera).with_retained(&identity),
        )
        .unwrap();
        assert_eq!(identity_overlay.lines, immediate.lines);
        let retained = identity.with_transform(transform);
        let request =
            GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera).with_retained(&retained);
        assert_eq!(estimated_request_line_count(&request), count);
        let overlay = extract_gizmo_overlay(request).unwrap();
        assert_eq!(overlay.lines.len(), count);
        let mirrored = RetainedGizmo::new(asset)
            .with_transform(transform.with_scale(Vec3::new(-2.0, 3.0, 4.0)));
        let mirrored_overlay = extract_gizmo_overlay(
            GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera).with_retained(&mirrored),
        )
        .unwrap();
        assert_eq!(mirrored_overlay.lines.len(), count);
        for (line, original) in mirrored_overlay.lines.iter().zip(&overlay.lines) {
            assert_eq!(line.color, color());
            assert_vec3_close(
                line.start,
                Vec3::new(original.start.x, 40.0 - original.start.y, original.start.z),
            );
            assert_vec3_close(
                line.end,
                Vec3::new(original.end.x, 40.0 - original.end.y, original.end.z),
            );
        }
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for (index, line) in overlay.lines.iter().enumerate() {
            assert_eq!(line.color, color());
            for point in [line.start, line.end] {
                min = min.min(point);
                max = max.max(point);
                let relative = point - Vec3::new(10.0, 20.0, 30.0);
                let ellipsoid = (relative / Vec3::new(3.0, 2.0, 4.0)).length_squared();
                assert!((ellipsoid - 1.0).abs() < 1.0e-5);
            }
            let loop_start = index / 32 * 32;
            let next = loop_start + (index + 1) % 32;
            assert_vec3_close(line.end, overlay.lines[next].start);
        }
        if count == 32 {
            assert_vec3_close(overlay.lines[0].start, Vec3::new(7.0, 20.0, 30.0));
            assert_vec3_close(overlay.lines[8].start, Vec3::new(10.0, 18.0, 30.0));
            assert_vec3_close(min, Vec3::new(7.0, 18.0, 30.0));
            assert_vec3_close(max, Vec3::new(13.0, 22.0, 30.0));
        } else {
            assert_vec3_close(overlay.lines[0].start, Vec3::new(10.0, 20.0, 34.0));
            assert_vec3_close(overlay.lines[32].start, Vec3::new(10.0, 20.0, 26.0));
            assert_vec3_close(overlay.lines[64].start, Vec3::new(7.0, 20.0, 30.0));
            assert_vec3_close(min, Vec3::new(7.0, 18.0, 26.0));
            assert_vec3_close(max, Vec3::new(13.0, 22.0, 34.0));
        }
    }
    let mut rejected = GizmoBuffer::new();
    rejected
        .circle(Vec3::ZERO, Vec3::Z, 0.0, color())
        .sphere(Vec3::ZERO, -1.0, color());
    let retained = RetainedGizmo::new(GizmoAsset::from_buffer(&rejected)).with_transform(transform);
    let request =
        GizmoOverlayExtractRequest::new(49, SceneGizmoKind::Camera).with_retained(&retained);
    assert_eq!(estimated_request_line_count(&request), 0);
    assert!(extract_gizmo_overlay(request).is_none());
}
