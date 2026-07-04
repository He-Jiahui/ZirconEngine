use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, PostProcessGraphResourceNames, PreviewEnvironmentExtract,
    RenderBlurSettings, RenderFrameExtract, RenderFramework, RenderLayerSet, RenderOverlayExtract,
    RenderParticleSpriteSnapshot, RenderPipelineHandle, RenderPostProcessEffectStackSettings,
    RenderQualityProfile, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats,
    RenderViewportDescriptor, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::math::{Transform, UVec2, Vec2, Vec3, Vec4};
use crate::graphics::WgpuRenderFramework;
use crate::graphics::{RenderPassExecutionContext, RenderPassExecutorRegistration};

use super::plugin_render_feature_fixtures::particle_render_feature_descriptor;

const PARTICLE_TRANSPARENT_EXECUTOR_ID: &str = "particle.transparent";
const POST_BLUR_EXECUTOR_ID: &str = "post.blur";

#[test]
fn render_product_post_blur_split_changes_final_frame() {
    let viewport_size = UVec2::new(160, 120);
    let framework = blur_product_framework();
    let baseline_viewport =
        create_blur_product_viewport(&framework, viewport_size, "post-blur-baseline");
    let blur_viewport = create_blur_product_viewport(&framework, viewport_size, "post-blur-split");

    let (baseline, _) = submit_and_capture_blur_product(
        &framework,
        baseline_viewport,
        blur_particle_product_extract(viewport_size, false),
    );
    let (blurred, stats) = submit_and_capture_blur_product(
        &framework,
        blur_viewport,
        blur_particle_product_extract(viewport_size, true),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "blur");
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, PARTICLE_TRANSPARENT_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, POST_BLUR_EXECUTOR_ID);
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");
    assert_graph_executor_order(
        &stats,
        PARTICLE_TRANSPARENT_EXECUTOR_ID,
        POST_BLUR_EXECUTOR_ID,
    );
    assert_graph_executor_order(&stats, POST_BLUR_EXECUTOR_ID, "post.uber");
    assert_graph_executor_order(&stats, "post.uber", "post.output-transfer");
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec!["blur".to_string()]
    );
    let effect_stack_report = &stats.last_post_process_effect_stack_report;
    assert!(
        effect_stack_report.missing_resources.is_empty(),
        "blur product scene should not miss required resources; report={effect_stack_report:?}"
    );
    assert_texture_backings_are_distinct(
        &stats,
        PostProcessGraphResourceNames::BLURRED,
        PostProcessGraphResourceNames::TONEMAPPED,
    );

    let baseline_rgb_sum = frame_rgb_sum(&baseline);
    assert!(
        baseline_rgb_sum > 50_000,
        "blur product baseline should contain visible high-contrast scene content; rgb_sum={baseline_rgb_sum}"
    );

    let frame_delta = frame_rgb_abs_delta(&blurred, &baseline);
    let blurred_rgb_sum = frame_rgb_sum(&blurred);
    assert!(
        frame_delta > 5_000,
        "split blur should produce a measurable final-frame delta; delta={frame_delta}, baseline_rgb_sum={baseline_rgb_sum}, blurred_rgb_sum={blurred_rgb_sum}, aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

fn blur_product_framework() -> WgpuRenderFramework {
    WgpuRenderFramework::new_with_plugin_render_features(
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

fn create_blur_product_viewport(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
) -> crate::core::framework::render::RenderViewportHandle {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, blur_product_profile(profile_name))
        .unwrap();
    viewport
}

fn blur_product_profile(profile_name: &str) -> RenderQualityProfile {
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
        .with_anti_alias(false)
}

fn blur_particle_product_extract(viewport_size: UVec2, blur_enabled: bool) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(922),
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
    extract.particles.emitters = vec![922];
    extract.particles.sprites = blur_particle_sprites();
    if blur_enabled {
        extract.post_process.effect_stack = RenderPostProcessEffectStackSettings {
            blur: RenderBlurSettings { radius: 12.0 },
            ..Default::default()
        };
    }
    extract
}

fn blur_particle_sprites() -> Vec<RenderParticleSpriteSnapshot> {
    let colors = [
        Vec4::new(1.0, 0.04, 0.02, 1.0),
        Vec4::new(0.02, 0.95, 1.0, 1.0),
        Vec4::new(1.0, 0.92, 0.04, 1.0),
        Vec4::new(0.92, 0.04, 1.0, 1.0),
    ];
    let mut sprites = Vec::with_capacity(12);
    let mut stable_sprite_key = 1;
    for row in 0..3 {
        for column in 0..4 {
            let x = -0.72 + column as f32 * 0.48;
            let y = -0.36 + row as f32 * 0.36;
            let color = colors[(row + column) % colors.len()];
            sprites.push(RenderParticleSpriteSnapshot {
                entity: 922,
                stable_sprite_key,
                position: Vec3::new(x, y, -2.5),
                size: 0.42,
                aspect_ratio: 1.0,
                billboard_offset: Vec2::ZERO,
                rotation: 0.0,
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

fn submit_and_capture_blur_product(
    framework: &WgpuRenderFramework,
    viewport: crate::core::framework::render::RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("blur product frame should be capturable");
    let stats = framework.query_stats().unwrap();
    (frame, stats)
}

fn assert_post_process_node_executed(stats: &RenderStats, node: &str) {
    assert!(
        stats
            .last_post_process_graph_executed_nodes
            .iter()
            .any(|executed| executed == node),
        "expected post-process node `{node}` to execute; executed={:?}",
        stats.last_post_process_graph_executed_nodes
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

fn assert_texture_backings_are_distinct(stats: &RenderStats, left: &str, right: &str) {
    let left_backing = texture_backing_name(stats, left);
    let right_backing = texture_backing_name(stats, right);
    assert_ne!(
        left_backing, right_backing,
        "texture resources `{left}` and `{right}` should not share backing; aliases={:?}",
        stats.last_graph_execution_alias_report.texture_aliases
    );
}

fn texture_backing_name<'a>(stats: &'a RenderStats, logical_name: &str) -> &'a str {
    stats
        .last_graph_execution_alias_report
        .texture_aliases
        .iter()
        .find(|record| record.logical_name == logical_name)
        .unwrap_or_else(|| {
            panic!(
                "expected texture alias report to include `{logical_name}`; aliases={:?}",
                stats.last_graph_execution_alias_report.texture_aliases
            )
        })
        .backing_name
        .as_str()
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
