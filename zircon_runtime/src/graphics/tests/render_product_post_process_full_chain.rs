use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasFallbackReason, AntiAliasMode, CapturedFrame, PostProcessGraphResourceNames,
    RenderCapabilitySummary, RenderFrameExtract, RenderFramework, RenderStats,
    RenderViewportDescriptor, RenderViewportHandle,
};
use crate::core::math::UVec2;
use crate::graphics::scene::anti_alias::fxaa::FXAA_EXECUTOR_ID;
use crate::graphics::scene::anti_alias::smaa::{SMAA_EXECUTOR_ID, SMAA_PASS_NAME};
use crate::graphics::{
    RenderPassExecutionContext, RenderPassExecutorRegistration, WgpuRenderFramework,
};

use super::plugin_render_feature_fixtures::particle_render_feature_descriptor_with_velocity;
use fixture::{
    full_chain_material, full_chain_product_extract, full_chain_product_profile,
    insert_user_lut_texture, register_full_chain_material,
};

#[path = "render_product_post_process_full_chain/fixture.rs"]
mod fixture;

const PARTICLE_TRANSPARENT_EXECUTOR_ID: &str = "particle.transparent";
const PARTICLE_VELOCITY_EXECUTOR_ID: &str = "particle.velocity";
const TEMPORAL_VELOCITY_OBJECT_EXECUTOR_ID: &str = "temporal.velocity-object";
const POST_BLOOM_EXTRACT_EXECUTOR_ID: &str = "post.bloom-extract";
const POST_EXPOSURE_HISTOGRAM_EXECUTOR_ID: &str = "post.exposure.histogram";
const POST_EXPOSURE_RESOLVE_EXECUTOR_ID: &str = "post.exposure.resolve";
const POST_DOF_PREPARE_EXECUTOR_ID: &str = "post.depth-of-field-prepare";
const POST_DOF_EXECUTOR_ID: &str = "post.depth-of-field";
const POST_MOTION_VECTOR_TILE_MAX_EXECUTOR_ID: &str = "post.motion-vector-tile-max";
const POST_MOTION_VECTOR_TILE_MAX_COARSE_EXECUTOR_ID: &str = "post.motion-vector-tile-max-coarse";
const POST_MOTION_VECTOR_NEIGHBOR_MAX_EXECUTOR_ID: &str = "post.motion-vector-neighbor-max";
const POST_MOTION_BLUR_EXECUTOR_ID: &str = "post.motion-blur";
const POST_SSR_REFLECTION_PYRAMID_EXECUTOR_ID: &str =
    "post.screen-space-reflection-reflection-pyramid";
const POST_SSR_REFLECTION_PYRAMID_COARSE_EXECUTOR_ID: &str =
    "post.screen-space-reflection-reflection-pyramid-coarse";
const POST_SSR_SPECULAR_OCCLUSION_EXECUTOR_ID: &str =
    "post.screen-space-reflection-specular-occlusion";
const POST_SSR_RESOLVE_EXECUTOR_ID: &str = "post.screen-space-reflection-resolve";
const POST_SCENE_COMPOSITE_EXECUTOR_ID: &str = "post.scene-composite";
const POST_BLUR_EXECUTOR_ID: &str = "post.blur";
const POST_COLOR_LUT_BAKE_EXECUTOR_ID: &str = "post.color-lut-bake";
const POST_UBER_EXECUTOR_ID: &str = "post.uber";
const POST_UPSCALE_EXECUTOR_ID: &str = "post.upscale";
const POST_OUTPUT_TRANSFER_EXECUTOR_ID: &str = "post.output-transfer";

#[test]
fn render_product_post_full_chain_all_effects_on() {
    let viewport_size = UVec2::new(160, 120);
    let internal_size = UVec2::new(80, 60);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_material = register_full_chain_material(
        asset_manager.as_ref(),
        "res://materials/post_full_chain_receiver.zmaterial",
        full_chain_material(
            "PostFullChainReceiver",
            [0.03, 0.04, 0.055, 1.0],
            1.0,
            0.04,
            [0.0, 0.0, 0.0],
            false,
            true,
        ),
    );
    let caster_material = register_full_chain_material(
        asset_manager.as_ref(),
        "res://materials/post_full_chain_caster.zmaterial",
        full_chain_material(
            "PostFullChainCaster",
            [1.0, 0.18, 0.04, 1.0],
            0.0,
            0.22,
            [4.2, 0.32, 0.10],
            false,
            false,
        ),
    );
    let user_lut = insert_user_lut_texture(
        asset_manager.as_ref(),
        "res://textures/post_full_chain_lut.png",
    );
    let baseline_framework = full_chain_product_framework(asset_manager.clone());
    let full_framework = full_chain_product_framework(asset_manager);
    let baseline_viewport = create_full_chain_product_viewport(
        &baseline_framework,
        viewport_size,
        "post-full-chain-baseline",
        false,
    );
    let full_viewport = create_full_chain_product_viewport(
        &full_framework,
        viewport_size,
        "post-full-chain-all",
        true,
    );

    let (baseline, baseline_stats) = submit_and_capture_full_chain_product(
        &baseline_framework,
        baseline_viewport,
        full_chain_product_extract(
            viewport_size,
            receiver_material,
            caster_material,
            user_lut,
            false,
        ),
    );
    full_framework
        .submit_frame_extract(
            full_viewport,
            full_chain_product_extract(
                viewport_size,
                receiver_material,
                caster_material,
                user_lut,
                true,
            ),
        )
        .unwrap();
    let (full_frame, stats) = submit_and_capture_full_chain_product(
        &full_framework,
        full_viewport,
        full_chain_product_extract(
            viewport_size,
            receiver_material,
            caster_material,
            user_lut,
            true,
        ),
    );

    assert_eq!(baseline.width, viewport_size.x);
    assert_eq!(baseline.height, viewport_size.y);
    assert_eq!(full_frame.width, viewport_size.x);
    assert_eq!(full_frame.height, viewport_size.y);
    assert_eq!(baseline_stats.last_frame_target_size, Some(viewport_size));
    assert_eq!(baseline_stats.last_frame_render_size, Some(viewport_size));
    assert_eq!(stats.last_frame_target_size, Some(viewport_size));
    assert_eq!(stats.last_frame_render_size, Some(internal_size));
    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_eq!(
        stats.last_anti_alias_fallback.requested_mode,
        AntiAliasMode::Auto
    );
    assert_eq!(
        stats.last_anti_alias_fallback.effective_mode,
        AntiAliasMode::Smaa
    );
    assert_eq!(
        stats.last_anti_alias_fallback.reason,
        Some(AntiAliasFallbackReason::AutoResolvedToSmaa)
    );
    assert_eq!(stats.last_anti_alias_graph_executed_pass_count, 1);

    for node in [
        "bloom",
        "exposure-histogram",
        "exposure-resolve",
        "depth-of-field",
        "motion-blur",
        "screen-space-reflection-reflection-pyramid",
        "screen-space-reflection-reflection-pyramid-coarse",
        "screen-space-reflection-specular-occlusion",
        "screen-space-reflection-resolve",
        "scene-composite",
        "blur",
        "color-lut-bake",
        "uber",
        "upscale",
        "output-transfer",
        SMAA_PASS_NAME,
    ] {
        assert_post_process_node_executed(&stats, node);
    }

    for executor_id in [
        TEMPORAL_VELOCITY_OBJECT_EXECUTOR_ID,
        PARTICLE_VELOCITY_EXECUTOR_ID,
        PARTICLE_TRANSPARENT_EXECUTOR_ID,
        POST_BLOOM_EXTRACT_EXECUTOR_ID,
        POST_EXPOSURE_HISTOGRAM_EXECUTOR_ID,
        POST_EXPOSURE_RESOLVE_EXECUTOR_ID,
        POST_DOF_PREPARE_EXECUTOR_ID,
        POST_DOF_EXECUTOR_ID,
        POST_MOTION_VECTOR_TILE_MAX_EXECUTOR_ID,
        POST_MOTION_VECTOR_TILE_MAX_COARSE_EXECUTOR_ID,
        POST_MOTION_VECTOR_NEIGHBOR_MAX_EXECUTOR_ID,
        POST_MOTION_BLUR_EXECUTOR_ID,
        POST_SSR_REFLECTION_PYRAMID_EXECUTOR_ID,
        POST_SSR_REFLECTION_PYRAMID_COARSE_EXECUTOR_ID,
        POST_SSR_SPECULAR_OCCLUSION_EXECUTOR_ID,
        POST_SSR_RESOLVE_EXECUTOR_ID,
        POST_SCENE_COMPOSITE_EXECUTOR_ID,
        POST_BLUR_EXECUTOR_ID,
        POST_COLOR_LUT_BAKE_EXECUTOR_ID,
        POST_UBER_EXECUTOR_ID,
        POST_UPSCALE_EXECUTOR_ID,
        POST_OUTPUT_TRANSFER_EXECUTOR_ID,
        SMAA_EXECUTOR_ID,
    ] {
        assert_graph_executor_executed(&stats, executor_id);
    }
    assert_graph_executor_not_executed(&stats, FXAA_EXECUTOR_ID);

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
    assert_graph_executor_order(&stats, POST_DOF_PREPARE_EXECUTOR_ID, POST_DOF_EXECUTOR_ID);
    assert_graph_executor_order(
        &stats,
        POST_MOTION_VECTOR_NEIGHBOR_MAX_EXECUTOR_ID,
        POST_MOTION_BLUR_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_DOF_EXECUTOR_ID, POST_MOTION_BLUR_EXECUTOR_ID);
    assert_graph_executor_order(
        &stats,
        POST_MOTION_BLUR_EXECUTOR_ID,
        POST_BLOOM_EXTRACT_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        POST_BLOOM_EXTRACT_EXECUTOR_ID,
        POST_EXPOSURE_HISTOGRAM_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        POST_EXPOSURE_HISTOGRAM_EXECUTOR_ID,
        POST_EXPOSURE_RESOLVE_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        POST_SSR_RESOLVE_EXECUTOR_ID,
        POST_SCENE_COMPOSITE_EXECUTOR_ID,
    );
    assert_graph_executor_order(
        &stats,
        POST_SCENE_COMPOSITE_EXECUTOR_ID,
        POST_BLUR_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_BLUR_EXECUTOR_ID, POST_UBER_EXECUTOR_ID);
    assert_graph_executor_order(
        &stats,
        POST_COLOR_LUT_BAKE_EXECUTOR_ID,
        POST_UBER_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_UBER_EXECUTOR_ID, POST_UPSCALE_EXECUTOR_ID);
    assert_graph_executor_order(
        &stats,
        POST_UPSCALE_EXECUTOR_ID,
        POST_OUTPUT_TRANSFER_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_OUTPUT_TRANSFER_EXECUTOR_ID, SMAA_EXECUTOR_ID);

    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec![
            "tonemap".to_string(),
            "lut".to_string(),
            "blur".to_string(),
            "depth-of-field".to_string(),
            "motion-blur".to_string(),
            "screen-space-reflection".to_string(),
            "vignette".to_string(),
            "film-grain".to_string(),
            "dither".to_string(),
            "chromatic-aberration".to_string(),
            "fog".to_string(),
        ]
    );
    let effect_stack_report = &stats.last_post_process_effect_stack_report;
    assert!(
        effect_stack_report.missing_resources.is_empty(),
        "full-chain post-process scene should not miss resources; report={effect_stack_report:?}"
    );
    assert_eq!(stats.last_particle_velocity_missing_sprite_count, 0);
    assert_scene_velocity_readback_nonzero(&stats, internal_size);

    for resource_name in [
        PostProcessGraphResourceNames::BLOOM,
        PostProcessGraphResourceNames::EXPOSURE_HISTOGRAM,
        PostProcessGraphResourceNames::EXPOSURE_CURRENT,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_COC,
        PostProcessGraphResourceNames::DEPTH_OF_FIELD_BOKEH,
        PostProcessGraphResourceNames::DEPTH_OF_FIELDED,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX,
        PostProcessGraphResourceNames::MOTION_VECTOR_TILE_MAX_COARSE,
        PostProcessGraphResourceNames::MOTION_VECTOR_NEIGHBOR_MAX,
        PostProcessGraphResourceNames::MOTION_BLURRED,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION,
        PostProcessGraphResourceNames::SCREEN_SPACE_REFLECTION_HISTORY,
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        PostProcessGraphResourceNames::BLURRED,
        PostProcessGraphResourceNames::COLOR_LUT,
        PostProcessGraphResourceNames::EFFECT_STACKED,
        PostProcessGraphResourceNames::TONEMAPPED,
        PostProcessGraphResourceNames::UPSCALED,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
    ] {
        assert_texture_backing_exists(&stats, resource_name);
    }
    assert_texture_backings_are_distinct(
        &stats,
        PostProcessGraphResourceNames::SCENE_COMPOSITED,
        PostProcessGraphResourceNames::TONEMAPPED,
    );
    assert_texture_backings_are_distinct(
        &stats,
        PostProcessGraphResourceNames::UPSCALED,
        PostProcessGraphResourceNames::FINAL_COMPOSITED,
    );

    let baseline_rgb_sum = frame_rgb_sum(&baseline);
    let full_rgb_sum = frame_rgb_sum(&full_frame);
    let frame_delta = frame_rgb_abs_delta(&full_frame, &baseline);
    assert!(
        baseline_rgb_sum > 50_000,
        "full-chain baseline should contain visible product scene content; rgb_sum={baseline_rgb_sum}"
    );
    assert!(
        full_rgb_sum > 20_000,
        "full-chain effects should preserve visible terminal content; rgb_sum={full_rgb_sum}"
    );
    assert!(
        frame_delta > 15_000,
        "all-effects chain should produce a measurable final-frame delta; delta={frame_delta}, baseline_rgb_sum={baseline_rgb_sum}, full_rgb_sum={full_rgb_sum}, aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

fn full_chain_product_framework(asset_manager: Arc<ProjectAssetManager>) -> WgpuRenderFramework {
    let framework = WgpuRenderFramework::new_with_plugin_render_features(
        asset_manager,
        [particle_render_feature_descriptor_with_velocity()],
        [RenderPassExecutorRegistration::new(
            PARTICLE_TRANSPARENT_EXECUTOR_ID,
            particle_transparent_billboard_executor,
        )],
        Vec::new(),
    )
    .unwrap();
    framework.override_capabilities_for_tests(full_chain_product_capabilities());
    framework
}

fn full_chain_product_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: "post-full-chain-product".to_string(),
        supports_offscreen: true,
        supports_fxaa: false,
        supports_smaa: true,
        supports_taa: true,
        supports_buffer_readback: true,
        max_supported_msaa_samples: 1,
        ..RenderCapabilitySummary::default()
    }
}

fn create_full_chain_product_viewport(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
    full_chain_enabled: bool,
) -> RenderViewportHandle {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            full_chain_product_profile(profile_name, full_chain_enabled),
        )
        .unwrap();
    viewport
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

fn submit_and_capture_full_chain_product(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("full-chain post-process product frame should be capturable");
    let stats = framework.query_stats().unwrap();
    (frame, stats)
}

fn assert_post_process_node_executed(stats: &RenderStats, node: &str) {
    assert!(
        stats
            .last_post_process_graph_executed_nodes
            .iter()
            .any(|executed| executed == node),
        "expected post-process node `{node}` to execute; executed={:?}; executors={:?}; effect_stack_report={:?}",
        stats.last_post_process_graph_executed_nodes,
        stats.last_graph_executed_executor_ids,
        stats.last_post_process_effect_stack_report
    );
}

fn assert_graph_executor_executed(stats: &RenderStats, executor_id: &str) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executed| executed == executor_id),
        "expected graph executor `{executor_id}` to execute; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_graph_executor_not_executed(stats: &RenderStats, executor_id: &str) {
    assert!(
        !stats
            .last_graph_executed_executor_ids
            .iter()
            .any(|executed| executed == executor_id),
        "graph executor `{executor_id}` should not execute; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_graph_executor_order(stats: &RenderStats, before: &str, after: &str) {
    let before_index = graph_executor_index(stats, before);
    let after_index = graph_executor_index(stats, after);
    assert!(
        before_index < after_index,
        "expected graph executor `{before}` before `{after}`; executed={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn graph_executor_index(stats: &RenderStats, executor_id: &str) -> usize {
    stats
        .last_graph_executed_executor_ids
        .iter()
        .position(|executed| executed == executor_id)
        .unwrap_or_else(|| {
            panic!(
                "graph executor `{executor_id}` was not executed; executed={:?}",
                stats.last_graph_executed_executor_ids
            )
        })
}

fn assert_scene_velocity_readback_nonzero(stats: &RenderStats, render_size: UVec2) {
    let report = stats.last_scene_velocity_readback_report;
    assert!(
        report.available,
        "scene-velocity readback should be available"
    );
    assert_eq!(report.size, render_size);
    assert_eq!(
        report.byte_len,
        (render_size.x * render_size.y * 4) as usize
    );
    assert!(
        report.nonzero_pixel_count > 0,
        "full-chain product scene should write nonzero scene-velocity pixels"
    );
}

fn assert_texture_backings_are_distinct(stats: &RenderStats, first: &str, second: &str) {
    let first_backing = texture_backing_for(stats, first);
    let second_backing = texture_backing_for(stats, second);
    assert_ne!(
        first_backing, second_backing,
        "expected `{first}` and `{second}` to use distinct texture backings; aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

fn assert_texture_backing_exists(stats: &RenderStats, resource_name: &str) {
    let _ = texture_backing_for(stats, resource_name);
}

fn texture_backing_for<'a>(stats: &'a RenderStats, resource_name: &str) -> &'a str {
    stats
        .last_graph_execution_alias_report
        .texture_aliases
        .iter()
        .find(|alias| alias.logical_name == resource_name)
        .map(|alias| alias.backing_name.as_str())
        .unwrap_or_else(|| {
            panic!(
                "missing texture alias for `{resource_name}`; aliases={:?}",
                stats.last_graph_execution_alias_report.texture_aliases
            )
        })
}

fn frame_rgb_abs_delta(left: &CapturedFrame, right: &CapturedFrame) -> u64 {
    assert_eq!(left.width, right.width);
    assert_eq!(left.height, right.height);
    assert_eq!(left.rgba.len(), right.rgba.len());
    left.rgba
        .chunks_exact(4)
        .zip(right.rgba.chunks_exact(4))
        .map(|(left, right)| {
            left[0].abs_diff(right[0]) as u64
                + left[1].abs_diff(right[1]) as u64
                + left[2].abs_diff(right[2]) as u64
        })
        .sum()
}

fn frame_rgb_sum(frame: &CapturedFrame) -> u64 {
    frame
        .rgba
        .chunks_exact(4)
        .map(|pixel| pixel[0] as u64 + pixel[1] as u64 + pixel[2] as u64)
        .sum()
}
