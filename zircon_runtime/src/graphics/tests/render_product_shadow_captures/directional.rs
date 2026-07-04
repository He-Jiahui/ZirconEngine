use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, LightShadowSettings, PreviewEnvironmentExtract,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderPipelineHandle, RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats,
    RenderViewportDescriptor, RenderWorldSnapshotHandle, ShadowPcfQuality, ShadowResolutionTier,
    ViewportCameraSnapshot, DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::ResourceId;
use crate::graphics::WgpuRenderFramework;

use super::{
    assert_directional_shadow_capture_stats, average_luma_in_region,
    directional_shadow_capture_profile, frame_darkened_pixel_count_and_luma_delta,
    frame_rgb_abs_delta, register_shadow_capture_material, shadow_capture_mesh,
    shadow_capture_settings_with_quality,
};

#[test]
fn render_product_directional_shadow_atlas_capture_records_receiver_path() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_shadow_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_capture_receiver.zmaterial",
        "ShadowCaptureReceiver",
        [0.28, 0.30, 0.32, 1.0],
        false,
        true,
    );
    let caster_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_capture_caster.zmaterial",
        "ShadowCaptureCaster",
        [0.44, 0.44, 0.42, 1.0],
        true,
        false,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let (frame, stats) = render_directional_shadow_capture_frame(
        &server,
        viewport_size,
        receiver_shadow_material,
        caster_material,
        "directional-shadow-receiver",
    );

    assert_directional_shadow_capture_stats("shadow receiver", &stats);

    let sample_origin = UVec2::new(78, 48);
    let sample_size = UVec2::new(20, 24);
    let receiver_luma = average_luma_in_region(&frame, sample_origin, sample_size);
    assert!(
        receiver_luma > 8.0,
        "directional shadow product capture should contain a visible receiver sample; receiver_luma={receiver_luma:.2}"
    );
}

#[test]
fn render_product_directional_shadow_atlas_darkens_receiver_capture() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_shadow_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_capture_receiver_on.zmaterial",
        "ShadowCaptureReceiverOn",
        [0.28, 0.30, 0.32, 1.0],
        false,
        true,
    );
    let receiver_unshadowed_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_capture_receiver_off.zmaterial",
        "ShadowCaptureReceiverOff",
        [0.28, 0.30, 0.32, 1.0],
        false,
        false,
    );
    let caster_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_capture_caster_compare.zmaterial",
        "ShadowCaptureCasterCompare",
        [0.44, 0.44, 0.42, 1.0],
        true,
        false,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let (shadowed_frame, shadowed_stats) = render_directional_shadow_capture_frame(
        &server,
        viewport_size,
        receiver_shadow_material,
        caster_material,
        "directional-shadow-receiver-on",
    );
    let (unshadowed_frame, unshadowed_stats) = render_directional_shadow_capture_frame(
        &server,
        viewport_size,
        receiver_unshadowed_material,
        caster_material,
        "directional-shadow-receiver-off",
    );

    assert_directional_shadow_capture_stats("shadowed receiver", &shadowed_stats);
    assert_directional_shadow_capture_stats("unshadowed receiver", &unshadowed_stats);

    let sample_origin = UVec2::new(78, 48);
    let sample_size = UVec2::new(20, 24);
    let shadowed_luma = average_luma_in_region(&shadowed_frame, sample_origin, sample_size);
    let unshadowed_luma = average_luma_in_region(&unshadowed_frame, sample_origin, sample_size);
    let frame_delta = frame_rgb_abs_delta(&shadowed_frame, &unshadowed_frame);
    assert!(
        shadowed_luma + 4.0 < unshadowed_luma,
        "directional shadow receiver should darken the sample region; shadowed_luma={shadowed_luma:.2} unshadowed_luma={unshadowed_luma:.2} frame_delta={frame_delta}"
    );
}

#[test]
fn render_product_csm_directional_remains_stable_under_subtexel_camera_shift() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_shadow_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/csm_stable_receiver_on.zmaterial",
        "CsmStableReceiverOn",
        [0.28, 0.30, 0.32, 1.0],
        false,
        true,
    );
    let receiver_unshadowed_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/csm_stable_receiver_off.zmaterial",
        "CsmStableReceiverOff",
        [0.28, 0.30, 0.32, 1.0],
        false,
        false,
    );
    let caster_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/csm_stable_caster.zmaterial",
        "CsmStableCaster",
        [0.44, 0.44, 0.42, 1.0],
        true,
        false,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let baseline_offset = Vec3::ZERO;
    let shifted_offset = Vec3::new(0.006, 0.0, 0.0);
    let (baseline_shadowed, baseline_shadowed_stats) =
        render_directional_shadow_capture_frame_with_camera_offset(
            &server,
            viewport_size,
            receiver_shadow_material,
            caster_material,
            "csm-stable-baseline-shadowed",
            baseline_offset,
        );
    let (baseline_unshadowed, baseline_unshadowed_stats) =
        render_directional_shadow_capture_frame_with_camera_offset(
            &server,
            viewport_size,
            receiver_unshadowed_material,
            caster_material,
            "csm-stable-baseline-unshadowed",
            baseline_offset,
        );
    let (shifted_shadowed, shifted_shadowed_stats) =
        render_directional_shadow_capture_frame_with_camera_offset(
            &server,
            viewport_size,
            receiver_shadow_material,
            caster_material,
            "csm-stable-shifted-shadowed",
            shifted_offset,
        );
    let (shifted_unshadowed, shifted_unshadowed_stats) =
        render_directional_shadow_capture_frame_with_camera_offset(
            &server,
            viewport_size,
            receiver_unshadowed_material,
            caster_material,
            "csm-stable-shifted-unshadowed",
            shifted_offset,
        );

    assert_directional_shadow_capture_stats("baseline shadowed receiver", &baseline_shadowed_stats);
    assert_directional_shadow_capture_stats(
        "baseline unshadowed receiver",
        &baseline_unshadowed_stats,
    );
    assert_directional_shadow_capture_stats("shifted shadowed receiver", &shifted_shadowed_stats);
    assert_directional_shadow_capture_stats(
        "shifted unshadowed receiver",
        &shifted_unshadowed_stats,
    );

    let baseline_darkening =
        frame_darkened_pixel_count_and_luma_delta(&baseline_shadowed, &baseline_unshadowed);
    let shifted_darkening =
        frame_darkened_pixel_count_and_luma_delta(&shifted_shadowed, &shifted_unshadowed);
    assert_darkening_stats_close(
        "CSM subtexel camera shift",
        baseline_darkening,
        shifted_darkening,
    );
}

#[test]
fn render_product_directional_shadow_atlas_forward_deferred_darkening_parity() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_shadow_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_parity_receiver_on.zmaterial",
        "ShadowParityReceiverOn",
        [0.28, 0.30, 0.32, 1.0],
        false,
        true,
    );
    let receiver_unshadowed_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_parity_receiver_off.zmaterial",
        "ShadowParityReceiverOff",
        [0.28, 0.30, 0.32, 1.0],
        false,
        false,
    );
    let caster_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_parity_caster.zmaterial",
        "ShadowParityCaster",
        [0.44, 0.44, 0.42, 1.0],
        true,
        false,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let (forward_shadowed, forward_shadowed_stats) =
        render_directional_shadow_capture_frame_with_pipeline(
            &server,
            viewport_size,
            RenderPipelineHandle::new(1),
            receiver_shadow_material,
            caster_material,
            "directional-shadow-forward-shadowed",
        );
    let (forward_unshadowed, forward_unshadowed_stats) =
        render_directional_shadow_capture_frame_with_pipeline(
            &server,
            viewport_size,
            RenderPipelineHandle::new(1),
            receiver_unshadowed_material,
            caster_material,
            "directional-shadow-forward-unshadowed",
        );
    let (deferred_shadowed, deferred_shadowed_stats) =
        render_directional_shadow_capture_frame_with_pipeline(
            &server,
            viewport_size,
            RenderPipelineHandle::new(2),
            receiver_shadow_material,
            caster_material,
            "directional-shadow-deferred-shadowed",
        );
    let (deferred_unshadowed, deferred_unshadowed_stats) =
        render_directional_shadow_capture_frame_with_pipeline(
            &server,
            viewport_size,
            RenderPipelineHandle::new(2),
            receiver_unshadowed_material,
            caster_material,
            "directional-shadow-deferred-unshadowed",
        );

    assert_directional_shadow_capture_stats("forward shadowed receiver", &forward_shadowed_stats);
    assert_directional_shadow_capture_stats(
        "forward unshadowed receiver",
        &forward_unshadowed_stats,
    );
    assert_directional_shadow_capture_stats("deferred shadowed receiver", &deferred_shadowed_stats);
    assert_directional_shadow_capture_stats(
        "deferred unshadowed receiver",
        &deferred_unshadowed_stats,
    );
    assert_pipeline_executor(
        "forward shadowed receiver",
        &forward_shadowed_stats,
        "mesh.opaque",
    );
    assert_pipeline_executor(
        "deferred shadowed receiver",
        &deferred_shadowed_stats,
        "lighting.deferred",
    );

    let forward_darkening =
        frame_darkened_pixel_count_and_luma_delta(&forward_shadowed, &forward_unshadowed);
    let deferred_darkening =
        frame_darkened_pixel_count_and_luma_delta(&deferred_shadowed, &deferred_unshadowed);
    assert_darkening_stats_same_product_range(
        "directional shadow forward/deferred parity",
        forward_darkening,
        deferred_darkening,
    );
}

fn render_directional_shadow_capture_frame(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    profile_name: &str,
) -> (CapturedFrame, RenderStats) {
    render_directional_shadow_capture_frame_with_camera_offset(
        server,
        viewport_size,
        receiver_material,
        caster_material,
        profile_name,
        Vec3::ZERO,
    )
}

fn render_directional_shadow_capture_frame_with_camera_offset(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    profile_name: &str,
    camera_offset: Vec3,
) -> (CapturedFrame, RenderStats) {
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, directional_shadow_capture_profile(profile_name))
        .unwrap();
    server
        .submit_frame_extract(
            viewport,
            directional_shadow_capture_extract(
                viewport_size,
                receiver_material,
                caster_material,
                camera_offset,
            ),
        )
        .unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("directional shadow product frame should be capturable");
    let stats = server.query_stats().unwrap();
    server.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn render_directional_shadow_capture_frame_with_pipeline(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    pipeline: RenderPipelineHandle,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    profile_name: &str,
) -> (CapturedFrame, RenderStats) {
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            directional_shadow_capture_profile(profile_name).with_pipeline_asset(pipeline),
        )
        .unwrap();
    server
        .submit_frame_extract(
            viewport,
            directional_shadow_capture_extract(
                viewport_size,
                receiver_material,
                caster_material,
                Vec3::ZERO,
            ),
        )
        .unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("directional shadow product frame should be capturable");
    let stats = server.query_stats().unwrap();
    server.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn directional_shadow_capture_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    camera_offset: Vec3,
) -> RenderFrameExtract {
    directional_shadow_capture_extract_with_shadow_settings(
        viewport_size,
        receiver_material,
        caster_material,
        camera_offset,
        directional_shadow_capture_settings(),
    )
}

fn directional_shadow_capture_extract_with_shadow_settings(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    camera_offset: Vec3,
    shadow_settings: LightShadowSettings,
) -> RenderFrameExtract {
    let camera_eye = Vec3::new(0.0, -3.2, 2.2) + camera_offset;
    let camera_target = Vec3::new(0.0, 0.0, 0.15) + camera_offset;
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(50_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(camera_eye, camera_target, Vec3::Y),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![
                    shadow_capture_mesh(
                        50_100,
                        Transform {
                            scale: Vec3::new(3.2, 2.2, 0.04),
                            ..Transform::default()
                        },
                        receiver_material,
                    ),
                    shadow_capture_mesh(
                        50_101,
                        Transform {
                            translation: Vec3::new(0.0, 0.0, 0.58),
                            scale: Vec3::new(0.38, 0.38, 0.72),
                            ..Transform::default()
                        },
                        caster_material,
                    ),
                ],
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 50_200,
                    light_id: 50_200,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        DEFAULT_RENDER_LAYER_MASK,
                    ),
                    direction: Vec3::new(0.45, 0.25, -1.0).normalize(),
                    color: Vec3::ONE,
                    intensity: 0.8,
                    shadow: Some(shadow_settings),
                }],
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: true,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(viewport_size)
}

fn directional_shadow_capture_settings() -> LightShadowSettings {
    shadow_capture_settings_with_quality(ShadowPcfQuality::Low, ShadowResolutionTier::T1024)
}

fn assert_pipeline_executor(label: &str, stats: &RenderStats, executor_id: &str) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&executor_id.to_string()),
        "{label}: expected executor `{executor_id}`; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
}

fn assert_darkening_stats_close(label: &str, baseline: (usize, f32), shifted: (usize, f32)) {
    let (baseline_pixels, baseline_sum) = baseline;
    let (shifted_pixels, shifted_sum) = shifted;
    assert!(
        baseline_pixels > 80 && baseline_sum > 600.0,
        "{label}: baseline frame should contain a measurable receiver shadow; pixels={baseline_pixels} sum={baseline_sum:.2}"
    );
    assert!(
        shifted_pixels > 80 && shifted_sum > 600.0,
        "{label}: shifted frame should retain a measurable receiver shadow; pixels={shifted_pixels} sum={shifted_sum:.2}"
    );

    let pixel_delta = baseline_pixels.abs_diff(shifted_pixels);
    let sum_delta = (baseline_sum - shifted_sum).abs();
    let pixel_budget = (baseline_pixels.max(shifted_pixels) / 4).max(45);
    let sum_budget = baseline_sum.max(shifted_sum) * 0.35;
    assert!(
        pixel_delta <= pixel_budget && sum_delta <= sum_budget,
        "{label}: subtexel camera shift should not cause shadow swimming; baseline=({baseline_pixels},{baseline_sum:.2}) shifted=({shifted_pixels},{shifted_sum:.2}) budgets=({pixel_budget},{sum_budget:.2})"
    );
}

fn assert_darkening_stats_same_product_range(
    label: &str,
    forward: (usize, f32),
    deferred: (usize, f32),
) {
    let (forward_pixels, forward_sum) = forward;
    let (deferred_pixels, deferred_sum) = deferred;
    assert!(
        forward_pixels > 80 && forward_sum > 600.0,
        "{label}: forward frame should contain a measurable receiver shadow; pixels={forward_pixels} sum={forward_sum:.2}"
    );
    assert!(
        deferred_pixels > 80 && deferred_sum > 600.0,
        "{label}: deferred frame should contain a measurable receiver shadow; pixels={deferred_pixels} sum={deferred_sum:.2}"
    );

    let stronger_pixels = forward_pixels.max(deferred_pixels);
    let weaker_pixels = forward_pixels.min(deferred_pixels);
    let stronger_sum = forward_sum.max(deferred_sum);
    let weaker_sum = forward_sum.min(deferred_sum);
    assert!(
        weaker_pixels * 2 >= stronger_pixels && weaker_sum >= stronger_sum * 0.40,
        "{label}: forward/deferred shadow darkening should stay in the same product range; forward=({forward_pixels},{forward_sum:.2}) deferred=({deferred_pixels},{deferred_sum:.2})"
    );
}
