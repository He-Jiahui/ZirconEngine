use std::collections::BTreeMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AlphaMode, AssetReference, AssetUri, MaterialAsset};
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, LightShadowSettings, PreviewEnvironmentExtract,
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderPipelineHandle, RenderQualityProfile, RenderSceneGeometryExtract,
    RenderSceneSnapshot, RenderSpotLightSnapshot, RenderStats, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ShadowPcfQuality, ShadowResolutionTier, ViewportCameraSnapshot,
    DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::WgpuRenderFramework;

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

#[test]
fn render_product_spot_shadow_pcf_quality_changes_receiver_edge_capture() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_shadow_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_pcf_receiver_on.zmaterial",
        "ShadowPcfReceiverOn",
        [0.28, 0.30, 0.32, 1.0],
        false,
        true,
    );
    let receiver_unshadowed_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_pcf_receiver_off.zmaterial",
        "ShadowPcfReceiverOff",
        [0.28, 0.30, 0.32, 1.0],
        false,
        false,
    );
    let caster_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/shadow_pcf_caster.zmaterial",
        "ShadowPcfCaster",
        [0.44, 0.44, 0.42, 1.0],
        true,
        false,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let low_settings =
        shadow_capture_settings_with_quality(ShadowPcfQuality::Low, ShadowResolutionTier::T128);
    let high_settings =
        shadow_capture_settings_with_quality(ShadowPcfQuality::High, ShadowResolutionTier::T128);
    let (low_shadowed, low_stats) = render_spot_shadow_pcf_capture_frame(
        &server,
        viewport_size,
        receiver_shadow_material,
        caster_material,
        "spot-shadow-pcf-low",
        low_settings,
    );
    let (high_shadowed, high_stats) = render_spot_shadow_pcf_capture_frame(
        &server,
        viewport_size,
        receiver_shadow_material,
        caster_material,
        "spot-shadow-pcf-high",
        high_settings,
    );
    let (unshadowed, unshadowed_stats) = render_spot_shadow_pcf_capture_frame(
        &server,
        viewport_size,
        receiver_unshadowed_material,
        caster_material,
        "spot-shadow-pcf-unshadowed",
        high_settings,
    );

    assert_directional_shadow_capture_stats("low PCF receiver", &low_stats);
    assert_directional_shadow_capture_stats("high PCF receiver", &high_stats);
    assert_directional_shadow_capture_stats("unshadowed PCF receiver", &unshadowed_stats);

    let low_profile = frame_shadow_darkening_profile(&low_shadowed, &unshadowed);
    let high_profile = frame_shadow_darkening_profile(&high_shadowed, &unshadowed);
    let quality_delta = frame_rgb_abs_delta(&low_shadowed, &high_shadowed);
    assert!(
        low_profile.darkened_pixels > 80 && low_profile.luma_delta > 600.0,
        "low PCF frame should contain a measurable receiver shadow; low_profile={low_profile:?}"
    );
    assert!(
        high_profile.darkened_pixels > 80 && high_profile.luma_delta > 600.0,
        "high PCF frame should contain a measurable receiver shadow; high_profile={high_profile:?}"
    );
    assert!(
        quality_delta > 250,
        "PCF quality should change the captured receiver edge; delta={quality_delta} low={low_profile:?} high={high_profile:?}"
    );
}

#[test]
fn render_product_multi_spot_shadow_atlas_darkens_receivers_capture() {
    let viewport_size = UVec2::new(180, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_shadow_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/multi_spot_shadow_receiver_on.zmaterial",
        "MultiSpotShadowReceiverOn",
        [0.30, 0.31, 0.28, 1.0],
        false,
        true,
    );
    let receiver_unshadowed_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/multi_spot_shadow_receiver_off.zmaterial",
        "MultiSpotShadowReceiverOff",
        [0.30, 0.31, 0.28, 1.0],
        false,
        false,
    );
    let caster_material = register_shadow_capture_material(
        asset_manager.as_ref(),
        "res://materials/multi_spot_shadow_caster.zmaterial",
        "MultiSpotShadowCaster",
        [0.50, 0.50, 0.45, 1.0],
        true,
        false,
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let (shadowed_frame, shadowed_stats) = render_multi_spot_shadow_capture_frame(
        &server,
        viewport_size,
        receiver_shadow_material,
        caster_material,
        "multi-spot-shadow-receiver-on",
    );
    let (unshadowed_frame, unshadowed_stats) = render_multi_spot_shadow_capture_frame(
        &server,
        viewport_size,
        receiver_unshadowed_material,
        caster_material,
        "multi-spot-shadow-receiver-off",
    );

    assert_multi_spot_shadow_capture_stats("shadowed receiver", &shadowed_stats);
    assert_multi_spot_shadow_capture_stats("unshadowed receiver", &unshadowed_stats);

    let (darkened_pixels, darkening_sum) =
        frame_darkened_pixel_count_and_luma_delta(&shadowed_frame, &unshadowed_frame);
    let frame_delta = frame_rgb_abs_delta(&shadowed_frame, &unshadowed_frame);
    assert!(
        darkened_pixels > 220 && darkening_sum > 2_000.0 && frame_delta > 18_000,
        "multi-spot shadow receiver should visibly darken atlas-sampled receiver pixels; darkened_pixels={darkened_pixels} darkening_sum={darkening_sum:.2} frame_delta={frame_delta}"
    );
}

fn register_shadow_capture_material(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    name: &str,
    base_color: [f32; 4],
    cast_shadows: bool,
    receive_shadows: bool,
) -> ResourceId {
    let mut property_values = BTreeMap::new();
    property_values.insert(
        "cast_shadows".to_string(),
        toml::Value::Boolean(cast_shadows),
    );
    property_values.insert(
        "receive_shadows".to_string(),
        toml::Value::Boolean(receive_shadows),
    );

    let material = MaterialAsset {
        name: Some(name.to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        base_color,
        base_color_texture: None,
        normal_texture: None,
        metallic: 0.0,
        roughness: 1.0,
        metallic_roughness_texture: None,
        occlusion_texture: None,
        emissive: [0.0, 0.0, 0.0],
        emissive_texture: None,
        alpha_mode: AlphaMode::Opaque,
        double_sided: false,
        property_values,
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    };
    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("shadow capture material insert");
    material_id
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

fn render_spot_shadow_pcf_capture_frame(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    profile_name: &str,
    shadow_settings: LightShadowSettings,
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
            spot_shadow_pcf_capture_extract(
                viewport_size,
                receiver_material,
                caster_material,
                shadow_settings,
            ),
        )
        .unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("spot PCF shadow product frame should be capturable");
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

fn render_multi_spot_shadow_capture_frame(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    profile_name: &str,
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
            multi_spot_shadow_capture_extract(viewport_size, receiver_material, caster_material),
        )
        .unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("multi-spot shadow product frame should be capturable");
    let stats = server.query_stats().unwrap();
    server.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn directional_shadow_capture_profile(name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(true)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn multi_spot_shadow_capture_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
) -> RenderFrameExtract {
    let caster_x_positions = [-1.05_f32, 0.0, 1.05];
    let mut meshes = vec![shadow_capture_mesh(
        51_100,
        Transform {
            scale: Vec3::new(3.8, 2.3, 0.04),
            ..Transform::default()
        },
        receiver_material,
    )];
    meshes.extend(caster_x_positions.iter().enumerate().map(|(index, x)| {
        shadow_capture_mesh(
            51_200 + index as u64,
            Transform {
                translation: Vec3::new(*x, -0.10, 0.54),
                scale: Vec3::new(0.26, 0.30, 0.86),
                ..Transform::default()
            },
            caster_material,
        )
    }));

    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(51_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -3.25, 2.35),
                        Vec3::new(0.0, 0.0, 0.16),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes,
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: caster_x_positions
                    .iter()
                    .enumerate()
                    .map(|(index, x)| multi_spot_shadow_capture_light(index, *x))
                    .collect(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
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
                    layer_mask: RenderLayerSet::from_legacy_mask(DEFAULT_RENDER_LAYER_MASK),
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

fn spot_shadow_pcf_capture_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
    shadow_settings: LightShadowSettings,
) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(52_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -3.2, 2.2),
                        Vec3::new(0.0, 0.0, 0.15),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![
                    shadow_capture_mesh(
                        52_100,
                        Transform {
                            scale: Vec3::new(3.2, 2.2, 0.04),
                            ..Transform::default()
                        },
                        receiver_material,
                    ),
                    shadow_capture_mesh(
                        52_101,
                        Transform {
                            translation: Vec3::new(-1.05, 0.0, 0.42),
                            scale: Vec3::new(0.10, 0.14, 0.42),
                            ..Transform::default()
                        },
                        caster_material,
                    ),
                ],
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: vec![RenderSpotLightSnapshot {
                    node_id: 52_200,
                    light_id: 52_200,
                    layer_mask: RenderLayerSet::from_legacy_mask(DEFAULT_RENDER_LAYER_MASK),
                    position: Vec3::new(-0.30, -1.85, 2.05),
                    direction: (Vec3::new(-0.65, 0.02, 0.04) - Vec3::new(-0.30, -1.85, 2.05))
                        .normalize(),
                    color: Vec3::ONE,
                    intensity: 3.6,
                    range: 4.6,
                    inner_angle_radians: 0.42,
                    outer_angle_radians: 0.72,
                    shadow: Some(shadow_settings),
                }],
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
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

fn multi_spot_shadow_capture_light(index: usize, x: f32) -> RenderSpotLightSnapshot {
    let position = Vec3::new(x, -1.95, 2.25);
    let target = Vec3::new(x, 0.05, 0.02);
    RenderSpotLightSnapshot {
        node_id: 51_300 + index as u64,
        light_id: 51_400 + index as u64,
        layer_mask: RenderLayerSet::from_legacy_mask(DEFAULT_RENDER_LAYER_MASK),
        position,
        direction: (target - position).normalize(),
        color: Vec3::ONE,
        intensity: 3.4,
        range: 5.0,
        inner_angle_radians: 0.48,
        outer_angle_radians: 0.78,
        shadow: Some(multi_spot_shadow_capture_settings()),
    }
}

fn directional_shadow_capture_settings() -> LightShadowSettings {
    shadow_capture_settings_with_quality(ShadowPcfQuality::Low, ShadowResolutionTier::T1024)
}

fn shadow_capture_settings_with_quality(
    pcf_quality: ShadowPcfQuality,
    resolution_preference: ShadowResolutionTier,
) -> LightShadowSettings {
    LightShadowSettings {
        casts_shadow: true,
        depth_bias: 0.0,
        normal_bias: 0.0,
        strength: 1.0,
        resolution_preference,
        pcf_quality,
    }
}

fn multi_spot_shadow_capture_settings() -> LightShadowSettings {
    LightShadowSettings {
        casts_shadow: true,
        depth_bias: 0.0,
        normal_bias: 0.0,
        strength: 1.0,
        resolution_preference: ShadowResolutionTier::T512,
        pcf_quality: ShadowPcfQuality::Medium,
    }
}

fn shadow_capture_mesh(
    node_id: u64,
    transform: Transform,
    material: ResourceId,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform,
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_legacy_mask(DEFAULT_RENDER_LAYER_MASK),
    }
}

fn assert_directional_shadow_capture_stats(label: &str, stats: &RenderStats) {
    let report = stats.last_shadow_execution_report;
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"shadow.atlas".to_string()),
        "{label}: shadow atlas executor should run; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert!(report.shadow_pass_executed, "{label}: shadow pass missing");
    assert!(
        report.shadow_atlas_write_count > 0,
        "{label}: shadow atlas should receive a graph write; report={report:?}"
    );
    assert!(
        report.receiver_available,
        "{label}: receiver should read the written shadow atlas; report={report:?}"
    );
    assert!(
        report.caster_draw_count > 0,
        "{label}: caster draw count should be non-zero; report={report:?}"
    );
}

fn assert_multi_spot_shadow_capture_stats(label: &str, stats: &RenderStats) {
    let report = stats.last_shadow_execution_report;
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"shadow.atlas".to_string()),
        "{label}: shadow atlas executor should run; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert_eq!(stats.last_spot_light_ready_count, 3, "{label}: spot lights");
    assert_eq!(
        report.shadowed_light_count, 3,
        "{label}: shadow report should count all three spot shadow lights; report={report:?}"
    );
    assert!(report.shadow_pass_executed, "{label}: shadow pass missing");
    assert!(
        report.shadow_atlas_write_count > 0,
        "{label}: shadow atlas should receive a graph write; report={report:?}"
    );
    assert!(
        report.receiver_available,
        "{label}: receiver should read the written shadow atlas; report={report:?}"
    );
    assert!(
        report.caster_draw_count >= 3,
        "{label}: each spot caster should contribute a shadow command; report={report:?}"
    );
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

fn average_luma_in_region(frame: &CapturedFrame, origin: UVec2, size: UVec2) -> f32 {
    let x_end = origin.x.saturating_add(size.x).min(frame.width) as usize;
    let y_end = origin.y.saturating_add(size.y).min(frame.height) as usize;
    let width = frame.width as usize;
    let mut total = 0.0;
    let mut count = 0.0;
    for y in origin.y as usize..y_end {
        for x in origin.x as usize..x_end {
            let index = (y * width + x) * 4;
            let pixel = &frame.rgba[index..index + 4];
            total += 0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32;
            count += 1.0;
        }
    }
    if count <= 0.0 {
        0.0
    } else {
        total / count
    }
}

fn frame_darkened_pixel_count_and_luma_delta(
    shadowed: &CapturedFrame,
    unshadowed: &CapturedFrame,
) -> (usize, f32) {
    assert_eq!(shadowed.width, unshadowed.width);
    assert_eq!(shadowed.height, unshadowed.height);
    let mut darkened_pixels = 0;
    let mut luma_delta = 0.0;
    for (shadowed_pixel, unshadowed_pixel) in shadowed
        .rgba
        .chunks_exact(4)
        .zip(unshadowed.rgba.chunks_exact(4))
    {
        let shadowed_luma = rgb_luma(shadowed_pixel);
        let unshadowed_luma = rgb_luma(unshadowed_pixel);
        let delta = unshadowed_luma - shadowed_luma;
        if delta > 2.0 {
            darkened_pixels += 1;
            luma_delta += delta;
        }
    }
    (darkened_pixels, luma_delta)
}

fn frame_rgb_abs_delta(left: &CapturedFrame, right: &CapturedFrame) -> u64 {
    assert_eq!(left.width, right.width);
    assert_eq!(left.height, right.height);
    left.rgba
        .chunks_exact(4)
        .zip(right.rgba.chunks_exact(4))
        .map(|(left, right)| {
            (left[0].abs_diff(right[0]) as u64)
                + (left[1].abs_diff(right[1]) as u64)
                + (left[2].abs_diff(right[2]) as u64)
        })
        .sum()
}

#[derive(Clone, Copy, Debug)]
struct ShadowDarkeningProfile {
    darkened_pixels: usize,
    luma_delta: f32,
}

fn frame_shadow_darkening_profile(
    shadowed: &CapturedFrame,
    unshadowed: &CapturedFrame,
) -> ShadowDarkeningProfile {
    let (darkened_pixels, luma_delta) =
        frame_darkened_pixel_count_and_luma_delta(shadowed, unshadowed);
    ShadowDarkeningProfile {
        darkened_pixels,
        luma_delta,
    }
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

fn rgb_luma(pixel: &[u8]) -> f32 {
    0.2126 * pixel[0] as f32 + 0.7152 * pixel[1] as f32 + 0.0722 * pixel[2] as f32
}
