use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasSettings, FallbackSkyboxKind, PostProcessGraphResourceNames,
    PreviewEnvironmentExtract, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMotionBlurSettings, RenderOverlayExtract, RenderParticlePreviousSpriteSnapshot,
    RenderParticleSpriteSnapshot, RenderPipelineHandle, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportHandle,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::graphics::{
    RenderPassExecutionContext, RenderPassExecutorRegistration, WgpuRenderFramework,
};

use super::super::plugin_render_feature_fixtures::{
    particle_render_feature_descriptor, particle_render_feature_descriptor_with_velocity,
};
use super::{
    assert_graph_executor_executed, assert_post_process_node_executed,
    create_post_process_viewport_with_profile, frame_rgb_abs_delta,
    submit_and_capture_post_process_product,
};

const PARTICLE_TRANSPARENT_EXECUTOR_ID: &str = "particle.transparent";
const PARTICLE_VELOCITY_EXECUTOR_ID: &str = "particle.velocity";
const TEMPORAL_VELOCITY_OBJECT_EXECUTOR_ID: &str = "temporal.velocity-object";
const POST_MOTION_VECTOR_TILE_MAX_EXECUTOR_ID: &str = "post.motion-vector-tile-max";
const POST_MOTION_VECTOR_TILE_MAX_COARSE_EXECUTOR_ID: &str = "post.motion-vector-tile-max-coarse";
const POST_MOTION_VECTOR_NEIGHBOR_MAX_EXECUTOR_ID: &str = "post.motion-vector-neighbor-max";
const POST_MOTION_BLUR_EXECUTOR_ID: &str = "post.motion-blur";

#[test]
fn render_product_post_motion_blur_split_uses_velocity_and_changes_final_frame() {
    let viewport_size = UVec2::new(192, 128);
    let baseline_server = particle_color_product_framework();
    let motion_server = motion_blur_product_framework();
    let baseline_viewport = create_motion_blur_product_viewport(
        &baseline_server,
        viewport_size,
        "post-motion-baseline",
    );
    let motion_viewport =
        create_motion_blur_product_viewport(&motion_server, viewport_size, "post-motion-blur");

    let (baseline, _) = submit_and_capture_post_process_product(
        &baseline_server,
        baseline_viewport,
        motion_blur_particle_product_extract(viewport_size, false),
    );
    motion_server
        .submit_frame_extract(
            motion_viewport,
            motion_blur_particle_product_extract(viewport_size, true),
        )
        .unwrap();
    let (blurred, stats) = submit_and_capture_post_process_product(
        &motion_server,
        motion_viewport,
        motion_blur_particle_product_extract(viewport_size, true),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "motion-blur");
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, TEMPORAL_VELOCITY_OBJECT_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, PARTICLE_VELOCITY_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, PARTICLE_TRANSPARENT_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_MOTION_VECTOR_TILE_MAX_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_MOTION_VECTOR_TILE_MAX_COARSE_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_MOTION_VECTOR_NEIGHBOR_MAX_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_MOTION_BLUR_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");
    assert_graph_executor_order(
        &stats,
        TEMPORAL_VELOCITY_OBJECT_EXECUTOR_ID,
        PARTICLE_VELOCITY_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        PARTICLE_VELOCITY_EXECUTOR_ID,
        POST_MOTION_VECTOR_TILE_MAX_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        POST_MOTION_VECTOR_NEIGHBOR_MAX_EXECUTOR_ID,
        POST_MOTION_BLUR_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_MOTION_BLUR_EXECUTOR_ID, "post.uber");
    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec!["motion-blur".to_string()]
    );
    let effect_stack_report = &stats.last_post_process_effect_stack_report;
    assert!(
        effect_stack_report.missing_resources.is_empty(),
        "motion blur product scene should not miss required resources; report={effect_stack_report:?}"
    );
    assert_scene_velocity_readback_nonzero(&stats, viewport_size);

    let frame_delta = frame_rgb_abs_delta(&blurred, &baseline);
    assert!(
        frame_delta > 5_000,
        "split motion blur should produce a measurable final-frame delta; delta={frame_delta}"
    );
}

fn motion_blur_product_framework() -> WgpuRenderFramework {
    WgpuRenderFramework::new_for_test_with_plugin_render_features(
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

fn particle_color_product_framework() -> WgpuRenderFramework {
    WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [particle_render_feature_descriptor()],
        [RenderPassExecutorRegistration::new(
            PARTICLE_TRANSPARENT_EXECUTOR_ID,
            particle_transparent_billboard_executor,
        )],
        Vec::new(),
    )
    .unwrap()
}

fn create_motion_blur_product_viewport(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
) -> RenderViewportHandle {
    create_post_process_viewport_with_profile(
        server,
        viewport_size,
        motion_blur_product_profile(profile_name),
    )
}

fn motion_blur_product_profile(profile_name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
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

fn motion_blur_particle_product_extract(
    viewport_size: UVec2,
    motion_blur_enabled: bool,
) -> RenderFrameExtract {
    let particles = motion_blur_particle_snapshots();
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(921),
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
            environment: crate::core::framework::render::EnvironmentExtract::default(),
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
    extract.particles.emitters = vec![921];
    extract.particles.sprites = particles.clone();
    if motion_blur_enabled {
        extract.particles.previous_sprites = particles
            .iter()
            .map(motion_blur_previous_particle_snapshot)
            .collect();
        extract.post_process.effect_stack.motion_blur = RenderMotionBlurSettings {
            shutter_angle: 1.0,
            samples: 12,
        };
    }
    extract
}

fn motion_blur_particle_snapshots() -> Vec<RenderParticleSpriteSnapshot> {
    [
        (
            1,
            Vec3::new(-0.45, -0.18, -2.5),
            Vec4::new(1.0, 0.46, 0.12, 0.92),
        ),
        (
            2,
            Vec3::new(0.12, 0.22, -2.5),
            Vec4::new(0.12, 0.82, 1.0, 0.88),
        ),
        (
            3,
            Vec3::new(0.62, -0.04, -2.5),
            Vec4::new(0.95, 0.92, 0.18, 0.9),
        ),
    ]
    .into_iter()
    .map(
        |(stable_sprite_key, position, color)| RenderParticleSpriteSnapshot {
            entity: 921,
            stable_sprite_key,
            position,
            size: 0.74,
            aspect_ratio: 1.0,
            billboard_offset: Vec2::ZERO,
            rotation: 0.0,
            sort_order: stable_sprite_key as i32,
            color,
            intensity: 1.0,
            depth_test: true,
            render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
            material: None,
            texture: None,
        },
    )
    .collect()
}

fn motion_blur_previous_particle_snapshot(
    sprite: &RenderParticleSpriteSnapshot,
) -> RenderParticlePreviousSpriteSnapshot {
    RenderParticlePreviousSpriteSnapshot {
        entity: sprite.entity,
        stable_sprite_key: sprite.stable_sprite_key,
        position: sprite.position - Vec3::new(0.82, 0.0, 0.0),
        size: sprite.size,
        aspect_ratio: sprite.aspect_ratio,
        billboard_offset: sprite.billboard_offset,
        rotation: sprite.rotation,
        billboard_basis: None,
    }
}

fn particle_transparent_billboard_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    context
        .require_gpu()?
        .record_particle_billboards_to_resources(
            PostProcessGraphResourceNames::SCENE_COLOR,
            PostProcessGraphResourceNames::SCENE_DEPTH,
        )
}

fn assert_graph_executor_order(stats: &RenderStats, before: &str, after: &str) {
    let before_index = stats
        .last_graph_executed_executor_ids
        .iter()
        .position(|executor_id| executor_id == before)
        .unwrap_or_else(|| {
            panic!(
                "expected executor `{before}` in executed executor ids: {:?}",
                stats.last_graph_executed_executor_ids
            )
        });
    let after_index = stats
        .last_graph_executed_executor_ids
        .iter()
        .position(|executor_id| executor_id == after)
        .unwrap_or_else(|| {
            panic!(
                "expected executor `{after}` in executed executor ids: {:?}",
                stats.last_graph_executed_executor_ids
            )
        });
    assert!(
        before_index < after_index,
        "expected executor `{before}` to run before `{after}`; executed={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_scene_velocity_readback_nonzero(stats: &RenderStats, viewport_size: UVec2) {
    let report = stats.last_scene_velocity_readback_report;

    assert!(
        report.available,
        "scene-velocity readback should be available"
    );
    assert_eq!(report.size, viewport_size);
    assert_eq!(
        report.byte_len,
        (viewport_size.x * viewport_size.y * 4) as usize
    );
    assert!(
        report.nonzero_pixel_count > 0,
        "motion blur product scene should write nonzero scene-velocity pixels"
    );
}
