use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, PostProcessVolumeExtract, PreviewEnvironmentExtract,
    RenderFrameExtract, RenderFramework, RenderOverlayExtract,
    RenderPostProcessEffectStackSettings, RenderPostProcessVolumeProfile, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderVignetteSettings, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
    VolumeComponentOverride, VolumeShapeExtract,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::graphics::WgpuRenderFramework;

#[test]
fn render_product_post_volume_camera_transition() {
    let viewport_size = UVec2::new(128, 96);
    let framework = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    let outside_viewport =
        create_volume_product_viewport(&framework, viewport_size, "post-volume-outside");
    let transition_viewport =
        create_volume_product_viewport(&framework, viewport_size, "post-volume-transition");
    let inside_viewport =
        create_volume_product_viewport(&framework, viewport_size, "post-volume-inside");

    let (outside, _) = submit_and_capture_volume_product(
        &framework,
        outside_viewport,
        volume_transition_extract(viewport_size, Vec3::new(6.0, 0.0, 0.0)),
    );
    let (transition, _) = submit_and_capture_volume_product(
        &framework,
        transition_viewport,
        volume_transition_extract(viewport_size, Vec3::new(2.0, 0.0, 0.0)),
    );
    let (inside, stats) = submit_and_capture_volume_product(
        &framework,
        inside_viewport,
        volume_transition_extract(viewport_size, Vec3::ZERO),
    );

    assert_eq!(
        stats.last_post_process_output_transfer_node.as_deref(),
        Some("output-transfer")
    );
    assert_post_process_node_executed(&stats, "uber");
    assert_graph_executor_executed(&stats, "post.uber");
    assert_graph_executor_executed(&stats, "post.output-transfer");
    assert_eq!(
        stats.last_post_process_effect_stack_report.active_families,
        vec!["vignette".to_string()]
    );
    assert!(
        stats
            .last_post_process_effect_stack_report
            .missing_resources
            .is_empty(),
        "volume-driven vignette should not miss post-process resources; report={:?}",
        stats.last_post_process_effect_stack_report
    );

    let corner_origin = UVec2::ZERO;
    let corner_size = UVec2::new(24, 24);
    let outside_corner_luma = average_luma_in_region(&outside, corner_origin, corner_size);
    let transition_corner_luma = average_luma_in_region(&transition, corner_origin, corner_size);
    let inside_corner_luma = average_luma_in_region(&inside, corner_origin, corner_size);

    assert!(
        outside_corner_luma > 40.0,
        "outside-volume frame should keep the preview color visible; luma={outside_corner_luma:.2}"
    );
    assert!(
        transition_corner_luma + 3.0 < outside_corner_luma,
        "transition-volume frame should partially darken the corner; outside={outside_corner_luma:.2}, transition={transition_corner_luma:.2}"
    );
    assert!(
        inside_corner_luma + 3.0 < transition_corner_luma,
        "inside-volume frame should darken the corner more than the transition frame; transition={transition_corner_luma:.2}, inside={inside_corner_luma:.2}"
    );

    let full_delta = frame_rgb_abs_delta(&inside, &outside);
    let transition_delta = frame_rgb_abs_delta(&transition, &outside);
    assert!(
        full_delta > transition_delta,
        "inside-volume product delta should exceed transition delta; full_delta={full_delta}, transition_delta={transition_delta}"
    );
    assert!(
        transition_delta > 2_000,
        "volume transition should produce a measurable final-frame delta; transition_delta={transition_delta}"
    );
}

fn create_volume_product_viewport(
    framework: &WgpuRenderFramework,
    viewport_size: UVec2,
    profile_name: &str,
) -> crate::core::framework::render::RenderViewportHandle {
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    framework
        .set_quality_profile(viewport, volume_product_profile(profile_name))
        .unwrap();
    viewport
}

fn volume_product_profile(profile_name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(profile_name)
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn volume_transition_extract(viewport_size: UVec2, camera_position: Vec3) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(936),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::from_translation(camera_position),
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
                clear_color: Vec4::new(0.68, 0.74, 0.82, 1.0),
            },
            virtual_geometry_debug: None,
        },
    );
    extract.apply_viewport_size(viewport_size);
    let profile = RenderPostProcessVolumeProfile::default().with_effect_stack(
        RenderPostProcessEffectStackSettings {
            vignette: RenderVignetteSettings {
                intensity: 0.75,
                smoothness: 0.45,
                roundness: 1.0,
            },
            ..Default::default()
        },
    );
    extract.post_process.volumes = vec![PostProcessVolumeExtract::new(
        true,
        VolumeShapeExtract::sphere(Vec3::ZERO, 0.0, 4.0),
        0.0,
        1.0,
        extract.view.selected_camera_layers().clone(),
        VolumeComponentOverride::from_profile(&profile),
    )];
    extract
}

fn submit_and_capture_volume_product(
    framework: &WgpuRenderFramework,
    viewport: crate::core::framework::render::RenderViewportHandle,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    framework.submit_frame_extract(viewport, extract).unwrap();
    let frame = framework
        .capture_frame(viewport)
        .unwrap()
        .expect("volume product frame should be capturable");
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

fn average_luma_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> f32 {
    let x_end = origin.x.saturating_add(size.x).min(frame.width) as usize;
    let y_end = origin.y.saturating_add(size.y).min(frame.height) as usize;
    let width = frame.width as usize;
    let mut sum = 0.0;
    let mut count = 0usize;
    for y in origin.y as usize..y_end {
        for x in origin.x as usize..x_end {
            let index = (y * width + x) * 4;
            let r = frame.rgba[index] as f32;
            let g = frame.rgba[index + 1] as f32;
            let b = frame.rgba[index + 2] as f32;
            sum += 0.2126 * r + 0.7152 * g + 0.0722 * b;
            count += 1;
        }
    }
    assert!(count > 0, "sample region should contain pixels");
    sum / count as f32
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
