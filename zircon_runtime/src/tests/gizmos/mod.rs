use std::{hint::black_box, time::Instant};

use crate::core::{
    framework::{
        gizmos::{
            GizmoAsset, GizmoAxis, GizmoBuffer, GizmoColorPolicy, GizmoConfig, GizmoConfigGroupId,
            GizmoLineConfig, GizmoOverlayExtractRequest, GizmoRenderLayer, GizmoScreenScalePolicy,
            RetainedGizmo, append_gizmo_overlay, extract_gizmo_overlay,
        },
        render::{RenderOverlayExtract, SceneGizmoKind},
    },
    math::{Transform, Vec2, Vec3, Vec4},
};

#[test]
fn gizmo_buffer_records_commands_in_order_and_clears() {
    let mut buffer = GizmoBuffer::new();
    buffer
        .line(Vec3::ZERO, Vec3::X, red())
        .ray(Vec3::Y, Vec3::Z, green())
        .axis(Vec3::ZERO, GizmoAxis::Z, 2.0, blue());

    assert_eq!(buffer.commands().len(), 3);
    assert!(!buffer.is_empty());

    buffer.clear();

    assert!(buffer.is_empty());
}

#[test]
fn disabled_gizmo_buffer_does_not_accumulate_commands() {
    let mut config = GizmoConfig::default();
    config.enabled = false;
    let mut buffer = GizmoBuffer::with_config(config);

    buffer
        .line(Vec3::ZERO, Vec3::X, red())
        .sphere(Vec3::ZERO, 1.0, green());

    assert!(buffer.is_empty());
}

#[test]
fn gizmo_config_defaults_cover_m3_rendering_policy() {
    let config = GizmoConfig::default();

    assert_eq!(config.group, GizmoConfigGroupId::new("default"));
    assert!(config.enabled);
    assert_eq!(config.line, GizmoLineConfig { width: 2.0 });
    assert_eq!(config.depth_bias, 0.0);
    assert_eq!(config.render_layer, GizmoRenderLayer(0));
    assert_eq!(config.color_policy, GizmoColorPolicy::UseCommandColor);
    assert_eq!(config.screen_scale_policy, GizmoScreenScalePolicy::World);
}

#[test]
fn retained_gizmo_reuses_buffer_commands_without_sharing_mutable_state() {
    let mut buffer = GizmoBuffer::new();
    buffer.line(Vec3::ZERO, Vec3::X, red());
    let asset = GizmoAsset::from_buffer(&buffer);
    buffer.clear();

    let retained = RetainedGizmo::new(asset.clone());

    assert!(buffer.is_empty());
    assert_eq!(asset.commands().len(), 1);
    assert_eq!(retained.asset.commands().len(), 1);
}

#[test]
fn gizmo_asset_clones_share_immutable_commands_and_preserve_serde_shape() {
    let mut buffer = GizmoBuffer::new();
    buffer
        .line(Vec3::ZERO, Vec3::X, red())
        .linestrip([Vec3::ZERO, Vec3::Y, Vec3::Z], blue());
    let asset = GizmoAsset::from_buffer(&buffer);

    let cloned = asset.clone();
    let retained = RetainedGizmo::new(asset.clone());

    assert!(std::ptr::eq(asset.commands(), cloned.commands()));
    assert!(std::ptr::eq(asset.commands(), retained.asset.commands()));

    let encoded = serde_json::to_string(&asset).unwrap();
    let decoded: GizmoAsset = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.commands(), asset.commands());
}

#[test]
fn gizmo_overlay_extract_converts_line_ray_and_strip_in_order() {
    let mut buffer = GizmoBuffer::new();
    buffer
        .line(Vec3::ZERO, Vec3::X, red())
        .ray(Vec3::Y, Vec3::Z, green())
        .linestrip([Vec3::ZERO, Vec3::Y, Vec3::Z], blue());

    let overlay = extract_gizmo_overlay(
        GizmoOverlayExtractRequest::new(7, SceneGizmoKind::Camera)
            .selected(true)
            .with_buffer(&buffer),
    )
    .expect("line commands should create an overlay");

    assert_eq!(overlay.owner, 7);
    assert_eq!(overlay.kind, SceneGizmoKind::Camera);
    assert!(overlay.selected);
    assert_eq!(overlay.lines.len(), 4);
    assert_eq!(overlay.lines[0].start, Vec3::ZERO);
    assert_eq!(overlay.lines[0].end, Vec3::X);
    assert_eq!(overlay.lines[1].start, Vec3::Y);
    assert_eq!(overlay.lines[1].end, Vec3::Y + Vec3::Z);
    assert_eq!(overlay.lines[2].start, Vec3::ZERO);
    assert_eq!(overlay.lines[2].end, Vec3::Y);
    assert_eq!(overlay.lines[3].start, Vec3::Y);
    assert_eq!(overlay.lines[3].end, Vec3::Z);
}

#[test]
fn gizmo_overlay_extract_filters_disabled_configs_and_applies_color_policy() {
    let mut disabled = GizmoBuffer::new();
    disabled.line(Vec3::ZERO, Vec3::X, red());
    disabled.config_mut().enabled = false;

    let mut config = GizmoConfig::default();
    config.color_policy = GizmoColorPolicy::Override(blue());
    let mut enabled = GizmoBuffer::with_config(config);
    enabled.line(Vec3::ZERO, Vec3::Y, red());

    let overlay = extract_gizmo_overlay(
        GizmoOverlayExtractRequest::new(8, SceneGizmoKind::DirectionalLight)
            .with_buffer(&disabled)
            .with_buffer(&enabled),
    )
    .expect("enabled command should create overlay");

    assert_eq!(overlay.lines.len(), 1);
    assert_eq!(overlay.lines[0].color, blue());
    assert_eq!(overlay.lines[0].end, Vec3::Y);
}

#[test]
fn gizmo_overlay_append_pushes_into_existing_render_overlay_packet() {
    let mut buffer = GizmoBuffer::new();
    buffer.line(Vec3::ZERO, Vec3::X, red());
    let mut packet = RenderOverlayExtract::default();

    append_gizmo_overlay(
        &mut packet,
        GizmoOverlayExtractRequest::new(11, SceneGizmoKind::Camera).with_buffer(&buffer),
    );

    assert_eq!(packet.scene_gizmos.len(), 1);
    assert_eq!(packet.scene_gizmos[0].owner, 11);
    assert_eq!(packet.scene_gizmos[0].lines.len(), 1);
}

#[test]
fn gizmo_overlay_extract_covers_shape_commands() {
    let mut buffer = GizmoBuffer::new();
    buffer
        .rect(Transform::identity(), Vec2::new(2.0, 2.0), red())
        .circle(Vec3::ZERO, Vec3::Z, 1.0, green())
        .sphere(Vec3::ZERO, 1.0, blue())
        .cube(Transform::identity(), Vec3::ONE, red())
        .aabb(Vec3::ZERO, Vec3::ONE, green())
        .axis(Vec3::ZERO, GizmoAxis::X, 1.0, blue());

    let overlay = extract_gizmo_overlay(
        GizmoOverlayExtractRequest::new(9, SceneGizmoKind::NavigationMesh).with_buffer(&buffer),
    )
    .expect("shape commands should create overlay lines");

    assert_eq!(overlay.lines.len(), 4 + 32 + 96 + 12 + 12 + 1);
}

#[test]
fn gizmo_overlay_extract_appends_retained_after_immediate_buffers() {
    let mut immediate = GizmoBuffer::new();
    immediate.line(Vec3::ZERO, Vec3::X, red());

    let mut retained_buffer = GizmoBuffer::new();
    retained_buffer.line(Vec3::ZERO, Vec3::Y, green());
    let retained = RetainedGizmo::new(GizmoAsset::from_buffer(&retained_buffer))
        .with_transform(Transform::from_translation(Vec3::Z));

    let overlay = extract_gizmo_overlay(
        GizmoOverlayExtractRequest::new(10, SceneGizmoKind::Camera)
            .with_buffer(&immediate)
            .with_retained(&retained),
    )
    .expect("immediate and retained commands should create overlay");

    assert_eq!(overlay.lines.len(), 2);
    assert_eq!(overlay.lines[0].start, Vec3::ZERO);
    assert_eq!(overlay.lines[0].end, Vec3::X);
    assert_eq!(overlay.lines[1].start, Vec3::Z);
    assert_eq!(overlay.lines[1].end, Vec3::Y + Vec3::Z);
}

#[test]
fn gizmo_extract_reuses_transform_matrices_and_streams_circle_segments() {
    let source = include_str!("../../core/framework/gizmos/extract.rs");
    assert!(
        !source.contains("transform.matrix().transform_point3"),
        "gizmo extraction must not rebuild a transform matrix per point"
    );
    assert!(
        !source.contains("transform.matrix().transform_vector3"),
        "gizmo extraction must not rebuild a transform matrix per vector"
    );
    assert!(
        !source.contains("Vec::with_capacity(segments)"),
        "circle extraction must stream segments without a temporary point allocation"
    );
}

#[test]
#[ignore = "managed release performance evidence"]
fn gizmo_asset_shared_command_clone_release_benchmark_evidence() {
    const COMMANDS: usize = 20_000;
    const CLONES_PER_SAMPLE: usize = 32;
    const SAMPLE_PAIRS: usize = 21;

    let mut buffer = GizmoBuffer::new();
    for index in 0..COMMANDS {
        let x = index as f32;
        buffer.line(Vec3::new(x, 0.0, 0.0), Vec3::new(x, 1.0, 0.0), red());
    }
    let legacy_commands = buffer.commands().to_vec();
    let asset = GizmoAsset::from_buffer(&buffer);
    let mut legacy_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut shared_samples_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..CLONES_PER_SAMPLE {
                black_box(legacy_commands.clone());
            }
            legacy_samples_ns.push(started.elapsed().as_nanos());
        };
        let mut measure_shared = || {
            let started = Instant::now();
            for _ in 0..CLONES_PER_SAMPLE {
                let cloned = asset.clone();
                black_box(cloned.commands().as_ptr());
            }
            shared_samples_ns.push(started.elapsed().as_nanos());
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_shared();
        } else {
            measure_shared();
            measure_legacy();
        }
    }

    let legacy_p50_ns = nearest_rank_percentile(&legacy_samples_ns, 50);
    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples_ns, 95);
    let shared_p50_ns = nearest_rank_percentile(&shared_samples_ns, 50);
    let shared_p95_ns = nearest_rank_percentile(&shared_samples_ns, 95);
    let legacy_ns = legacy_samples_ns
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let shared_ns = shared_samples_ns
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let legacy_command_copies = COMMANDS * CLONES_PER_SAMPLE * SAMPLE_PAIRS;

    println!(
        "GIZMO_ASSET_CLONE_BENCH_V1 commands={COMMANDS} clones_per_sample={CLONES_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_command_copies={legacy_command_copies} \
         shared_command_copies=0 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} \
         shared_p50_ns={shared_p50_ns} shared_p95_ns={shared_p95_ns} \
         legacy_ns={legacy_ns} shared_ns={shared_ns}"
    );
    assert!(
        shared_p95_ns.saturating_mul(4) <= legacy_p95_ns,
        "shared P95 {shared_p95_ns}ns must be at most 25% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let index = (ordered.len() * percentile).div_ceil(100) - 1;
    ordered[index]
}

fn red() -> Vec4 {
    Vec4::new(1.0, 0.0, 0.0, 1.0)
}

fn green() -> Vec4 {
    Vec4::new(0.0, 1.0, 0.0, 1.0)
}

fn blue() -> Vec4 {
    Vec4::new(0.0, 0.0, 1.0, 1.0)
}
