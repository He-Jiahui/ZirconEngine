use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    RenderDirectionalLightSnapshot, RenderFrameExtract, RenderHybridGiExtract, RenderHybridGiProbe,
    RenderHybridGiTraceRegion, RenderMeshSnapshot, RenderSceneSnapshot, RenderSpotLightSnapshot,
    RenderWorldSnapshotHandle,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::scene::world::World;
use crate::test_support::render_feature_fixtures::hybrid_gi_render_feature_descriptor;
use crate::{
    runtime::HybridGiRuntimeState,
    types::{
        HybridGiPrepareFrame, HybridGiPrepareProbe, HybridGiScenePrepareFrame, ViewportRenderFrame,
    },
    BuiltinRenderFeature, CompiledRenderPipeline, RenderFeatureCapabilityRequirement,
    RenderPipelineAsset, RenderPipelineCompileOptions, SceneRenderer,
};

use super::hybrid_gi_scene_prepare_material_fixtures::{
    material_capture_test_assets, model_handle,
};

#[test]
fn hybrid_gi_resolve_uses_runtime_scene_voxel_directional_light_seed_when_layout_and_tint_stay_fixed(
) {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let warm_scene_prepare = runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
        [1.0, 1.0, 1.0],
        Vec3::ZERO,
        2.0,
        &[test_directional_light(
            30,
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.08, 0.04),
            3.0,
        )],
        &[],
    );
    let cool_scene_prepare = runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
        [1.0, 1.0, 1.0],
        Vec3::ZERO,
        2.0,
        &[test_directional_light(
            30,
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.04, 0.08, 1.0),
            3.0,
        )],
        &[],
    );

    assert_eq!(
        warm_scene_prepare.voxel_clipmaps,
        cool_scene_prepare.voxel_clipmaps
    );
    assert_eq!(
        voxel_layout(&warm_scene_prepare),
        voxel_layout(&cool_scene_prepare),
        "expected warm/cool directional-light fixtures to keep identical runtime voxel layout so this regression only checks direct-light seed authority"
    );
    assert_ne!(
        voxel_radiance(&warm_scene_prepare),
        voxel_radiance(&cool_scene_prepare),
        "expected runtime voxel cell radiance to change with directional-light direct seed even when layout and mesh tint stay fixed"
    );

    let prepare = resident_prepare();
    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected scene-driven runtime voxel fallback to preserve warm directional-light seed when voxel layout and mesh tint stay fixed; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected scene-driven runtime voxel fallback to preserve cool directional-light seed when voxel layout and mesh tint stay fixed; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_runtime_scene_voxel_spot_light_seed_when_layout_and_tint_stay_fixed() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let mut renderer = hybrid_gi_scene_renderer(asset_manager);
    let viewport_size = UVec2::new(96, 64);
    let extract = build_extract_with_probes_and_trace_regions(
        viewport_size,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
        Vec::new(),
    );
    let compiled = compile_hybrid_gi_pipeline(&extract);

    let warm_scene_prepare = runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
        [1.0, 1.0, 1.0],
        Vec3::ZERO,
        2.0,
        &[],
        &[test_spot_light(
            20,
            Vec3::new(0.0, 0.0, 0.35),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(1.0, 0.08, 0.04),
            4.0,
            3.0,
            0.2,
            0.65,
        )],
    );
    let cool_scene_prepare = runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
        [1.0, 1.0, 1.0],
        Vec3::ZERO,
        2.0,
        &[],
        &[test_spot_light(
            20,
            Vec3::new(0.0, 0.0, 0.35),
            Vec3::new(0.0, 0.0, -1.0),
            Vec3::new(0.04, 0.08, 1.0),
            4.0,
            3.0,
            0.2,
            0.65,
        )],
    );

    assert_eq!(
        warm_scene_prepare.voxel_clipmaps,
        cool_scene_prepare.voxel_clipmaps
    );
    assert_eq!(
        voxel_layout(&warm_scene_prepare),
        voxel_layout(&cool_scene_prepare),
        "expected warm/cool spot-light fixtures to keep identical runtime voxel layout so this regression only checks direct-light seed authority"
    );
    assert_ne!(
        voxel_radiance(&warm_scene_prepare),
        voxel_radiance(&cool_scene_prepare),
        "expected runtime voxel cell radiance to change with spot-light direct seed even when layout and mesh tint stay fixed"
    );

    let prepare = HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    };

    let warm = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract.clone(), viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(warm_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let cool = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(cool_scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();

    let warm_red = average_region_channel(&warm.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let cool_red = average_region_channel(&cool.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let warm_blue = average_region_channel(&warm.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);
    let cool_blue = average_region_channel(&cool.rgba, viewport_size, 2, 0.25, 0.75, 0.25, 0.75);

    assert!(
        warm_red > cool_red + 0.5,
        "expected scene-driven runtime voxel fallback to preserve warm spot-light seed when voxel layout and mesh tint stay fixed; warm_red={warm_red:.2}, cool_red={cool_red:.2}"
    );
    assert!(
        cool_blue > warm_blue + 0.5,
        "expected scene-driven runtime voxel fallback to preserve cool spot-light seed when voxel layout and mesh tint stay fixed; warm_blue={warm_blue:.2}, cool_blue={cool_blue:.2}"
    );
}

#[test]
fn hybrid_gi_resolve_uses_scene_card_capture_emissive_seed_when_voxel_owner_and_layout_stay_fixed()
{
    let (asset_manager, root, black_material, emissive_material) = material_capture_test_assets();
    let mut renderer = hybrid_gi_scene_renderer(asset_manager.clone());
    let viewport_size = UVec2::new(96, 64);
    let black_extract = build_extract_with_material_mesh(
        viewport_size,
        &asset_manager,
        black_material,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
    );
    let emissive_extract = build_extract_with_material_mesh(
        viewport_size,
        &asset_manager,
        emissive_material,
        vec![probe(200, true, 128, Vec3::ZERO, 1.8)],
    );
    let compiled = compile_hybrid_gi_pipeline(&black_extract);
    let prepare = resident_prepare();
    let scene_prepare = owner_card_voxel_scene_prepare();

    let black = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(black_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare.clone()))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare.clone())),
            &compiled,
            None,
        )
        .unwrap();
    let black_capture = renderer
        .take_last_hybrid_gi_gpu_readback()
        .unwrap()
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected black emissive capture sample");
    let emissive = renderer
        .render_frame_with_pipeline(
            &ViewportRenderFrame::from_extract(emissive_extract, viewport_size)
                .with_hybrid_gi_prepare(Some(prepare))
                .with_hybrid_gi_scene_prepare(Some(scene_prepare)),
            &compiled,
            None,
        )
        .unwrap();
    let emissive_capture = renderer
        .take_last_hybrid_gi_gpu_readback()
        .unwrap()
        .scene_prepare_resources()
        .and_then(|snapshot| {
            snapshot
                .capture_slot_rgba_samples()
                .iter()
                .find(|sample| sample.0 == 4)
                .map(|sample| sample.1)
        })
        .expect("expected emissive capture sample");
    let black_red = average_region_channel(&black.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let emissive_red =
        average_region_channel(&emissive.rgba, viewport_size, 0, 0.25, 0.75, 0.25, 0.75);
    let _ = std::fs::remove_dir_all(root);

    assert_ne!(
        black_capture, emissive_capture,
        "expected card-capture samples to change when only material emissive changes while request layout stays fixed"
    );
    assert!(
        emissive_red > black_red + 0.5,
        "expected final resolve to preserve emissive card-capture seed through owner-card voxel fallback when voxel owner and layout stay fixed; black_red={black_red:.2}, emissive_red={emissive_red:.2}, black_capture={black_capture:?}, emissive_capture={emissive_capture:?}"
    );
}

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

fn build_extract_with_probes_and_trace_regions(
    viewport_size: UVec2,
    probes: Vec<RenderHybridGiProbe>,
    trace_regions: Vec<RenderHybridGiTraceRegion>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes.clear();
    snapshot.scene.directional_lights.clear();
    snapshot.scene.point_lights.clear();
    snapshot.scene.spot_lights.clear();
    snapshot.preview.clear_color = Vec4::ZERO;
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
        probe_budget: 2,
        tracing_budget: 1,
        probes,
        trace_regions,
    });
    extract
}

fn build_extract_with_material_mesh(
    viewport_size: UVec2,
    asset_manager: &ProjectAssetManager,
    material: ResourceHandle<MaterialMarker>,
    probes: Vec<RenderHybridGiProbe>,
) -> RenderFrameExtract {
    let mut snapshot: RenderSceneSnapshot = World::new().to_render_snapshot();
    snapshot.scene.meshes = vec![RenderMeshSnapshot {
        node_id: 11,
        transform: Transform::from_translation(Vec3::new(10_000.0, 10_000.0, 10_000.0)),
        model: model_handle(asset_manager),
        material,
        tint: Vec4::ONE,
        mobility: Mobility::Static,
        render_layer_mask: u32::MAX,
    }];
    snapshot.scene.directional_lights.clear();
    snapshot.scene.point_lights.clear();
    snapshot.scene.spot_lights.clear();
    snapshot.preview.clear_color = Vec4::ZERO;
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
        probe_budget: 2,
        tracing_budget: 0,
        probes,
        trace_regions: Vec::new(),
    });
    extract
}

fn resident_prepare() -> HybridGiPrepareFrame {
    HybridGiPrepareFrame {
        resident_probes: vec![HybridGiPrepareProbe {
            probe_id: 200,
            slot: 0,
            ray_budget: 128,
            irradiance_rgb: [112, 112, 112],
        }],
        pending_updates: Vec::new(),
        scheduled_trace_region_ids: Vec::new(),
        evictable_probe_ids: Vec::new(),
    }
}

fn owner_card_voxel_scene_prepare() -> HybridGiScenePrepareFrame {
    HybridGiScenePrepareFrame {
        card_capture_requests: vec![crate::types::HybridGiPrepareCardCaptureRequest {
            card_id: 11,
            page_id: 22,
            atlas_slot_id: 3,
            capture_slot_id: 4,
            bounds_center: Vec3::new(20.0, 20.0, 20.0),
            bounds_radius: 0.25,
        }],
        surface_cache_page_contents: Vec::new(),
        voxel_clipmaps: vec![crate::types::HybridGiPrepareVoxelClipmap {
            clipmap_id: 7,
            center: Vec3::ZERO,
            half_extent: 4.0,
        }],
        voxel_cells: vec![
            owner_card_voxel_cell(20),
            owner_card_voxel_cell(21),
            owner_card_voxel_cell(24),
            owner_card_voxel_cell(25),
        ],
    }
}

fn owner_card_voxel_cell(cell_index: u32) -> crate::types::HybridGiPrepareVoxelCell {
    crate::types::HybridGiPrepareVoxelCell {
        clipmap_id: 7,
        cell_index,
        occupancy_count: 4,
        dominant_card_id: 11,
        radiance_present: false,
        radiance_rgb: [0, 0, 0],
    }
}

fn probe(
    probe_id: u32,
    resident: bool,
    ray_budget: u32,
    position: Vec3,
    radius: f32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity: 1,
        probe_id,
        position,
        radius,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn runtime_voxel_scene_prepare_from_tinted_mesh_and_lights(
    tint_rgb: [f32; 3],
    translation: Vec3,
    uniform_scale: f32,
    directional_lights: &[RenderDirectionalLightSnapshot],
    spot_lights: &[RenderSpotLightSnapshot],
) -> HybridGiScenePrepareFrame {
    let mut runtime = HybridGiRuntimeState::default();
    let extract = RenderHybridGiExtract {
        enabled: true,
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 0,
        voxel_budget: 1,
        debug_view: Default::default(),
        probe_budget: 0,
        tracing_budget: 0,
        probes: Vec::new(),
        trace_regions: Vec::new(),
    };

    runtime.register_scene_extract(
        Some(&extract),
        &[RenderMeshSnapshot {
            node_id: 11,
            transform: Transform::from_translation(translation)
                .with_scale(Vec3::splat(uniform_scale)),
            model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label(
                "res://models/card.obj",
            )),
            material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
                "res://materials/runtime-voxel-spot-light.mat",
            )),
            tint: Vec4::new(tint_rgb[0], tint_rgb[1], tint_rgb[2], 1.0),
            mobility: Mobility::Static,
            render_layer_mask: u32::MAX,
        }],
        directional_lights,
        &[],
        spot_lights,
    );

    runtime.build_scene_prepare_frame()
}

fn test_directional_light(
    node_id: u64,
    direction: Vec3,
    color: Vec3,
    intensity: f32,
) -> RenderDirectionalLightSnapshot {
    RenderDirectionalLightSnapshot {
        node_id,
        direction,
        color,
        intensity,
    }
}

fn test_spot_light(
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

fn voxel_layout(scene_prepare: &HybridGiScenePrepareFrame) -> Vec<(u32, u32, u32)> {
    scene_prepare
        .voxel_cells
        .iter()
        .map(|cell| (cell.clipmap_id, cell.cell_index, cell.occupancy_count))
        .collect()
}

fn voxel_radiance(scene_prepare: &HybridGiScenePrepareFrame) -> Vec<(u32, u32, [u8; 3])> {
    scene_prepare
        .voxel_cells
        .iter()
        .map(|cell| (cell.clipmap_id, cell.cell_index, cell.radiance_rgb))
        .collect()
}

fn average_region_channel(
    rgba: &[u8],
    viewport_size: UVec2,
    channel: usize,
    x_min: f32,
    x_max: f32,
    y_min: f32,
    y_max: f32,
) -> f32 {
    if rgba.is_empty() {
        return 0.0;
    }

    let width = viewport_size.x as usize;
    let height = viewport_size.y as usize;
    let start_x = ((width as f32) * x_min.clamp(0.0, 1.0)).floor() as usize;
    let end_x = ((width as f32) * x_max.clamp(0.0, 1.0)).ceil() as usize;
    let start_y = ((height as f32) * y_min.clamp(0.0, 1.0)).floor() as usize;
    let end_y = ((height as f32) * y_max.clamp(0.0, 1.0)).ceil() as usize;

    let mut total = 0.0;
    let mut count = 0usize;
    for y in start_y.min(height)..end_y.min(height).max(start_y.min(height) + 1) {
        for x in start_x.min(width)..end_x.min(width).max(start_x.min(width) + 1) {
            let pixel_index = (y * width + x) * 4;
            total += rgba[pixel_index + channel] as f32;
            count += 1;
        }
    }

    if count == 0 {
        return 0.0;
    }
    total / count as f32
}
