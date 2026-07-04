use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AntiAliasFallbackReason, AntiAliasMode, AntiAliasSettings, CapturedFrame, FallbackSkyboxKind,
    PostProcessGraphResourceNames, PreviewEnvironmentExtract, RenderCapabilitySummary,
    RenderDynamicResolutionSettings, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderOverlayExtract, RenderParticleSpriteSnapshot, RenderPipelineHandle, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderViewportHandle, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::graphics::scene::anti_alias::fxaa::{FXAA_EXECUTOR_ID, FXAA_PASS_NAME};
use crate::graphics::scene::anti_alias::smaa::{SMAA_EXECUTOR_ID, SMAA_PASS_NAME};
use crate::graphics::{
    RenderPassExecutionContext, RenderPassExecutorRegistration, WgpuRenderFramework,
};

use super::plugin_render_feature_fixtures::particle_render_feature_descriptor;

const PARTICLE_TRANSPARENT_EXECUTOR_ID: &str = "particle.transparent";
const POST_OUTPUT_TRANSFER_EXECUTOR_ID: &str = "post.output-transfer";
const POST_UPSCALE_EXECUTOR_ID: &str = "post.upscale";
const POST_UBER_EXECUTOR_ID: &str = "post.uber";

#[test]
fn render_product_post_terminal_fxaa_changes_final_frame_after_output_transfer() {
    let viewport_size = UVec2::new(160, 120);
    let framework = terminal_product_framework();
    let baseline_viewport = create_terminal_product_viewport_with_anti_alias(
        &framework,
        viewport_size,
        "terminal-fxaa-baseline",
        false,
    );
    let fxaa_viewport =
        create_terminal_product_viewport(&framework, viewport_size, "terminal-fxaa-product");

    let (baseline, baseline_stats) = submit_and_capture_terminal_product(
        &framework,
        baseline_viewport,
        terminal_particle_product_extract(viewport_size, AntiAliasSettings::off(), None),
    );
    let (fxaa_frame, stats) = submit_and_capture_terminal_product(
        &framework,
        fxaa_viewport,
        terminal_particle_product_extract(viewport_size, AntiAliasSettings::auto(), None),
    );

    assert_eq!(baseline.width, viewport_size.x);
    assert_eq!(baseline.height, viewport_size.y);
    assert_eq!(fxaa_frame.width, viewport_size.x);
    assert_eq!(fxaa_frame.height, viewport_size.y);
    assert_eq!(
        baseline_stats.last_anti_alias_fallback.effective_mode,
        AntiAliasMode::Off
    );
    assert_eq!(baseline_stats.last_anti_alias_graph_executed_pass_count, 0);
    assert_graph_executor_not_executed(&baseline_stats, FXAA_EXECUTOR_ID);

    assert_terminal_anti_alias_product_stats(
        &stats,
        AntiAliasMode::Auto,
        AntiAliasMode::Fxaa,
        Some(AntiAliasFallbackReason::AutoResolvedToFxaa),
        FXAA_EXECUTOR_ID,
        FXAA_PASS_NAME,
    );
    assert_post_process_node_executed(&stats, "output-transfer");
    assert_graph_executor_executed(&stats, PARTICLE_TRANSPARENT_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_OUTPUT_TRANSFER_EXECUTOR_ID);
    assert_graph_executor_order(&stats, POST_OUTPUT_TRANSFER_EXECUTOR_ID, FXAA_EXECUTOR_ID);
    assert_eq!(stats.last_frame_target_size, Some(viewport_size));
    assert_eq!(stats.last_frame_render_size, Some(viewport_size));

    let baseline_rgb_sum = frame_rgb_sum(&baseline);
    let frame_delta = frame_rgb_abs_delta(&fxaa_frame, &baseline);
    assert!(
        baseline_rgb_sum > 50_000,
        "terminal AA baseline should contain visible high-contrast product content; rgb_sum={baseline_rgb_sum}"
    );
    assert!(
        frame_delta > 500,
        "FXAA terminal pass should produce a measurable final-frame delta; delta={frame_delta}, baseline_rgb_sum={baseline_rgb_sum}, executors={:?}",
        stats.last_graph_executed_executor_ids
    );
}

#[test]
fn render_product_post_dynamic_resolution_upscale_feeds_smaa_terminal_frame() {
    let viewport_size = UVec2::new(160, 120);
    let internal_size = UVec2::new(80, 60);
    let framework =
        terminal_product_framework_with_capabilities(terminal_product_capabilities(false, true));
    let viewport =
        create_terminal_product_viewport(&framework, viewport_size, "terminal-smaa-upscale");

    let (frame, stats) = submit_and_capture_terminal_product(
        &framework,
        viewport,
        terminal_particle_product_extract(
            viewport_size,
            AntiAliasSettings::auto(),
            Some(RenderDynamicResolutionSettings::fixed_scale(0.5)),
        ),
    );

    assert_eq!(frame.width, viewport_size.x);
    assert_eq!(frame.height, viewport_size.y);
    assert_eq!(stats.last_frame_target_size, Some(viewport_size));
    assert_eq!(
        stats.last_frame_render_size,
        Some(internal_size),
        "dynamic resolution should shrink scene/post-process internals before terminal presentation"
    );
    assert_terminal_anti_alias_product_stats(
        &stats,
        AntiAliasMode::Auto,
        AntiAliasMode::Smaa,
        Some(AntiAliasFallbackReason::AutoResolvedToSmaa),
        SMAA_EXECUTOR_ID,
        SMAA_PASS_NAME,
    );
    assert_post_process_node_executed(&stats, "upscale");
    assert_post_process_node_executed(&stats, "output-transfer");
    assert_graph_executor_executed(&stats, PARTICLE_TRANSPARENT_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_UBER_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_UPSCALE_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_OUTPUT_TRANSFER_EXECUTOR_ID);
    assert_graph_executor_order(&stats, POST_UBER_EXECUTOR_ID, POST_UPSCALE_EXECUTOR_ID);
    assert_graph_executor_order(
        &stats,
        POST_UPSCALE_EXECUTOR_ID,
        POST_OUTPUT_TRANSFER_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_OUTPUT_TRANSFER_EXECUTOR_ID, SMAA_EXECUTOR_ID);
    assert_graph_executor_not_executed(&stats, FXAA_EXECUTOR_ID);
    assert_texture_backing_exists(&stats, PostProcessGraphResourceNames::UPSCALED);
    assert_texture_backing_exists(&stats, PostProcessGraphResourceNames::FINAL_COMPOSITED);

    let frame_rgb_sum = frame_rgb_sum(&frame);
    assert!(
        frame_rgb_sum > 50_000,
        "SMAA/upscale terminal product frame should keep visible content at presentation size; rgb_sum={frame_rgb_sum}"
    );
}

fn terminal_product_framework() -> WgpuRenderFramework {
    terminal_product_framework_with_capabilities(terminal_product_capabilities(true, true))
}

fn terminal_product_framework_with_capabilities(
    capabilities: RenderCapabilitySummary,
) -> WgpuRenderFramework {
    let framework = WgpuRenderFramework::new_with_plugin_render_features(
        Arc::new(ProjectAssetManager::default()),
        [particle_render_feature_descriptor()],
        [RenderPassExecutorRegistration::new(
            PARTICLE_TRANSPARENT_EXECUTOR_ID,
            particle_transparent_billboard_executor,
        )],
        Vec::new(),
    )
    .unwrap();
    framework.override_capabilities_for_tests(capabilities);
    framework
}

fn terminal_product_capabilities(
    supports_fxaa: bool,
    supports_smaa: bool,
) -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: "terminal-aa-product".to_string(),
        supports_offscreen: true,
        supports_fxaa,
        supports_smaa,
        supports_taa: true,
        supports_buffer_readback: true,
        max_supported_msaa_samples: 1,
        ..RenderCapabilitySummary::default()
    }
}

fn create_terminal_product_viewport(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
) -> RenderViewportHandle {
    create_terminal_product_viewport_with_anti_alias(framework, viewport_size, profile_name, true)
}

fn create_terminal_product_viewport_with_anti_alias(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
    anti_alias_enabled: bool,
) -> RenderViewportHandle {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            terminal_product_profile(profile_name, anti_alias_enabled),
        )
        .unwrap();
    viewport
}

fn terminal_product_profile(profile_name: &str, anti_alias_enabled: bool) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_reflection_probes(false)
        .with_baked_lighting(false)
        .with_particle_rendering(true)
        .with_anti_alias(anti_alias_enabled)
}

fn terminal_particle_product_extract(
    viewport_size: UVec2,
    anti_alias: AntiAliasSettings,
    dynamic_resolution: Option<RenderDynamicResolutionSettings>,
) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(941),
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
    extract.view.anti_alias = anti_alias;
    if let Some(dynamic_resolution) = dynamic_resolution {
        extract.view.camera.dynamic_resolution = dynamic_resolution;
        extract.view.sync_selected_descriptor_camera_payload();
    }
    extract.particles.emitters = vec![941];
    extract.particles.sprites = terminal_particle_sprites();
    extract
}

fn terminal_particle_sprites() -> Vec<RenderParticleSpriteSnapshot> {
    let colors = [
        Vec4::new(1.0, 0.03, 0.02, 1.0),
        Vec4::new(0.02, 0.95, 1.0, 1.0),
        Vec4::new(1.0, 0.92, 0.04, 1.0),
        Vec4::new(0.9, 0.04, 1.0, 1.0),
    ];
    let mut sprites = Vec::with_capacity(16);
    let mut stable_sprite_key = 1;
    for row in 0..4 {
        for column in 0..4 {
            let color = colors[(row + column) % colors.len()];
            sprites.push(RenderParticleSpriteSnapshot {
                entity: 941,
                stable_sprite_key,
                position: Vec3::new(-0.75 + column as f32 * 0.5, -0.55 + row as f32 * 0.36, -2.5),
                size: 0.36,
                aspect_ratio: 1.0,
                billboard_offset: Vec2::ZERO,
                rotation: (row as f32 + column as f32) * 0.2,
                sort_order: stable_sprite_key as i32,
                color,
                intensity: 1.0,
                depth_test: false,
                render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
                material: None,
                texture: None,
            });
            stable_sprite_key += 1;
        }
    }
    sprites
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

fn submit_and_capture_terminal_product(
    framework: &WgpuRenderFramework,
    viewport: RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("terminal post-process product frame should be capturable");
    let stats = framework.query_stats().unwrap();
    (frame, stats)
}

fn assert_terminal_anti_alias_product_stats(
    stats: &RenderStats,
    expected_requested_mode: AntiAliasMode,
    expected_mode: AntiAliasMode,
    expected_reason: Option<AntiAliasFallbackReason>,
    expected_executor_id: &str,
    expected_node: &str,
) {
    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_eq!(
        stats.last_anti_alias_fallback.requested_mode,
        expected_requested_mode
    );
    assert_eq!(stats.last_anti_alias_fallback.effective_mode, expected_mode);
    assert_eq!(stats.last_anti_alias_fallback.reason, expected_reason);
    assert_eq!(
        stats.last_anti_alias_graph_executed_pass_count, 1,
        "expected one terminal AA pass; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert_post_process_node_executed(stats, expected_node);
    assert_graph_executor_executed(stats, expected_executor_id);
}

fn assert_post_process_node_executed(stats: &RenderStats, node: &str) {
    assert!(
        stats
            .last_post_process_graph_executed_nodes
            .iter()
            .any(|executed| executed == node),
        "expected post-process node `{node}` to execute; executed={:?}; executors={:?}",
        stats.last_post_process_graph_executed_nodes,
        stats.last_graph_executed_executor_ids
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

fn assert_texture_backing_exists(stats: &RenderStats, resource_name: &str) {
    let _ = stats
        .last_graph_execution_alias_report
        .texture_aliases
        .iter()
        .find(|alias| alias.logical_name == resource_name)
        .unwrap_or_else(|| {
            panic!(
                "missing texture alias for `{resource_name}`; aliases={:?}",
                stats.last_graph_execution_alias_report.texture_aliases
            )
        });
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
