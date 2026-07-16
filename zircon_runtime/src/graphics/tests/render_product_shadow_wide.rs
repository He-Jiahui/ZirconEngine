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

const SHADOW_ATLAS_EXECUTOR_ID: &str = "shadow.atlas";
const LIGHT_GRID_EXECUTOR_ID: &str = "lighting.light-grid";

#[test]
fn render_product_mixed_shadow_atlas_lights_darkens_multiple_receivers_capture() {
    let viewport_size = UVec2::new(192, 128);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let receiver_shadow_material = register_material(
        asset_manager.as_ref(),
        "res://materials/mixed_shadow_wide_receiver_shadowed.zmaterial",
        "MixedShadowWideReceiverShadowed",
        [0.34, 0.34, 0.30, 1.0],
        false,
        true,
    );
    let receiver_unshadowed_material = register_material(
        asset_manager.as_ref(),
        "res://materials/mixed_shadow_wide_receiver_unshadowed.zmaterial",
        "MixedShadowWideReceiverUnshadowed",
        [0.34, 0.34, 0.30, 1.0],
        false,
        false,
    );
    let caster_material = register_material(
        asset_manager.as_ref(),
        "res://materials/mixed_shadow_wide_caster.zmaterial",
        "MixedShadowWideCaster",
        [0.58, 0.53, 0.46, 1.0],
        true,
        false,
    );
    let server = WgpuRenderFramework::new_for_test(asset_manager).unwrap();

    let (shadowed_frame, shadowed_stats) = render_mixed_shadow_frame(
        &server,
        viewport_size,
        receiver_shadow_material,
        caster_material,
        "mixed-shadow-wide-shadowed",
    );
    let (unshadowed_frame, unshadowed_stats) = render_mixed_shadow_frame(
        &server,
        viewport_size,
        receiver_unshadowed_material,
        caster_material,
        "mixed-shadow-wide-unshadowed",
    );

    assert_mixed_shadow_stats(&shadowed_stats);
    assert_mixed_shadow_stats(&unshadowed_stats);

    let whole_frame = darkening_profile(&shadowed_frame, &unshadowed_frame);
    assert!(
        whole_frame.darkened_pixels > 520,
        "mixed shadow scene should darken a broad set of receiver pixels, got {}",
        whole_frame.darkened_pixels
    );
    assert!(
        whole_frame.luma_delta_sum > 5_000.0,
        "mixed shadow scene should accumulate visible luma darkening, got {:.2}",
        whole_frame.luma_delta_sum
    );
    assert!(
        whole_frame.rgb_delta_sum > 28_000,
        "mixed shadow scene should visibly change the final color capture, got {}",
        whole_frame.rgb_delta_sum
    );

    assert_region_darkened(
        "left receiver shadow",
        &shadowed_frame,
        &unshadowed_frame,
        UVec2::new(20, 48),
        UVec2::new(48, 46),
        55,
        420.0,
    );
    assert_region_darkened(
        "center receiver shadow",
        &shadowed_frame,
        &unshadowed_frame,
        UVec2::new(72, 44),
        UVec2::new(48, 50),
        70,
        580.0,
    );
    assert_region_darkened(
        "right receiver shadow",
        &shadowed_frame,
        &unshadowed_frame,
        UVec2::new(124, 48),
        UVec2::new(48, 46),
        55,
        420.0,
    );
}

fn register_material(
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

    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            MaterialAsset {
                name: Some(name.to_string()),
                shader: AssetReference::from_locator(
                    AssetUri::parse("builtin://shader/pbr.wgsl").unwrap(),
                ),
                parent: None,
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
                options: Default::default(),
                queue: None,
                validation_diagnostics: Vec::new(),
            },
        )
        .expect("mixed shadow material should insert");
    material_id
}

fn render_mixed_shadow_frame(
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
        .set_quality_profile(viewport, mixed_shadow_profile(profile_name))
        .unwrap();
    server
        .submit_frame_extract(
            viewport,
            mixed_shadow_extract(viewport_size, receiver_material, caster_material),
        )
        .unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("mixed shadow product frame should be capturable");
    let stats = server.query_stats().unwrap();
    server.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn mixed_shadow_profile(name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_pipeline_asset(RenderPipelineHandle::new(1))
        .with_clustered_lighting(true)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn mixed_shadow_extract(
    viewport_size: UVec2,
    receiver_material: ResourceId,
    caster_material: ResourceId,
) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(55_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, -3.75, 2.55),
                        Vec3::new(0.0, 0.02, 0.15),
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: mixed_shadow_meshes(receiver_material, caster_material),
                directional_lights: vec![RenderDirectionalLightSnapshot {
                    node_id: 55_200,
                    light_id: 55_200,
                    layer_mask: RenderLayerSet::from_scene_schema_v1_mask(
                        DEFAULT_RENDER_LAYER_MASK,
                    ),
                    direction: Vec3::new(0.38, 0.20, -1.0).normalize(),
                    color: Vec3::ONE,
                    intensity: 0.78,
                    mobility: crate::core::framework::scene::Mobility::Dynamic,
                    shadow: Some(mixed_shadow_settings(
                        ShadowResolutionTier::T1024,
                        ShadowPcfQuality::Medium,
                    )),
                }],
                point_lights: Vec::new(),
                spot_lights: vec![
                    mixed_shadow_spot_light(0, -1.35),
                    mixed_shadow_spot_light(1, 0.0),
                    mixed_shadow_spot_light(2, 1.35),
                ],
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

fn mixed_shadow_meshes(
    receiver_material: ResourceId,
    caster_material: ResourceId,
) -> Vec<RenderMeshSnapshot> {
    let mut meshes = vec![
        shadow_mesh(
            55_100,
            Transform {
                translation: Vec3::new(0.0, 0.1, 0.0),
                scale: Vec3::new(4.5, 2.65, 0.04),
                ..Transform::default()
            },
            receiver_material,
        ),
        shadow_mesh(
            55_101,
            Transform {
                translation: Vec3::new(0.0, -0.18, 0.62),
                scale: Vec3::new(0.30, 0.36, 0.92),
                ..Transform::default()
            },
            caster_material,
        ),
    ];

    for (index, x) in [-1.38_f32, 0.0, 1.38].into_iter().enumerate() {
        meshes.push(shadow_mesh(
            55_110 + index as u64,
            Transform {
                translation: Vec3::new(x, 0.05, 0.45),
                scale: Vec3::new(0.28, 0.28, 0.72),
                ..Transform::default()
            },
            caster_material,
        ));
    }

    meshes
}

fn mixed_shadow_spot_light(index: usize, x: f32) -> RenderSpotLightSnapshot {
    let position = Vec3::new(x, -2.1, 2.35);
    let target = Vec3::new(x, 0.05, 0.02);
    RenderSpotLightSnapshot {
        node_id: 55_300 + index as u64,
        light_id: 55_400 + index as u64,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position,
        direction: (target - position).normalize(),
        color: Vec3::ONE,
        intensity: 3.25,
        range: 5.4,
        inner_angle_radians: 0.48,
        outer_angle_radians: 0.78,
        mobility: crate::core::framework::scene::Mobility::Dynamic,
        shadow: Some(mixed_shadow_settings(
            ShadowResolutionTier::T512,
            ShadowPcfQuality::Medium,
        )),
    }
}

fn mixed_shadow_settings(
    resolution_preference: ShadowResolutionTier,
    pcf_quality: ShadowPcfQuality,
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

fn shadow_mesh(node_id: u64, transform: Transform, material: ResourceId) -> RenderMeshSnapshot {
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
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
    }
}

fn assert_mixed_shadow_stats(stats: &RenderStats) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&SHADOW_ATLAS_EXECUTOR_ID.to_string()),
        "mixed shadow scene should execute the shadow atlas executor, got {:?}",
        stats.last_graph_executed_executor_ids
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&LIGHT_GRID_EXECUTOR_ID.to_string()),
        "mixed shadow scene should execute clustered light grid, got {:?}",
        stats.last_graph_executed_executor_ids
    );
    let report = &stats.last_shadow_execution_report;
    assert!(
        report.shadow_pass_executed,
        "shadow atlas pass should execute for mixed light scene"
    );
    assert!(
        report.shadow_atlas_write_count > 0,
        "mixed light scene should write the shadow atlas, got {}",
        report.shadow_atlas_write_count
    );
    assert!(
        report.caster_draw_count >= 4,
        "mixed light scene should draw all caster groups into the atlas, got {}",
        report.caster_draw_count
    );
    assert!(
        report.receiver_available,
        "mixed light scene should keep receiver path available"
    );
    assert_eq!(
        stats.last_directional_light_ready_count, 1,
        "mixed shadow scene should prepare one directional light"
    );
    assert_eq!(
        stats.last_spot_light_ready_count, 3,
        "mixed shadow scene should prepare three spot lights"
    );
    assert_eq!(
        report.shadowed_light_count, 4,
        "mixed shadow scene should count one directional plus three spot shadowed lights"
    );
}

#[derive(Default)]
struct DarkeningProfile {
    darkened_pixels: usize,
    luma_delta_sum: f32,
    rgb_delta_sum: u64,
}

fn assert_region_darkened(
    label: &str,
    shadowed: &CapturedFrame,
    unshadowed: &CapturedFrame,
    origin: UVec2,
    size: UVec2,
    min_darkened_pixels: usize,
    min_luma_delta_sum: f32,
) {
    let profile = darkening_profile_in_region(shadowed, unshadowed, origin, size);
    assert!(
        profile.darkened_pixels >= min_darkened_pixels,
        "{label} should darken at least {min_darkened_pixels} pixels, got {}",
        profile.darkened_pixels
    );
    assert!(
        profile.luma_delta_sum >= min_luma_delta_sum,
        "{label} should accumulate luma darkening >= {min_luma_delta_sum:.2}, got {:.2}",
        profile.luma_delta_sum
    );
}

fn darkening_profile(shadowed: &CapturedFrame, unshadowed: &CapturedFrame) -> DarkeningProfile {
    darkening_profile_in_region(
        shadowed,
        unshadowed,
        UVec2::ZERO,
        UVec2::new(shadowed.width, shadowed.height),
    )
}

fn darkening_profile_in_region(
    shadowed: &CapturedFrame,
    unshadowed: &CapturedFrame,
    origin: UVec2,
    size: UVec2,
) -> DarkeningProfile {
    assert_eq!(shadowed.width, unshadowed.width);
    assert_eq!(shadowed.height, unshadowed.height);
    let max_x = (origin.x + size.x).min(shadowed.width);
    let max_y = (origin.y + size.y).min(shadowed.height);
    let width = shadowed.width as usize;
    let mut profile = DarkeningProfile::default();
    for y in origin.y as usize..max_y as usize {
        for x in origin.x as usize..max_x as usize {
            let pixel_index = (y * width + x) * 4;
            let shadowed_pixel = &shadowed.rgba[pixel_index..pixel_index + 4];
            let unshadowed_pixel = &unshadowed.rgba[pixel_index..pixel_index + 4];
            let shadowed_luma = rgb_luma(shadowed_pixel);
            let unshadowed_luma = rgb_luma(unshadowed_pixel);
            let luma_delta = unshadowed_luma - shadowed_luma;
            if luma_delta > 2.0 {
                profile.darkened_pixels += 1;
                profile.luma_delta_sum += luma_delta;
            }
            profile.rgb_delta_sum += shadowed_pixel[0].abs_diff(unshadowed_pixel[0]) as u64;
            profile.rgb_delta_sum += shadowed_pixel[1].abs_diff(unshadowed_pixel[1]) as u64;
            profile.rgb_delta_sum += shadowed_pixel[2].abs_diff(unshadowed_pixel[2]) as u64;
        }
    }
    profile
}

fn rgb_luma(pixel: &[u8]) -> f32 {
    pixel[0] as f32 * 0.2126 + pixel[1] as f32 * 0.7152 + pixel[2] as f32 * 0.0722
}
