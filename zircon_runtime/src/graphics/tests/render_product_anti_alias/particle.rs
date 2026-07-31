use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderFrameExtract, RenderFramework, RenderLayerSet, RenderMotionBlurSettings,
    RenderParticlePreviousSpriteSnapshot, RenderParticleSpriteSnapshot, RenderViewportDescriptor,
};
use crate::core::math::{UVec2, Vec2, Vec3, Vec4};
use crate::graphics::{
    RenderPassExecutionContext, RenderPassExecutorRegistration, WgpuRenderFramework,
};

use super::super::plugin_render_feature_fixtures::particle_render_feature_descriptor;
use super::{
    TAA_RESOLVE_EXECUTOR_ID, anti_alias_product_profile, assert_executor_order,
    assert_taa_resolve_product_stats, empty_temporal_taa_product_extract, frame_rgba_abs_delta,
    submit_and_capture_anti_alias_product,
};

const PARTICLE_TRANSPARENT_EXECUTOR_ID: &str = "particle.transparent";

#[test]
fn render_product_taa_particle_transparent_pass_contributes_before_resolve() {
    let framework = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [particle_render_feature_descriptor()],
        [RenderPassExecutorRegistration::new(
            PARTICLE_TRANSPARENT_EXECUTOR_ID,
            particle_transparent_billboard_executor,
        )],
        Vec::new(),
    )
    .unwrap();
    let viewport_size = UVec2::new(320, 240);
    let empty_viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let particle_viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    framework
        .set_quality_profile(
            empty_viewport,
            anti_alias_product_profile("runtime-taa-particle-empty", true)
                .with_temporal_history(true)
                .with_particle_rendering(true),
        )
        .unwrap();
    framework
        .set_quality_profile(
            particle_viewport,
            anti_alias_product_profile("runtime-taa-particle-transparent", true)
                .with_temporal_history(true)
                .with_particle_rendering(true),
        )
        .unwrap();

    let (empty_frame, empty_stats) = submit_and_capture_anti_alias_product(
        &framework,
        empty_viewport,
        empty_temporal_taa_product_extract(viewport_size),
    );
    let (particle_frame, particle_stats) = submit_and_capture_anti_alias_product(
        &framework,
        particle_viewport,
        particle_taa_product_extract(viewport_size),
    );

    assert_taa_resolve_product_stats(&empty_stats);
    assert_taa_resolve_product_stats(&particle_stats);
    assert_eq!(empty_stats.last_particle_graph_executed_pass_count, 1);
    assert_eq!(particle_stats.last_particle_graph_executed_pass_count, 1);
    assert_executor_order(
        &empty_stats,
        PARTICLE_TRANSPARENT_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    );
    assert_executor_order(
        &particle_stats,
        PARTICLE_TRANSPARENT_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    );
    assert_eq!(empty_stats.last_particle_velocity_missing_sprite_count, 0);
    assert_eq!(
        particle_stats.last_particle_velocity_missing_sprite_count,
        0
    );
    assert!(
        frame_rgba_abs_delta(&empty_frame, &particle_frame) > 0,
        "particle transparent pass should visibly change the TAA product frame"
    );
}

#[test]
fn render_product_particle_previous_state_suppresses_velocity_gap_stats() {
    let framework = WgpuRenderFramework::new_for_test_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [particle_render_feature_descriptor()],
        [RenderPassExecutorRegistration::new(
            PARTICLE_TRANSPARENT_EXECUTOR_ID,
            particle_transparent_billboard_executor,
        )],
        Vec::new(),
    )
    .unwrap();
    let viewport_size = UVec2::new(320, 240);
    let missing_previous_viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    let previous_state_viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();

    for viewport in [missing_previous_viewport, previous_state_viewport] {
        framework
            .set_quality_profile(
                viewport,
                anti_alias_product_profile("taa-particle-velocity-state", true)
                    .with_temporal_history(true)
                    .with_particle_rendering(true),
            )
            .unwrap();
    }

    framework
        .submit_frame_extract(
            missing_previous_viewport,
            particle_motion_blur_taa_product_extract(viewport_size, false),
        )
        .unwrap();
    let missing_previous_stats = framework.query_stats().unwrap();

    framework
        .submit_frame_extract(
            previous_state_viewport,
            particle_motion_blur_taa_product_extract(viewport_size, true),
        )
        .unwrap();
    let previous_state_stats = framework.query_stats().unwrap();

    assert_taa_resolve_product_stats(&missing_previous_stats);
    assert_taa_resolve_product_stats(&previous_state_stats);
    assert_eq!(
        missing_previous_stats.last_particle_velocity_missing_sprite_count,
        1
    );
    assert_eq!(
        previous_state_stats.last_particle_velocity_missing_sprite_count,
        0
    );
    assert_executor_order(
        &previous_state_stats,
        PARTICLE_TRANSPARENT_EXECUTOR_ID,
        TAA_RESOLVE_EXECUTOR_ID,
    );
}

fn particle_taa_product_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut extract = empty_temporal_taa_product_extract(viewport_size);
    extract.particles.emitters = vec![831];
    extract.particles.sprites = vec![RenderParticleSpriteSnapshot {
        entity: 831,
        stable_sprite_key: 1,
        position: Vec3::new(0.0, 0.0, -2.5),
        size: 1.1,
        aspect_ratio: 1.0,
        billboard_offset: Vec2::ZERO,
        rotation: 0.0,
        sort_order: 0,
        color: Vec4::new(1.0, 0.48, 0.12, 0.85),
        intensity: 1.0,
        depth_test: true,
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
        material: None,
        texture: None,
    }];
    extract
}

fn particle_motion_blur_taa_product_extract(
    viewport_size: UVec2,
    include_previous_state: bool,
) -> RenderFrameExtract {
    let mut extract = particle_taa_product_extract(viewport_size);
    extract.post_process.effect_stack.motion_blur = RenderMotionBlurSettings {
        shutter_angle: 0.5,
        samples: 1,
    };
    if include_previous_state {
        extract.particles.previous_sprites = extract
            .particles
            .sprites
            .iter()
            .map(RenderParticlePreviousSpriteSnapshot::from_current)
            .collect();
    }
    extract
}

fn particle_transparent_billboard_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    context
        .require_gpu()?
        .record_particle_billboards_to_resources("scene-color", "scene-depth")
}
