use std::fs;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe,
    RenderHybridGiTraceRegion, RenderMeshSnapshot, RenderPointLightSnapshot, RenderSceneSnapshot,
    RenderSpotLightSnapshot, RenderWorldSnapshotHandle,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::UVec2;
use crate::core::math::{Quat, Transform, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::tests::plugin_render_feature_fixtures::hybrid_gi_render_feature_descriptor;
use crate::scene::world::World;

use super::hybrid_gi_scene_prepare_material_fixtures::{
    material_capture_test_assets, material_surface_response_test_assets,
    material_texture_capture_test_assets, material_visibility_capture_test_assets, model_handle,
    MaterialTextureCaptureTestAssets, MaterialVisibilityCaptureTestAssets,
};

use crate::{
    types::{
        hybrid_gi_voxel_clipmap_bounds_cell_ranges, hybrid_gi_voxel_clipmap_cell_bit_index,
        HybridGiPrepareCardCaptureRequest, HybridGiPrepareFrame, HybridGiPrepareProbe,
        HybridGiPrepareSurfaceCachePageContent, HybridGiPrepareVoxelCell,
        HybridGiPrepareVoxelClipmap, HybridGiScenePrepareFrame, ViewportRenderFrame,
    },
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

const CARD_CAPTURE_TILE_EXTENT: u32 = 64;
const CARD_CAPTURE_ATLAS_COLUMNS: u32 = 8;
const TEST_SCENE_PREPARE_VOXEL_MIN_MESH_BOUNDS_RADIUS: f32 = 0.5;

fn compile_hybrid_gi_pipeline(extract: &RenderFrameExtract) -> CompiledRenderPipeline {
    RenderPipelineAsset::default_forward_plus()
        .with_plugin_render_features([hybrid_gi_render_feature_descriptor()])
        .compile_with_options(
            extract,
            &RenderPipelineCompileOptions::default()
                .with_capability_enabled(
                    RenderFeatureCapabilityRequirement::HybridGlobalIllumination,
                )
                .with_feature_disabled(BuiltinRenderFeature::ClusteredLighting)
                .with_feature_disabled(BuiltinRenderFeature::ScreenSpaceAmbientOcclusion)
                .with_feature_disabled(BuiltinRenderFeature::HistoryResolve)
                .with_feature_disabled(BuiltinRenderFeature::Bloom)
                .with_feature_disabled(BuiltinRenderFeature::ColorGrading)
                .with_feature_disabled(BuiltinRenderFeature::ReflectionProbes)
                .with_feature_disabled(BuiltinRenderFeature::BakedLighting)
                .with_feature_disabled(BuiltinRenderFeature::Particle)
                .with_feature_disabled(BuiltinRenderFeature::VirtualGeometry)
                .with_async_compute(false),
        )
        .unwrap()
}

fn hybrid_gi_scene_renderer(asset_manager: Arc<ProjectAssetManager>) -> SceneRenderer {
    SceneRenderer::new_with_plugin_render_features(
        asset_manager,
        [hybrid_gi_render_feature_descriptor()],
    )
    .unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ScenePrepareResourceSnapshotForTest {
    occupied_atlas_slots: Vec<u32>,
    occupied_capture_slots: Vec<u32>,
    atlas_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    capture_slot_rgba_samples: Vec<(u32, [u8; 4])>,
    atlas_slot_count: u32,
    capture_slot_count: u32,
    voxel_clipmap_rgba_samples: Vec<(u32, [u8; 4])>,
    voxel_clipmap_occupancy_masks: Vec<(u32, u64)>,
    voxel_clipmap_cell_rgba_samples: Vec<(u32, u32, [u8; 4])>,
    voxel_clipmap_cell_occupancy_counts: Vec<(u32, u32, u32)>,
    voxel_clipmap_cell_dominant_node_ids: Vec<(u32, u32, u64)>,
    voxel_clipmap_cell_dominant_rgba_samples: Vec<(u32, u32, [u8; 4])>,
}

#[test]
fn hybrid_gi_gpu_readback_reports_scene_prepare_card_capture_resource_snapshot() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract(
        viewport_size,
        vec![probe(200, 96, Vec3::ZERO, 0.85)],
        vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(HybridGiPrepareFrame {
                    resident_probes: vec![HybridGiPrepareProbe {
                        probe_id: 200,
                        slot: 0,
                        ray_budget: 96,
                        irradiance_rgb: [96, 96, 96],
                    }],
                    pending_updates: Vec::new(),
                    scheduled_trace_region_ids: vec![40],
                    evictable_probe_ids: Vec::new(),
                }))
                .with_hybrid_gi_scene_prepare(Some(HybridGiScenePrepareFrame {
                    card_capture_requests: vec![
                        HybridGiPrepareCardCaptureRequest {
                            card_id: 11,
                            page_id: 21,
                            atlas_slot_id: 0,
                            capture_slot_id: 1,
                            bounds_center: Vec3::new(0.0, 0.0, 0.0),
                            bounds_radius: 0.75,
                        },
                        HybridGiPrepareCardCaptureRequest {
                            card_id: 12,
                            page_id: 22,
                            atlas_slot_id: 9,
                            capture_slot_id: 4,
                            bounds_center: Vec3::new(1.0, 0.5, -0.25),
                            bounds_radius: 1.25,
                        },
                    ],
                    surface_cache_page_contents: Vec::new(),
                    voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                        clipmap_id: 7,
                        center: Vec3::new(0.0, 0.0, 0.0),
                        half_extent: 8.0,
                    }],
                    voxel_cells: Vec::new(),
                })),
            &compiled,
            None,
        )
        .unwrap();

    let readback = renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback");
    let snapshot = readback
        .scene_prepare_resources()
        .expect("expected Hybrid GI scene-prepare resource snapshot");

    assert_eq!(snapshot.card_capture_request_count(), 2);
    assert_eq!(snapshot.voxel_clipmap_ids().to_vec(), vec![7]);
    assert_eq!(snapshot.occupied_atlas_slots().to_vec(), vec![0, 9]);
    assert_eq!(snapshot.occupied_capture_slots().to_vec(), vec![1, 4]);
    assert_eq!(
        snapshot.atlas_slot_rgba_samples(),
        vec![
            (0, expected_card_capture_debug_rgba(11, 21, 0, 1)),
            (9, expected_card_capture_debug_rgba(12, 22, 9, 4)),
        ]
    );
    assert_eq!(
        snapshot.capture_slot_rgba_samples(),
        vec![
            (1, expected_card_capture_debug_rgba(11, 21, 0, 1)),
            (4, expected_card_capture_debug_rgba(12, 22, 9, 4)),
        ]
    );
    assert_eq!(snapshot.atlas_slot_count(), 10);
    assert_eq!(snapshot.capture_slot_count(), 5);
    assert_eq!(
        snapshot.atlas_texture_extent(),
        (
            CARD_CAPTURE_TILE_EXTENT * CARD_CAPTURE_ATLAS_COLUMNS,
            CARD_CAPTURE_TILE_EXTENT * 2,
        )
    );
    assert_eq!(
        snapshot.capture_texture_extent(),
        (CARD_CAPTURE_TILE_EXTENT, CARD_CAPTURE_TILE_EXTENT)
    );
    assert_eq!(snapshot.capture_layer_count(), 5);
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_mesh_tint_and_directional_light() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();

    let warm = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_tint(11, Vec4::new(1.0, 0.25, 0.2, 1.0))],
            vec![directional_light(Vec3::new(1.0, 0.2, 0.1), 2.0)],
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let cool = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_tint(11, Vec4::new(0.2, 0.3, 1.0, 1.0))],
            vec![directional_light(Vec3::new(0.1, 0.2, 1.0), 2.0)],
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let warm_atlas = slot_sample(&warm.0, 0, "atlas");
    let cool_atlas = slot_sample(&cool.0, 0, "atlas");
    let warm_capture = slot_sample(&warm.1, 1, "capture");
    let cool_capture = slot_sample(&cool.1, 1, "capture");

    assert_ne!(
        warm_atlas, cool_atlas,
        "expected card-capture atlas shading to react to scene mesh tint and directional light input instead of staying a fixed slot debug color"
    );
    assert_ne!(
        warm_capture, cool_capture,
        "expected card-capture capture-layer shading to react to scene mesh tint and directional light input instead of staying a fixed slot debug color"
    );
    assert!(
        warm_atlas[0] > cool_atlas[0] + 20,
        "expected warm directional-light seed to push the atlas sample redder than the cool one; warm_atlas={warm_atlas:?}, cool_atlas={cool_atlas:?}"
    );
    assert!(
        cool_atlas[2] > warm_atlas[2] + 20,
        "expected cool directional-light seed to push the atlas sample bluer than the warm one; warm_atlas={warm_atlas:?}, cool_atlas={cool_atlas:?}"
    );
    assert_eq!(warm_atlas, warm_capture);
    assert_eq!(cool_atlas, cool_capture);
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_point_and_spot_lights() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let meshes = vec![mesh_with_tint(11, Vec4::ONE)];

    let unlit = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            meshes.clone(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let point_lit = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            meshes.clone(),
            Vec::new(),
            vec![point_light(
                100,
                Vec3::new(1.0, 0.1, 0.1),
                3.5,
                Vec3::new(0.0, 0.0, 1.5),
                4.0,
            )],
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let spot_lit = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            meshes,
            Vec::new(),
            Vec::new(),
            vec![spot_light(
                200,
                Vec3::new(0.1, 0.1, 1.75),
                Vec3::new(0.0, 0.0, -1.0),
                Vec3::new(0.1, 0.2, 1.0),
                4.0,
                4.0,
                0.2,
                0.6,
            )],
        ),
        prepare,
        scene_prepare,
    );

    let unlit_atlas = slot_sample(&unlit.0, 0, "atlas");
    let point_atlas = slot_sample(&point_lit.0, 0, "atlas");
    let spot_atlas = slot_sample(&spot_lit.0, 0, "atlas");

    assert_ne!(
        point_atlas, unlit_atlas,
        "expected point lights to feed the card-capture sample instead of leaving the atlas on the same unlit debug color"
    );
    assert_ne!(
        spot_atlas, unlit_atlas,
        "expected spot lights to feed the card-capture sample instead of leaving the atlas on the same unlit debug color"
    );
    assert!(
        point_atlas[0] > unlit_atlas[0] + 20,
        "expected the point-light sample to raise the red channel above the unlit capture; point_atlas={point_atlas:?}, unlit_atlas={unlit_atlas:?}"
    );
    assert!(
        spot_atlas[2] > unlit_atlas[2] + 20,
        "expected the spot-light sample to raise the blue channel above the unlit capture; spot_atlas={spot_atlas:?}, unlit_atlas={unlit_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_material_base_color() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 2.0)];

    let default_material = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_material_and_tint(
                11,
                "builtin://material/default",
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let missing_material = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_material_and_tint(
                11,
                "builtin://missing-material",
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let default_atlas = slot_sample(&default_material.0, 0, "atlas");
    let missing_atlas = slot_sample(&missing_material.0, 0, "atlas");

    assert_ne!(
        default_atlas, missing_atlas,
        "expected material base_color to affect card-capture shading instead of collapsing to the same mesh-tint-only sample"
    );
    assert!(
        default_atlas[1] > missing_atlas[1] + 40,
        "expected the white builtin material to preserve substantially more green than the magenta missing-material fallback; default_atlas={default_atlas:?}, missing_atlas={missing_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_material_base_color_texture() {
    let MaterialTextureCaptureTestAssets {
        asset_manager,
        root,
        base_color_red,
        base_color_blue,
        ..
    } = material_texture_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 2.0)];

    let red = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                base_color_red,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let blue = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                base_color_blue,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let red_atlas = slot_sample(&red.0, 0, "atlas");
    let blue_atlas = slot_sample(&blue.0, 0, "atlas");
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        red_atlas, blue_atlas,
        "expected base-color texture content to affect card-capture shading even when scalar base_color stays fixed"
    );
    assert!(
        red_atlas[0] > blue_atlas[0] + 25,
        "expected the red base-color texture to bias the capture redder than the blue texture; red_atlas={red_atlas:?}, blue_atlas={blue_atlas:?}"
    );
    assert!(
        blue_atlas[2] > red_atlas[2] + 25,
        "expected the blue base-color texture to bias the capture bluer than the red texture; red_atlas={red_atlas:?}, blue_atlas={blue_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_material_emissive() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();

    let black = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                black_material,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let emissive = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                emissive_material,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let black_atlas = slot_sample(&black.0, 0, "atlas");
    let emissive_atlas = slot_sample(&emissive.0, 0, "atlas");
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        black_atlas, emissive_atlas,
        "expected material emissive to feed card-capture shading even without direct lights instead of leaving the same non-emissive sample"
    );
    assert!(
        emissive_atlas[0] > black_atlas[0] + 30,
        "expected emissive material to raise the red channel without any direct lights; black_atlas={black_atlas:?}, emissive_atlas={emissive_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_material_emissive_texture() {
    let MaterialTextureCaptureTestAssets {
        asset_manager,
        root,
        emissive_warm,
        emissive_cool,
        ..
    } = material_texture_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();

    let warm = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                emissive_warm,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let cool = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                emissive_cool,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let warm_atlas = slot_sample(&warm.0, 0, "atlas");
    let cool_atlas = slot_sample(&cool.0, 0, "atlas");
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        warm_atlas, cool_atlas,
        "expected emissive texture content to affect card-capture shading even when scalar emissive stays fixed"
    );
    assert!(
        warm_atlas[0] > cool_atlas[0] + 25,
        "expected the warm emissive texture to bias the capture redder than the cool emissive texture; warm_atlas={warm_atlas:?}, cool_atlas={cool_atlas:?}"
    );
    assert!(
        cool_atlas[2] > warm_atlas[2] + 25,
        "expected the cool emissive texture to bias the capture bluer than the warm emissive texture; warm_atlas={warm_atlas:?}, cool_atlas={cool_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_material_roughness() {
    let (asset_manager, root, smooth_white, rough_white, _, _) =
        material_surface_response_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 3.0)];

    let smooth = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                smooth_white,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let rough = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                rough_white,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let smooth_atlas = slot_sample(&smooth.0, 0, "atlas");
    let rough_atlas = slot_sample(&rough.0, 0, "atlas");
    let _ = fs::remove_dir_all(root);
    let smooth_luma =
        u16::from(smooth_atlas[0]) + u16::from(smooth_atlas[1]) + u16::from(smooth_atlas[2]);
    let rough_luma =
        u16::from(rough_atlas[0]) + u16::from(rough_atlas[1]) + u16::from(rough_atlas[2]);

    assert_ne!(
        smooth_atlas, rough_atlas,
        "expected material roughness to affect card-capture shading instead of collapsing to the same base-color sample"
    );
    assert!(
        smooth_luma.abs_diff(rough_luma) > 18,
        "expected roughness to produce a visibly different card-capture response, not just a quantization blip; smooth_atlas={smooth_atlas:?}, rough_atlas={rough_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_samples_change_with_material_normal_texture() {
    let MaterialTextureCaptureTestAssets {
        asset_manager,
        root,
        flat_normal,
        tilted_normal,
        ..
    } = material_texture_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 3.0)];

    let flat = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                flat_normal,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let tilted = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                tilted_normal,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let flat_atlas = slot_sample(&flat.0, 0, "atlas");
    let tilted_atlas = slot_sample(&tilted.0, 0, "atlas");
    let flat_luma = u16::from(flat_atlas[0]) + u16::from(flat_atlas[1]) + u16::from(flat_atlas[2]);
    let tilted_luma =
        u16::from(tilted_atlas[0]) + u16::from(tilted_atlas[1]) + u16::from(tilted_atlas[2]);
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        flat_atlas, tilted_atlas,
        "expected normal texture content to affect card-capture shading even when scalar material values stay fixed"
    );
    assert!(
        flat_luma.abs_diff(tilted_luma) > 20,
        "expected normal texture content to produce a visibly different card-capture response; flat_atlas={flat_atlas:?}, tilted_atlas={tilted_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_respects_material_double_sided_backface_lighting() {
    let MaterialVisibilityCaptureTestAssets {
        asset_manager,
        root,
        single_sided_white,
        double_sided_white,
        ..
    } = material_visibility_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 3.0)];
    let backface_transform =
        Transform::identity().with_rotation(Quat::from_rotation_y(std::f32::consts::PI));

    let single_sided = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                single_sided_white,
                backface_transform,
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let double_sided = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                double_sided_white,
                backface_transform,
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let single_atlas = slot_sample(&single_sided.0, 0, "atlas");
    let double_atlas = slot_sample(&double_sided.0, 0, "atlas");
    let single_luma =
        u16::from(single_atlas[0]) + u16::from(single_atlas[1]) + u16::from(single_atlas[2]);
    let double_luma =
        u16::from(double_atlas[0]) + u16::from(double_atlas[1]) + u16::from(double_atlas[2]);
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        single_atlas, double_atlas,
        "expected backface card capture shading to change when material double_sided changes instead of treating both materials as equally lit"
    );
    assert!(
        double_luma > single_luma + 35,
        "expected the double-sided material to preserve materially more backface light than the single-sided material; single_atlas={single_atlas:?}, double_atlas={double_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_respects_material_alpha_mask() {
    let MaterialVisibilityCaptureTestAssets {
        asset_manager,
        root,
        opaque_white,
        masked_cutout_white,
        ..
    } = material_visibility_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 3.0)];

    let opaque = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                opaque_white,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let masked = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                masked_cutout_white,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let opaque_atlas = slot_sample(&opaque.0, 0, "atlas");
    let masked_atlas = slot_sample(&masked.0, 0, "atlas");
    let opaque_luma =
        u16::from(opaque_atlas[0]) + u16::from(opaque_atlas[1]) + u16::from(opaque_atlas[2]);
    let masked_luma =
        u16::from(masked_atlas[0]) + u16::from(masked_atlas[1]) + u16::from(masked_atlas[2]);
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        opaque_atlas, masked_atlas,
        "expected masked alpha-mode materials to cut card capture energy instead of shading like fully opaque materials"
    );
    assert!(
        opaque_luma > masked_luma + 45,
        "expected the alpha-mask cutoff to reject most of the card-capture energy for this material; opaque_atlas={opaque_atlas:?}, masked_atlas={masked_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_card_capture_respects_material_alpha_blend() {
    let MaterialVisibilityCaptureTestAssets {
        asset_manager,
        root,
        opaque_white,
        blended_white,
        ..
    } = material_visibility_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_card_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 3.0)];

    let opaque = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                opaque_white,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let blended = render_scene_prepare_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                blended_white,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let opaque_atlas = slot_sample(&opaque.0, 0, "atlas");
    let blended_atlas = slot_sample(&blended.0, 0, "atlas");
    let opaque_luma =
        u16::from(opaque_atlas[0]) + u16::from(opaque_atlas[1]) + u16::from(opaque_atlas[2]);
    let blended_luma =
        u16::from(blended_atlas[0]) + u16::from(blended_atlas[1]) + u16::from(blended_atlas[2]);
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        opaque_atlas, blended_atlas,
        "expected alpha-blend materials to attenuate card capture energy instead of shading like fully opaque materials"
    );
    assert!(
        opaque_luma > blended_luma + 30,
        "expected the alpha-blend material to keep materially less card-capture energy than the opaque one; opaque_atlas={opaque_atlas:?}, blended_atlas={blended_atlas:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_samples_change_with_material_emissive() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();

    let black = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                black_material,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let emissive = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                emissive_material,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let black_voxel = slot_sample(&black.voxel_clipmap_rgba_samples, 7, "voxel");
    let emissive_voxel = slot_sample(&emissive.voxel_clipmap_rgba_samples, 7, "voxel");
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        black_voxel, emissive_voxel,
        "expected voxel clipmap debug samples to react to material emissive instead of staying identical"
    );
    assert!(
        emissive_voxel[0] > black_voxel[0] + 30,
        "expected emissive material to raise voxel clipmap sample red without direct lights; black_voxel={black_voxel:?}, emissive_voxel={emissive_voxel:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_samples_change_with_material_occlusion_texture() {
    let MaterialTextureCaptureTestAssets {
        asset_manager,
        root,
        open_occlusion,
        blocked_occlusion,
        ..
    } = material_texture_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();

    let open = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                open_occlusion,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let blocked = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                blocked_occlusion,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let open_voxel = slot_sample(&open.voxel_clipmap_rgba_samples, 7, "voxel");
    let blocked_voxel = slot_sample(&blocked.voxel_clipmap_rgba_samples, 7, "voxel");
    let open_luma = u16::from(open_voxel[0]) + u16::from(open_voxel[1]) + u16::from(open_voxel[2]);
    let blocked_luma =
        u16::from(blocked_voxel[0]) + u16::from(blocked_voxel[1]) + u16::from(blocked_voxel[2]);
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        open_voxel, blocked_voxel,
        "expected occlusion texture content to affect voxel clipmap shading even when scalar material values stay fixed"
    );
    assert!(
        open_luma > blocked_luma + 20,
        "expected the open occlusion texture to preserve noticeably more voxel energy than the blocked texture; open_voxel={open_voxel:?}, blocked_voxel={blocked_voxel:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_samples_change_with_material_emissive_texture() {
    let MaterialTextureCaptureTestAssets {
        asset_manager,
        root,
        emissive_warm,
        emissive_cool,
        ..
    } = material_texture_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();

    let warm = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                emissive_warm,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let cool = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                emissive_cool,
                Transform::identity(),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let warm_voxel = slot_sample(&warm.voxel_clipmap_rgba_samples, 7, "voxel");
    let cool_voxel = slot_sample(&cool.voxel_clipmap_rgba_samples, 7, "voxel");
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        warm_voxel, cool_voxel,
        "expected emissive texture content to affect voxel clipmap shading even when scalar emissive stays fixed"
    );
    assert!(
        warm_voxel[0] > cool_voxel[0] + 25,
        "expected the warm emissive texture to bias the voxel sample redder than the cool emissive texture; warm_voxel={warm_voxel:?}, cool_voxel={cool_voxel:?}"
    );
    assert!(
        cool_voxel[2] > warm_voxel[2] + 25,
        "expected the cool emissive texture to bias the voxel sample bluer than the warm emissive texture; warm_voxel={warm_voxel:?}, cool_voxel={cool_voxel:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_samples_change_with_material_metallic() {
    let (asset_manager, root, _, _, dielectric_red, metallic_red) =
        material_surface_response_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 3.0)];

    let dielectric = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                dielectric_red,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let metallic = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                metallic_red,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let dielectric_voxel = slot_sample(&dielectric.voxel_clipmap_rgba_samples, 7, "voxel");
    let metallic_voxel = slot_sample(&metallic.voxel_clipmap_rgba_samples, 7, "voxel");
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        dielectric_voxel, metallic_voxel,
        "expected material metallic to affect voxel clipmap shading instead of collapsing to the same diffuse-only sample"
    );
    assert!(
        (u16::from(dielectric_voxel[0])
            + u16::from(dielectric_voxel[1])
            + u16::from(dielectric_voxel[2]))
        .abs_diff(
            u16::from(metallic_voxel[0])
                + u16::from(metallic_voxel[1])
                + u16::from(metallic_voxel[2]),
        ) > 40,
        "expected metallic to produce a materially different voxel clipmap radiance response, not just a tiny channel wobble; dielectric_voxel={dielectric_voxel:?}, metallic_voxel={metallic_voxel:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_samples_change_with_material_metallic_roughness_texture() {
    let MaterialTextureCaptureTestAssets {
        asset_manager,
        root,
        rough_dielectric,
        smooth_metallic,
        ..
    } = material_texture_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();
    let lights = vec![directional_light(Vec3::ONE, 3.0)];

    let rough = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                rough_dielectric,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights.clone(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let smooth = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                11,
                model_handle(&asset_manager),
                smooth_metallic,
                Transform::identity(),
                Vec4::ONE,
            )],
            lights,
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let rough_voxel = slot_sample(&rough.voxel_clipmap_rgba_samples, 7, "voxel");
    let smooth_voxel = slot_sample(&smooth.voxel_clipmap_rgba_samples, 7, "voxel");
    let rough_luma =
        u16::from(rough_voxel[0]) + u16::from(rough_voxel[1]) + u16::from(rough_voxel[2]);
    let smooth_luma =
        u16::from(smooth_voxel[0]) + u16::from(smooth_voxel[1]) + u16::from(smooth_voxel[2]);
    let _ = fs::remove_dir_all(root);

    assert_ne!(
        rough_voxel, smooth_voxel,
        "expected metallic-roughness texture content to affect voxel clipmap shading even when scalar metallic and roughness stay fixed"
    );
    assert!(
        rough_luma.abs_diff(smooth_luma) > 30,
        "expected metallic-roughness texture content to produce a visibly different voxel response, not just a quantization blip; rough_voxel={rough_voxel:?}, smooth_voxel={smooth_voxel:?}"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_occupancy_changes_with_mesh_translation() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let left_meshes = vec![mesh_with_transform_and_tint(
        11,
        Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
        Vec4::ONE,
    )];
    let left_scene_prepare = single_voxel_scene_prepare_for_meshes(&left_meshes);
    let right_meshes = vec![mesh_with_transform_and_tint(
        11,
        Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        Vec4::ONE,
    )];
    let right_scene_prepare = single_voxel_scene_prepare_for_meshes(&right_meshes);

    let left = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            left_meshes,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        left_scene_prepare,
    );
    let right = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            right_meshes,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        right_scene_prepare,
    );

    let left_mask = voxel_occupancy_mask(&left.voxel_clipmap_occupancy_masks, 7);
    let right_mask = voxel_occupancy_mask(&right.voxel_clipmap_occupancy_masks, 7);

    assert_ne!(
        left_mask, right_mask,
        "expected voxel occupancy mask to move when the same mesh translates across clipmap cells instead of staying fixed"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_cell_samples_follow_mesh_translation() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();

    let left = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_transform_and_tint(
                11,
                Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let right = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_transform_and_tint(
                11,
                Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let left_cell = voxel_cell_sample(&left.voxel_clipmap_cell_rgba_samples, 7, 21);
    let right_same_cell = voxel_cell_sample(&right.voxel_clipmap_cell_rgba_samples, 7, 21);
    let left_other_cell = voxel_cell_sample(&left.voxel_clipmap_cell_rgba_samples, 7, 22);
    let right_cell = voxel_cell_sample(&right.voxel_clipmap_cell_rgba_samples, 7, 22);

    assert_ne!(
        left_cell, right_same_cell,
        "expected voxel cell radiance sample to vacate the old cell when the mesh moves across the clipmap grid"
    );
    assert_ne!(
        left_other_cell, right_cell,
        "expected voxel cell radiance sample to appear in the translated cell instead of keeping the old cell-local volume content"
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_cell_occupancy_counts_accumulate_overlapping_meshes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let single_meshes = vec![mesh_with_transform_and_tint(
        11,
        Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
        Vec4::ONE,
    )];
    let single_scene_prepare = single_voxel_scene_prepare_for_meshes(&single_meshes);
    let overlapped_meshes = vec![
        mesh_with_transform_and_tint(
            11,
            Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
            Vec4::ONE,
        ),
        mesh_with_transform_and_tint(
            12,
            Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
            Vec4::ONE,
        ),
    ];
    let overlapped_scene_prepare = single_voxel_scene_prepare_for_meshes(&overlapped_meshes);

    let single = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            single_meshes,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        single_scene_prepare,
    );
    let overlapped = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            overlapped_meshes,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        overlapped_scene_prepare,
    );

    assert_eq!(
        voxel_cell_occupancy_count(&single.voxel_clipmap_cell_occupancy_counts, 7, 21),
        1,
        "expected a single mesh to contribute exactly one occupancy count to its representative left-side cell"
    );
    assert_eq!(
        voxel_cell_occupancy_count(&overlapped.voxel_clipmap_cell_occupancy_counts, 7, 21),
        2,
        "expected overlapping meshes to accumulate occupancy count in the same voxel cell instead of collapsing back to a boolean mask"
    );
    assert_eq!(
        voxel_cell_occupancy_count(&overlapped.voxel_clipmap_cell_occupancy_counts, 7, 22),
        0,
        "expected the overlapped left-side meshes to stay out of the neighboring right-side cell"
    );
}

#[test]
fn hybrid_gi_scene_prepare_uses_runtime_voxel_cell_payload_without_scene_meshes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let snapshot = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 8.0,
            }],
            voxel_cells: vec![
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 21,
                    occupancy_count: 2,
                    dominant_card_id: 111,
                    radiance_present: true,
                    radiance_rgb: [32, 96, 224],
                },
                HybridGiPrepareVoxelCell {
                    clipmap_id: 7,
                    cell_index: 22,
                    occupancy_count: 1,
                    dominant_card_id: 222,
                    radiance_present: true,
                    radiance_rgb: [224, 64, 48],
                },
            ],
        },
    );

    assert_eq!(
        voxel_occupancy_mask(&snapshot.voxel_clipmap_occupancy_masks, 7),
        (1_u64 << 21) | (1_u64 << 22),
        "expected renderer scene-prepare occupancy mask to come from runtime-owned voxel cells even when no scene meshes are available locally"
    );
    assert_eq!(
        voxel_cell_occupancy_count(&snapshot.voxel_clipmap_cell_occupancy_counts, 7, 21),
        2,
        "expected runtime-provided occupancy counts to survive renderer prepare instead of collapsing back to zero when scene meshes are absent"
    );
    assert_eq!(
        voxel_cell_occupancy_count(&snapshot.voxel_clipmap_cell_occupancy_counts, 7, 22),
        1
    );
    assert_eq!(
        voxel_cell_occupancy_count(&snapshot.voxel_clipmap_cell_occupancy_counts, 7, 20),
        0
    );
    assert_eq!(
        slot_sample(&snapshot.voxel_clipmap_rgba_samples, 7, "voxel"),
        [96, 85, 165, 255],
        "expected renderer scene-prepare clipmap RGBA sample to aggregate runtime voxel radiance when no scene meshes are available locally"
    );
    assert_eq!(
        voxel_cell_sample(&snapshot.voxel_clipmap_cell_rgba_samples, 7, 21),
        [32, 96, 224, 255],
        "expected renderer scene-prepare cell RGBA samples to honor runtime voxel radiance even when no scene meshes are available locally"
    );
    assert_eq!(
        voxel_cell_sample(&snapshot.voxel_clipmap_cell_rgba_samples, 7, 22),
        [224, 64, 48, 255]
    );
    assert_eq!(
        voxel_cell_dominant_sample(&snapshot.voxel_clipmap_cell_dominant_rgba_samples, 7, 21),
        [32, 96, 224, 255],
        "expected renderer scene-prepare dominant RGBA samples to honor runtime voxel radiance even when no scene meshes are available locally"
    );
    assert_eq!(
        voxel_cell_dominant_sample(&snapshot.voxel_clipmap_cell_dominant_rgba_samples, 7, 22),
        [224, 64, 48, 255]
    );
    assert_eq!(
        voxel_cell_dominant_node_id(&snapshot.voxel_clipmap_cell_dominant_node_ids, 7, 21),
        111,
        "expected renderer scene-prepare dominant-node ids to honor runtime voxel authority even when no scene meshes are available locally"
    );
    assert_eq!(
        voxel_cell_dominant_node_id(&snapshot.voxel_clipmap_cell_dominant_node_ids, 7, 22),
        222
    );
}

#[test]
fn hybrid_gi_scene_prepare_reuses_persisted_surface_cache_page_contents_without_card_capture_requests(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let snapshot = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![
                HybridGiPrepareSurfaceCachePageContent {
                    page_id: 21,
                    owner_card_id: 21,
                    atlas_slot_id: 0,
                    capture_slot_id: 1,
                    bounds_center: Vec3::ZERO,
                    bounds_radius: 0.5,
                    atlas_sample_rgba: [224, 96, 48, 255],
                    capture_sample_rgba: [240, 112, 64, 255],
                },
                HybridGiPrepareSurfaceCachePageContent {
                    page_id: 22,
                    owner_card_id: 22,
                    atlas_slot_id: 9,
                    capture_slot_id: 4,
                    bounds_center: Vec3::new(2.0, 0.0, 0.0),
                    bounds_radius: 0.75,
                    atlas_sample_rgba: [48, 96, 224, 255],
                    capture_sample_rgba: [64, 112, 240, 255],
                },
            ],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    assert_eq!(
        snapshot.atlas_slot_rgba_samples,
        vec![(0, [224, 96, 48, 255]), (9, [48, 96, 224, 255])]
    );
    assert_eq!(
        snapshot.capture_slot_rgba_samples,
        vec![(1, [240, 112, 64, 255]), (4, [64, 112, 240, 255])]
    );
}

#[test]
fn hybrid_gi_scene_prepare_absent_persisted_surface_cache_page_contents_do_not_create_resource_snapshot_without_other_scene_prepare_data(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let snapshot = render_optional_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 21,
                owner_card_id: 21,
                atlas_slot_id: 0,
                capture_slot_id: 1,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                atlas_sample_rgba: [0, 0, 0, 0],
                capture_sample_rgba: [0, 0, 0, 0],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    assert!(
        snapshot.is_none(),
        "expected absent persisted surface-cache page samples to match the no-page scene-prepare baseline instead of creating an empty-but-authoritative resource snapshot"
    );
}

#[test]
fn hybrid_gi_scene_prepare_absent_persisted_surface_cache_page_contents_do_not_occupy_atlas_or_capture_slots(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let snapshot = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 21,
                owner_card_id: 21,
                atlas_slot_id: 7,
                capture_slot_id: 5,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                atlas_sample_rgba: [0, 0, 0, 0],
                capture_sample_rgba: [0, 0, 0, 0],
            }],
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 8.0,
            }],
            voxel_cells: Vec::new(),
        },
    );

    assert!(
        snapshot.occupied_atlas_slots.is_empty(),
        "expected absent persisted page samples to stay out of occupied atlas slots even when other scene-prepare resources keep the snapshot alive"
    );
    assert!(
        snapshot.occupied_capture_slots.is_empty(),
        "expected absent persisted page samples to stay out of occupied capture slots even when other scene-prepare resources keep the snapshot alive"
    );
    assert_eq!(snapshot.atlas_slot_count, 0);
    assert_eq!(snapshot.capture_slot_count, 0);
    assert!(
        snapshot.atlas_slot_rgba_samples.is_empty(),
        "expected absent persisted page samples to avoid atlas RGBA readback authority"
    );
    assert!(
        snapshot.capture_slot_rgba_samples.is_empty(),
        "expected absent persisted page samples to avoid capture RGBA readback authority"
    );
    assert_eq!(
        slot_sample(&snapshot.voxel_clipmap_rgba_samples, 7, "voxel"),
        [0, 0, 0, 0],
        "expected the snapshot to stay alive only because of voxel data, not because absent persisted pages fabricated atlas/capture support"
    );
}

#[test]
fn hybrid_gi_scene_prepare_atlas_only_persisted_surface_cache_page_contents_do_not_occupy_capture_slots(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let snapshot = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 21,
                owner_card_id: 21,
                atlas_slot_id: 7,
                capture_slot_id: 5,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                atlas_sample_rgba: [224, 96, 48, 255],
                capture_sample_rgba: [0, 0, 0, 0],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    assert_eq!(snapshot.occupied_atlas_slots, vec![7]);
    assert!(snapshot.occupied_capture_slots.is_empty());
    assert_eq!(snapshot.atlas_slot_count, 8);
    assert_eq!(snapshot.capture_slot_count, 0);
    assert_eq!(
        snapshot.atlas_slot_rgba_samples,
        vec![(7, [224, 96, 48, 255])]
    );
    assert!(
        snapshot.capture_slot_rgba_samples.is_empty(),
        "expected atlas-only persisted page samples to avoid fabricating capture-slot authority"
    );
}

#[test]
fn hybrid_gi_scene_prepare_capture_only_persisted_surface_cache_page_contents_do_not_occupy_atlas_slots(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let snapshot = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: vec![HybridGiPrepareSurfaceCachePageContent {
                page_id: 21,
                owner_card_id: 21,
                atlas_slot_id: 7,
                capture_slot_id: 5,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                atlas_sample_rgba: [0, 0, 0, 0],
                capture_sample_rgba: [240, 112, 64, 255],
            }],
            voxel_clipmaps: Vec::new(),
            voxel_cells: Vec::new(),
        },
    );

    assert!(snapshot.occupied_atlas_slots.is_empty());
    assert_eq!(snapshot.occupied_capture_slots, vec![5]);
    assert_eq!(snapshot.atlas_slot_count, 0);
    assert_eq!(snapshot.capture_slot_count, 6);
    assert!(
        snapshot.atlas_slot_rgba_samples.is_empty(),
        "expected capture-only persisted page samples to avoid fabricating atlas-slot authority"
    );
    assert_eq!(
        snapshot.capture_slot_rgba_samples,
        vec![(5, [240, 112, 64, 255])]
    );
}

#[test]
fn hybrid_gi_scene_prepare_preserves_explicit_black_runtime_voxel_radiance_without_scene_meshes() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let present_black = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 8.0,
            }],
            voxel_cells: vec![HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 21,
                occupancy_count: 2,
                dominant_card_id: 111,
                radiance_present: true,
                radiance_rgb: [0, 0, 0],
            }],
        },
    );
    let absent = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
        ),
        prepare_frame(),
        HybridGiScenePrepareFrame {
            card_capture_requests: Vec::new(),
            surface_cache_page_contents: Vec::new(),
            voxel_clipmaps: vec![HybridGiPrepareVoxelClipmap {
                clipmap_id: 7,
                center: Vec3::ZERO,
                half_extent: 8.0,
            }],
            voxel_cells: vec![HybridGiPrepareVoxelCell {
                clipmap_id: 7,
                cell_index: 21,
                occupancy_count: 2,
                dominant_card_id: 111,
                radiance_present: false,
                radiance_rgb: [0, 0, 0],
            }],
        },
    );

    assert_eq!(
        slot_sample(&present_black.voxel_clipmap_rgba_samples, 7, "voxel"),
        [0, 0, 0, 255],
        "expected explicit-black runtime voxel radiance to stay present in clipmap aggregate readback instead of collapsing to absence"
    );
    assert_eq!(
        voxel_cell_sample(&present_black.voxel_clipmap_cell_rgba_samples, 7, 21),
        [0, 0, 0, 255],
        "expected explicit-black runtime voxel radiance to stay present in cell readback instead of collapsing to absence"
    );
    assert_eq!(
        voxel_cell_dominant_sample(&present_black.voxel_clipmap_cell_dominant_rgba_samples, 7, 21),
        [0, 0, 0, 255],
        "expected explicit-black runtime voxel radiance to stay present in dominant cell readback instead of collapsing to absence"
    );
    assert_eq!(
        slot_sample(&absent.voxel_clipmap_rgba_samples, 7, "voxel"),
        [0, 0, 0, 0],
        "expected missing runtime voxel radiance authority to stay absent when no renderer-local scene meshes exist"
    );
    assert_eq!(
        voxel_cell_sample(&absent.voxel_clipmap_cell_rgba_samples, 7, 21),
        [0, 0, 0, 0]
    );
    assert_eq!(
        voxel_cell_dominant_sample(&absent.voxel_clipmap_cell_dominant_rgba_samples, 7, 21),
        [0, 0, 0, 0]
    );
}

#[test]
fn hybrid_gi_scene_prepare_requires_runtime_voxel_cells_for_occupancy_and_count_truth() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);

    let snapshot = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_transform_and_tint(
                11,
                Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare_frame(),
        single_voxel_scene_prepare(),
    );

    assert_eq!(
        voxel_occupancy_mask(&snapshot.voxel_clipmap_occupancy_masks, 7),
        0,
        "expected renderer scene-prepare occupancy mask to stay empty when runtime did not provide voxel_cells, even if scene meshes are present"
    );
    assert!(
        snapshot
            .voxel_clipmap_cell_occupancy_counts
            .iter()
            .all(|&(clipmap_id, _, occupancy_count)| clipmap_id != 7 || occupancy_count == 0),
        "expected renderer scene-prepare occupancy counts to stay empty when runtime did not provide voxel_cells, even if scene meshes are present; counts={:?}",
        snapshot.voxel_clipmap_cell_occupancy_counts
    );
}

#[test]
fn hybrid_gi_scene_prepare_voxel_cell_dominant_node_prefers_brighter_overlap() {
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();

    let snapshot = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![
                mesh_with_handles_and_tint(
                    11,
                    model_handle(&asset_manager),
                    black_material,
                    Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
                    Vec4::ONE,
                ),
                mesh_with_handles_and_tint(
                    12,
                    model_handle(&asset_manager),
                    emissive_material,
                    Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
                    Vec4::ONE,
                ),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    assert_eq!(
        voxel_cell_dominant_node_id(&snapshot.voxel_clipmap_cell_dominant_node_ids, 7, 21),
        12,
        "expected the emissive overlapping mesh to become the dominant cell contributor instead of leaving dominance on the darker overlap"
    );
    assert_eq!(
        voxel_cell_dominant_node_id(&snapshot.voxel_clipmap_cell_dominant_node_ids, 7, 22),
        0,
        "expected empty neighboring cells to keep a zero dominant-node marker"
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn hybrid_gi_scene_prepare_voxel_cell_dominant_sample_matches_brighter_overlap() {
    let (asset_manager, root, _black_material, emissive_material) = material_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let prepare = prepare_frame();
    let scene_prepare = single_voxel_scene_prepare();

    let emissive_only = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![mesh_with_handles_and_tint(
                12,
                model_handle(&asset_manager),
                emissive_material,
                Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
                Vec4::ONE,
            )],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare.clone(),
        scene_prepare.clone(),
    );
    let overlap = render_scene_prepare_resource_snapshot(
        &mut renderer,
        viewport_size,
        build_extract_with_scene(
            viewport_size,
            vec![probe(200, 96, Vec3::ZERO, 0.85)],
            vec![trace_region(40, Vec3::ZERO, 0.8, 0.9)],
            vec![
                mesh_with_transform_and_tint(
                    11,
                    Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
                    Vec4::ONE,
                ),
                mesh_with_handles_and_tint(
                    12,
                    model_handle(&asset_manager),
                    emissive_material,
                    Transform::from_translation(Vec3::new(-3.0, 0.0, 0.0)),
                    Vec4::ONE,
                ),
            ],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ),
        prepare,
        scene_prepare,
    );

    let overlap_dominant_sample =
        voxel_cell_dominant_sample(&overlap.voxel_clipmap_cell_dominant_rgba_samples, 7, 21);
    assert_eq!(
        overlap_dominant_sample,
        voxel_cell_sample(&emissive_only.voxel_clipmap_cell_rgba_samples, 7, 21),
        "expected the dominant voxel-cell sample to preserve the brighter emissive contributor instead of baking the darker overlap into the authority color"
    );
    assert_ne!(
        voxel_cell_sample(&overlap.voxel_clipmap_cell_rgba_samples, 7, 21),
        overlap_dominant_sample,
        "expected aggregate voxel-cell radiance to stay distinct from dominant-authority radiance when darker overlapping meshes also contribute to the cell"
    );
    assert_eq!(
        voxel_cell_dominant_sample(&overlap.voxel_clipmap_cell_dominant_rgba_samples, 7, 22),
        [0, 0, 0, 255],
        "expected empty neighboring cells to keep the zero-radiance dominant sample"
    );

    let _ = fs::remove_dir_all(root);
}

fn expected_card_capture_debug_rgba(
    card_id: u32,
    page_id: u32,
    atlas_slot_id: u32,
    capture_slot_id: u32,
) -> [u8; 4] {
    [
        (32 + ((card_id * 17 + atlas_slot_id * 13) % 192)) as u8,
        (32 + ((page_id * 11 + capture_slot_id * 7) % 192)) as u8,
        (32 + ((card_id * 5 + page_id * 3 + capture_slot_id * 19) % 192)) as u8,
        255,
    ]
}

fn slot_sample(samples: &[(u32, [u8; 4])], slot_id: u32, label: &str) -> [u8; 4] {
    samples
        .iter()
        .find(|(occupied_slot, _)| *occupied_slot == slot_id)
        .map(|(_, rgba)| *rgba)
        .unwrap_or_else(|| panic!("expected {label} slot sample"))
}

fn voxel_occupancy_mask(masks: &[(u32, u64)], clipmap_id: u32) -> u64 {
    masks
        .iter()
        .find(|(occupied_clipmap_id, _)| *occupied_clipmap_id == clipmap_id)
        .map(|(_, mask)| *mask)
        .unwrap_or_else(|| panic!("expected voxel occupancy mask"))
}

fn voxel_cell_sample(samples: &[(u32, u32, [u8; 4])], clipmap_id: u32, cell_index: u32) -> [u8; 4] {
    samples
        .iter()
        .find(|(occupied_clipmap_id, occupied_cell_index, _)| {
            *occupied_clipmap_id == clipmap_id && *occupied_cell_index == cell_index
        })
        .map(|(_, _, rgba)| *rgba)
        .unwrap_or_else(|| panic!("expected voxel cell sample"))
}

fn voxel_cell_occupancy_count(counts: &[(u32, u32, u32)], clipmap_id: u32, cell_index: u32) -> u32 {
    counts
        .iter()
        .find(|(occupied_clipmap_id, occupied_cell_index, _)| {
            *occupied_clipmap_id == clipmap_id && *occupied_cell_index == cell_index
        })
        .map(|(_, _, occupancy_count)| *occupancy_count)
        .unwrap_or_else(|| panic!("expected voxel cell occupancy count"))
}

fn voxel_cell_dominant_node_id(ids: &[(u32, u32, u64)], clipmap_id: u32, cell_index: u32) -> u64 {
    ids.iter()
        .find(|(occupied_clipmap_id, occupied_cell_index, _)| {
            *occupied_clipmap_id == clipmap_id && *occupied_cell_index == cell_index
        })
        .map(|(_, _, node_id)| *node_id)
        .unwrap_or_else(|| panic!("expected voxel cell dominant node id"))
}

fn voxel_cell_dominant_sample(
    samples: &[(u32, u32, [u8; 4])],
    clipmap_id: u32,
    cell_index: u32,
) -> [u8; 4] {
    samples
        .iter()
        .find(|(occupied_clipmap_id, occupied_cell_index, _)| {
            *occupied_clipmap_id == clipmap_id && *occupied_cell_index == cell_index
        })
        .map(|(_, _, rgba)| *rgba)
        .unwrap_or_else(|| panic!("expected voxel cell dominant sample"))
}

fn prepare_frame() -> HybridGiPrepareFrame {
    HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 96,
            irradiance_rgb: [96, 96, 96],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: vec![40],
        evictable_probe_ids: Vec::new(),
    }
}

fn single_card_scene_prepare() -> HybridGiScenePrepareFrame {
    HybridGiScenePrepareFrame {
        card_capture_requests: vec![HybridGiPrepareCardCaptureRequest {
            card_id: 11,
            page_id: 21,
            atlas_slot_id: 0,
            capture_slot_id: 1,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.9,
        }],
        surface_cache_page_contents: Vec::new(),
        voxel_clipmaps: Vec::new(),
        voxel_cells: Vec::new(),
    }
}

fn single_voxel_scene_prepare() -> HybridGiScenePrepareFrame {
    single_voxel_scene_prepare_with_voxel_cells(Vec::new())
}

fn single_voxel_scene_prepare_for_meshes(
    meshes: &[RenderMeshSnapshot],
) -> HybridGiScenePrepareFrame {
    let clipmap = single_voxel_clipmap();
    let mut occupancy_counts = [0_u32; 64];
    for mesh in meshes {
        let bounds_radius = (mesh.transform.scale.abs().max_element() * 0.5)
            .max(TEST_SCENE_PREPARE_VOXEL_MIN_MESH_BOUNDS_RADIUS);
        let Some([(x_start, x_end), (y_start, y_end), (z_start, z_end)]) =
            hybrid_gi_voxel_clipmap_bounds_cell_ranges(
                &clipmap,
                mesh.transform.translation,
                bounds_radius,
            )
        else {
            continue;
        };

        for z in z_start..=z_end {
            for y in y_start..=y_end {
                for x in x_start..=x_end {
                    let cell_index = hybrid_gi_voxel_clipmap_cell_bit_index(x, y, z);
                    occupancy_counts[cell_index] = occupancy_counts[cell_index].saturating_add(1);
                }
            }
        }
    }

    let voxel_cells = occupancy_counts
        .into_iter()
        .enumerate()
        .filter_map(|(cell_index, occupancy_count)| {
            (occupancy_count > 0).then_some(HybridGiPrepareVoxelCell {
                clipmap_id: clipmap.clipmap_id,
                cell_index: cell_index as u32,
                occupancy_count,
                dominant_card_id: 0,
                radiance_present: true,
                radiance_rgb: [255, 255, 255],
            })
        })
        .collect();

    single_voxel_scene_prepare_with_voxel_cells(voxel_cells)
}

fn single_voxel_scene_prepare_with_voxel_cells(
    voxel_cells: Vec<HybridGiPrepareVoxelCell>,
) -> HybridGiScenePrepareFrame {
    HybridGiScenePrepareFrame {
        card_capture_requests: Vec::new(),
        surface_cache_page_contents: Vec::new(),
        voxel_clipmaps: vec![single_voxel_clipmap()],
        voxel_cells,
    }
}

fn single_voxel_clipmap() -> HybridGiPrepareVoxelClipmap {
    HybridGiPrepareVoxelClipmap {
        clipmap_id: 7,
        center: Vec3::ZERO,
        half_extent: 8.0,
    }
}

fn render_scene_prepare_resource_snapshot(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    scene_prepare: HybridGiScenePrepareFrame,
) -> ScenePrepareResourceSnapshotForTest {
    render_optional_scene_prepare_resource_snapshot(
        renderer,
        viewport_size,
        extract,
        prepare,
        scene_prepare,
    )
    .expect("expected scene-prepare resource snapshot")
}

fn render_optional_scene_prepare_resource_snapshot(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    scene_prepare: HybridGiScenePrepareFrame,
) -> Option<ScenePrepareResourceSnapshotForTest> {
    let compiled = compile_hybrid_gi_pipeline(&extract);

    renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let snapshot = renderer
        .take_last_hybrid_gi_gpu_readback()
        .expect("expected hybrid gi GPU readback")
        .scene_prepare_resources();
    snapshot.map(|snapshot| ScenePrepareResourceSnapshotForTest {
        occupied_atlas_slots: snapshot.occupied_atlas_slots().to_vec(),
        occupied_capture_slots: snapshot.occupied_capture_slots().to_vec(),
        atlas_slot_rgba_samples: snapshot.atlas_slot_rgba_samples().to_vec(),
        capture_slot_rgba_samples: snapshot.capture_slot_rgba_samples().to_vec(),
        atlas_slot_count: snapshot.atlas_slot_count(),
        capture_slot_count: snapshot.capture_slot_count(),
        voxel_clipmap_rgba_samples: snapshot.voxel_clipmap_rgba_samples().to_vec(),
        voxel_clipmap_occupancy_masks: snapshot.voxel_clipmap_occupancy_masks().to_vec(),
        voxel_clipmap_cell_rgba_samples: snapshot.voxel_clipmap_cell_rgba_samples().to_vec(),
        voxel_clipmap_cell_occupancy_counts: snapshot
            .voxel_clipmap_cell_occupancy_counts()
            .to_vec(),
        voxel_clipmap_cell_dominant_node_ids: snapshot
            .voxel_clipmap_cell_dominant_node_ids()
            .to_vec(),
        voxel_clipmap_cell_dominant_rgba_samples: snapshot
            .voxel_clipmap_cell_dominant_rgba_samples()
            .to_vec(),
    })
}

fn render_scene_prepare_snapshot(
    renderer: &mut SceneRenderer,
    viewport_size: UVec2,
    extract: RenderFrameExtract,
    prepare: HybridGiPrepareFrame,
    scene_prepare: HybridGiScenePrepareFrame,
) -> (Vec<(u32, [u8; 4])>, Vec<(u32, [u8; 4])>) {
    let snapshot = render_scene_prepare_resource_snapshot(
        renderer,
        viewport_size,
        extract,
        prepare,
        scene_prepare,
    );
    (
        snapshot.atlas_slot_rgba_samples,
        snapshot.capture_slot_rgba_samples,
    )
}

fn build_extract(
    viewport_size: UVec2,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
) -> RenderFrameExtract {
    build_extract_with_scene(
        viewport_size,
        probes,
        trace_regions,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn build_extract_with_scene(
    viewport_size: UVec2,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
    meshes: Vec<RenderMeshSnapshot>,
    directional_lights: Vec<RenderDirectionalLightSnapshot>,
    point_lights: Vec<RenderPointLightSnapshot>,
    spot_lights: Vec<RenderSpotLightSnapshot>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes = meshes;
    snapshot.scene.directional_lights = directional_lights;
    snapshot.scene.point_lights = point_lights;
    snapshot.scene.spot_lights = spot_lights;
    let mut extract =
        RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(1), snapshot);
    extract.apply_viewport_size(viewport_size);
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 0,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 1,
        probes,
        trace_regions,
    });
    extract
}

fn probe(probe_id: u32, ray_budget: u32, position: Vec3, radius: f32) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position,
        radius,
        parent_probe_id: None,
        resident: true,
        ray_budget,
    }
}

fn mesh_with_tint(node_id: u64, tint: Vec4) -> RenderMeshSnapshot {
    mesh_with_material_and_tint(node_id, "builtin://material/default", tint)
}

fn mesh_with_transform_and_tint(
    node_id: u64,
    transform: Transform,
    tint: Vec4,
) -> RenderMeshSnapshot {
    mesh_with_material_transform_and_tint(node_id, "builtin://material/default", transform, tint)
}

fn mesh_with_material_and_tint(node_id: u64, material_uri: &str, tint: Vec4) -> RenderMeshSnapshot {
    mesh_with_material_transform_and_tint(node_id, material_uri, Transform::identity(), tint)
}

fn mesh_with_material_transform_and_tint(
    node_id: u64,
    material_uri: &str,
    transform: Transform,
    tint: Vec4,
) -> RenderMeshSnapshot {
    mesh_with_handles_and_tint(
        node_id,
        ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(material_uri)),
        transform,
        tint,
    )
}

fn mesh_with_handles_and_tint(
    node_id: u64,
    model: ResourceHandle<ModelMarker>,
    material: ResourceHandle<MaterialMarker>,
    transform: Transform,
    tint: Vec4,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        transform,
        model,
        material,
        tint,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }
}

fn directional_light(color: Vec3, intensity: f32) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id: 1,
        direction: Vec3::new(0.0, 0.0, -1.0),
        color,
        intensity,
    }
}

fn point_light(
    node_id: u64,
    color: Vec3,
    intensity: f32,
    position: Vec3,
    range: f32,
) -> RenderPointLightSnapshot {
    RenderPointLightSnapshot {
        node_id,
        position,
        color,
        intensity,
        range,
    }
}

fn spot_light(
    node_id: u64,
    position: Vec3,
    direction: Vec3,
    color: Vec3,
    intensity: f32,
    range: f32,
    inner_angle_radians: f32,
    outer_angle_radians: f32,
) -> RenderSpotLightSnapshot {
    RenderSpotLightSnapshot {
        node_id,
        position,
        direction,
        color,
        intensity,
        range,
        inner_angle_radians,
        outer_angle_radians,
    }
}

fn trace_region(
    region_id: u32,
    bounds_center: Vec3,
    bounds_radius: f32,
    screen_coverage: f32,
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity: 1,
        region_id,
        bounds_center,
        bounds_radius,
        screen_coverage,
        rt_lighting_rgb: [0, 0, 0],
    }
}
