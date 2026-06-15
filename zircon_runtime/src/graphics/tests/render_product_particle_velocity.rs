use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasSettings, FallbackSkyboxKind, PreviewEnvironmentExtract, RenderFrameExtract,
    RenderFramework, RenderMotionBlurSettings, RenderOverlayExtract,
    RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot, RenderPipelineHandle,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot,
    RenderViewportDescriptor, RenderViewportHandle, RenderWorldSnapshotHandle,
    ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::graphics::WgpuRenderFramework;
use crate::graphics::{RenderPassExecutionContext, RenderPassExecutorRegistration};

use super::plugin_render_feature_fixtures::particle_render_feature_descriptor_with_velocity;

const PARTICLE_TRANSPARENT_EXECUTOR_ID: &str = "particle.transparent";
const PARTICLE_VELOCITY_EXECUTOR_ID: &str = "particle.velocity";
const TAA_RESOLVE_EXECUTOR_ID: &str = "temporal.taa-resolve";

#[test]
fn render_product_particle_velocity_writer_runs_before_particle_color_and_taa() {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract(viewport_size, true),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_particle_graph_executed_pass_count, 2);
    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
    assert_executor_executed(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_VELOCITY_EXECUTOR_ID,
    );
    assert_executor_executed(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_TRANSPARENT_EXECUTOR_ID,
    );
    assert_executor_order(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_VELOCITY_EXECUTOR_ID,
        PARTICLE_TRANSPARENT_EXECUTOR_ID,
    );
    assert_executor_order(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_VELOCITY_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    );
}

#[test]
fn render_product_particle_velocity_writer_writes_nonzero_scene_velocity_pixels() {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract(viewport_size, true),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_scene_velocity_readback_nonzero(&stats, viewport_size);
    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
}

#[test]
fn render_product_particle_velocity_writer_noops_without_previous_state() {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract(viewport_size, false),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_particle_graph_executed_pass_count, 2);
    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 1);
    assert_executor_executed(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_VELOCITY_EXECUTOR_ID,
    );
    assert_executor_order(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_VELOCITY_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    );
}

#[test]
fn render_product_particle_velocity_writer_uses_renderer_owned_previous_state_on_second_frame() {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_at(viewport_size, false, Vec3::new(-0.25, 0.0, -2.5)),
        )
        .unwrap();
    assert_eq!(
        framework
            .query_stats()
            .unwrap()
            .last_particle_velocity_missing_sprite_count,
        1
    );

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_at(viewport_size, false, Vec3::new(0.25, 0.0, -2.5)),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
    assert_scene_velocity_readback_nonzero(&stats, viewport_size);
    assert_executor_order(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_VELOCITY_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    );
}

#[test]
fn render_product_particle_velocity_writer_matches_same_entity_renderer_owned_sprites_by_key_on_second_frame(
) {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(
                viewport_size,
                &[
                    (11, Vec3::new(-0.55, -0.18, -2.5)),
                    (12, Vec3::new(0.55, 0.18, -2.5)),
                ],
            ),
        )
        .unwrap();
    assert_eq!(
        framework
            .query_stats()
            .unwrap()
            .last_particle_velocity_missing_sprite_count,
        2
    );

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(
                viewport_size,
                &[
                    (11, Vec3::new(-0.15, -0.18, -2.5)),
                    (12, Vec3::new(0.15, 0.18, -2.5)),
                ],
            ),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
    assert_eq!(
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        0
    );
    assert_scene_velocity_readback_nonzero(&stats, viewport_size);
    assert_executor_order(
        &stats.last_graph_executed_executor_ids,
        PARTICLE_VELOCITY_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    );
}

#[test]
fn render_product_particle_velocity_writer_rolls_keyed_multi_sprite_motion_across_three_frames() {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);
    let frame_one = [
        (21, Vec3::new(-0.72, -0.30, -2.5)),
        (22, Vec3::new(-0.24, 0.25, -2.5)),
        (23, Vec3::new(0.24, -0.05, -2.5)),
        (24, Vec3::new(0.72, 0.30, -2.5)),
    ];
    let frame_two = [
        (21, Vec3::new(-0.52, -0.25, -2.5)),
        (22, Vec3::new(-0.06, 0.18, -2.5)),
        (23, Vec3::new(0.42, 0.05, -2.5)),
        (24, Vec3::new(0.54, 0.20, -2.5)),
    ];
    let frame_three = [
        (21, Vec3::new(-0.34, -0.18, -2.5)),
        (22, Vec3::new(0.12, 0.12, -2.5)),
        (23, Vec3::new(0.60, 0.14, -2.5)),
        (24, Vec3::new(0.36, 0.10, -2.5)),
    ];

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(viewport_size, &frame_one),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();
    assert_eq!(
        stats.last_particle_velocity_missing_sprite_count,
        frame_one.len()
    );
    assert_eq!(
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        0
    );

    for keyed_positions in [&frame_two[..], &frame_three[..]] {
        framework
            .submit_frame_extract(
                viewport,
                particle_motion_blur_taa_extract_with_keyed_positions(
                    viewport_size,
                    keyed_positions,
                ),
            )
            .unwrap();
        let stats = framework.query_stats().unwrap();

        assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
        assert_eq!(
            stats.last_particle_velocity_anonymous_stream_ambiguity_count,
            0
        );
        assert_scene_velocity_readback_nonzero(&stats, viewport_size);
    }
}

#[test]
fn render_product_particle_velocity_writer_rolls_keyed_stress_field_motion() {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);
    let frame_one = keyed_particle_field_positions(Vec3::ZERO);
    let frame_two = keyed_particle_field_positions(Vec3::new(0.08, -0.04, 0.0));

    assert_eq!(frame_one.len(), 32);
    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(viewport_size, &frame_one),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();
    assert_eq!(
        stats.last_particle_velocity_missing_sprite_count,
        frame_one.len()
    );
    assert_eq!(
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        0
    );

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(viewport_size, &frame_two),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
    assert_eq!(
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        0
    );
    assert_scene_velocity_readback_nonzero(&stats, viewport_size);
}

#[test]
fn render_product_particle_velocity_reports_anonymous_key_multi_sprite_ambiguity() {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(
                viewport_size,
                &[
                    (0, Vec3::new(-0.55, -0.18, -2.5)),
                    (0, Vec3::new(0.55, 0.18, -2.5)),
                ],
            ),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 2);
    assert_eq!(
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        2
    );
}

#[test]
fn render_product_particle_velocity_rejects_anonymous_key_multi_sprite_previous_state_on_second_frame(
) {
    let framework = particle_velocity_framework();
    let viewport_size = UVec2::new(320, 240);
    let viewport = create_particle_velocity_viewport(&framework, viewport_size);

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(
                viewport_size,
                &[
                    (0, Vec3::new(-0.55, -0.18, -2.5)),
                    (0, Vec3::new(0.55, 0.18, -2.5)),
                ],
            ),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();
    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 2);
    assert_eq!(
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        2
    );

    framework
        .submit_frame_extract(
            viewport,
            particle_motion_blur_taa_extract_with_keyed_positions(
                viewport_size,
                &[
                    (0, Vec3::new(-0.15, -0.18, -2.5)),
                    (0, Vec3::new(0.15, 0.18, -2.5)),
                ],
            ),
        )
        .unwrap();
    let stats = framework.query_stats().unwrap();

    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 2);
    assert_eq!(
        stats.last_particle_velocity_anonymous_stream_ambiguity_count,
        2
    );
}

fn particle_velocity_framework() -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [particle_render_feature_descriptor_with_velocity()],
        [RenderPassExecutorRegistration::new(
            PARTICLE_TRANSPARENT_EXECUTOR_ID,
            particle_transparent_billboard_executor,
        )],
        Vec::new(),
    )
    .unwrap()
}

fn create_particle_velocity_viewport(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
) -> RenderViewportHandle {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, particle_velocity_profile())
        .unwrap();
    viewport
}

fn particle_velocity_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("runtime-particle-velocity")
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(true)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(true)
        .with_anti_alias(true)
}

fn particle_motion_blur_taa_extract(
    viewport_size: UVec2,
    include_previous_state: bool,
) -> RenderFrameExtract {
    particle_motion_blur_taa_extract_at(
        viewport_size,
        include_previous_state,
        Vec3::new(0.25, 0.0, -2.5),
    )
}

fn particle_motion_blur_taa_extract_at(
    viewport_size: UVec2,
    include_previous_state: bool,
    position: Vec3,
) -> RenderFrameExtract {
    let mut extract = particle_motion_blur_taa_base_extract(viewport_size);
    extract.particles.emitters = vec![806];
    extract.particles.sprites = vec![particle_sprite_snapshot(1, position, 0)];
    if include_previous_state {
        extract.particles.previous_sprites = vec![previous_particle_sprite_snapshot(
            1,
            position - Vec3::new(0.5, 0.0, 0.0),
        )];
    }
    extract
}

fn particle_motion_blur_taa_extract_with_keyed_positions(
    viewport_size: UVec2,
    keyed_positions: &[(u64, Vec3)],
) -> RenderFrameExtract {
    let mut extract = particle_motion_blur_taa_base_extract(viewport_size);
    extract.particles.emitters = vec![806];
    extract.particles.sprites = keyed_positions
        .iter()
        .enumerate()
        .map(|(sort_order, (stable_sprite_key, position))| {
            particle_sprite_snapshot(*stable_sprite_key, *position, sort_order as i32)
        })
        .collect();
    extract
}

fn keyed_particle_field_positions(offset: Vec3) -> Vec<(u64, Vec3)> {
    let mut keyed_positions = Vec::new();
    for row in 0..4 {
        for column in 0..8 {
            let key = 1000 + (row * 8 + column) as u64;
            let x = -0.84 + column as f32 * 0.24 + offset.x;
            let y = -0.45 + row as f32 * 0.30 + offset.y;
            keyed_positions.push((key, Vec3::new(x, y, -2.5 + offset.z)));
        }
    }
    keyed_positions
}

fn particle_motion_blur_taa_base_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(806),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: RenderOverlayExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    );
    extract.apply_viewport_size(viewport_size);
    extract.view.anti_alias = AntiAliasSettings::taa();
    extract.post_process.effect_stack.motion_blur = RenderMotionBlurSettings {
        shutter_angle: 0.5,
        samples: 1,
    };
    extract
}

fn particle_sprite_snapshot(
    stable_sprite_key: u64,
    position: Vec3,
    sort_order: i32,
) -> RenderParticleSpriteSnapshot {
    RenderParticleSpriteSnapshot {
        entity: 806,
        stable_sprite_key,
        position,
        size: 1.1,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        sort_order,
        color: Vec4::new(1.0, 0.48, 0.12, 0.85),
        intensity: 1.0,
        material: None,
        texture: None,
    }
}

fn previous_particle_sprite_snapshot(
    stable_sprite_key: u64,
    position: Vec3,
) -> RenderParticlePreviousSpriteSnapshot {
    RenderParticlePreviousSpriteSnapshot {
        entity: 806,
        stable_sprite_key,
        position,
        size: 1.1,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        billboard_basis: None,
    }
}

fn particle_transparent_billboard_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    context
        .require_gpu()?
        .record_particle_billboards_to_resources("scene-color", "scene-depth")
}

fn assert_scene_velocity_readback_nonzero(
    stats: &crate::core::framework::render::RenderStats,
    viewport_size: UVec2,
) {
    let report = stats.last_scene_velocity_readback_report;

    assert!(
        report.available,
        "scene-velocity readback was not available"
    );
    assert_eq!(report.size, viewport_size);
    assert_eq!(
        report.byte_len,
        (viewport_size.x * viewport_size.y * 4) as usize
    );
    assert!(
        report.nonzero_pixel_count > 0,
        "expected particle velocity to write nonzero scene-velocity pixels"
    );
}

fn assert_executor_executed(executor_ids: &[String], executor_id: &str) {
    assert!(
        executor_ids.iter().any(|id| id == executor_id),
        "expected executor `{executor_id}` in executed executor ids: {executor_ids:?}"
    );
}

fn assert_executor_order(executor_ids: &[String], before: &str, after: &str) {
    let before_index = executor_ids
        .iter()
        .position(|executor_id| executor_id == before)
        .unwrap_or_else(|| {
            panic!("expected executor `{before}` in executed executor ids: {executor_ids:?}")
        });
    let after_index = executor_ids
        .iter()
        .position(|executor_id| executor_id == after)
        .unwrap_or_else(|| {
            panic!("expected executor `{after}` in executed executor ids: {executor_ids:?}")
        });
    assert!(
        before_index < after_index,
        "expected executor `{before}` to run before `{after}`, executed executor ids: {executor_ids:?}"
    );
}
