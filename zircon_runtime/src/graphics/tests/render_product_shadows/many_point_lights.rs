use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::asset::{AlphaMode, AssetReference, AssetUri, MaterialAsset};
use crate::core::framework::render::{
    CapturedFrame, FallbackSkyboxKind, GpuLightType, PostProcessGraphResourceNames,
    PreviewEnvironmentExtract, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderPipelineHandle, RenderPointLightSnapshot, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderWorldSnapshotHandle, ViewportCameraSnapshot, DEFAULT_RENDER_LAYER_MASK,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{
    MaterialMarker, ModelMarker, ResourceHandle, ResourceId, ResourceKind, ResourceRecord,
};
use crate::graphics::scene::{build_light_grid_for_frame, pack_lighting_extract};
use crate::graphics::{RenderPipelineAsset, WgpuRenderFramework};
use crate::render_graph::{RenderGraphResourceAccessKind, RenderGraphResourceKind};

use super::pass_resource_access;

#[test]
fn render_product_many_point_lights() {
    let extract = many_point_light_extract();
    let packed = pack_lighting_extract(&extract.lighting, true);

    assert_eq!(packed.point_count, 64);
    assert_eq!(packed.light_count(), 64);
    assert!(packed
        .lights
        .iter()
        .all(|light| { light.direction_type[3].to_bits() == GpuLightType::Point.as_u32() }));
    assert_eq!(packed.lights[0].shadow_slot_layer[2], 20_000);
    assert_eq!(packed.lights[63].shadow_slot_layer[2], 20_063);

    let light_grid = build_light_grid_for_frame(&extract, UVec2::new(640, 360), true);
    assert_eq!(light_grid.params.light_count, 64);
    assert_eq!(
        light_grid.params.words_per_tile, 2,
        "64 point lights should cross the old 32-bit/scene-uniform-style light word boundary"
    );
    assert!(light_grid.stats.non_empty_tile_count > 0);
    assert!(light_grid.stats.non_empty_zbin_count > 0);
    assert!(
        light_grid.stats.peak_lights_per_cluster >= 32,
        "the grid should shade dense point-light clusters rather than truncating after a tiny fixed set"
    );
    assert!(light_grid
        .tile_masks
        .chunks(light_grid.params.words_per_tile as usize)
        .any(|words| words[0] != 0 && words[1] != 0));

    let bin_stride = 2 + light_grid.params.words_per_tile as usize;
    assert!(light_grid.zbins.chunks(bin_stride).any(|bin| bin[3] != 0));

    let forward = RenderPipelineAsset::default_forward_plus()
        .compile(&extract)
        .unwrap();
    let deferred = RenderPipelineAsset::default_deferred()
        .compile(&extract)
        .unwrap();

    for compiled in [&forward, &deferred] {
        for resource_name in [
            PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
            PostProcessGraphResourceNames::LIGHT_ZBINS,
            PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
        ] {
            let write = pass_resource_access(
                compiled,
                "light-grid-build",
                resource_name,
                RenderGraphResourceAccessKind::Write,
            );
            assert_eq!(write.kind, RenderGraphResourceKind::TransientBuffer);
        }
    }

    for pass_name in ["opaque-mesh", "alpha-mask-mesh", "transparent-mesh"] {
        assert_light_grid_reads(&forward, pass_name);
    }
    assert_light_grid_reads(&deferred, "deferred-lighting");
}

#[test]
fn render_product_many_point_lights_forward_deferred_capture_parity() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material = register_many_point_light_product_material_asset(
        asset_manager.as_ref(),
        "res://materials/many_point_lights_product.zmaterial",
        many_point_light_product_material(),
    );
    let server = WgpuRenderFramework::new(asset_manager).unwrap();

    let (forward_baseline, _) = render_many_point_light_product_frame(
        &server,
        viewport_size,
        RenderPipelineHandle::new(1),
        "many-point-forward-baseline",
        many_point_light_product_extract(viewport_size, material, false),
    );
    let (forward_lit, forward_stats) = render_many_point_light_product_frame(
        &server,
        viewport_size,
        RenderPipelineHandle::new(1),
        "many-point-forward-lit",
        many_point_light_product_extract(viewport_size, material, true),
    );
    let (deferred_baseline, _) = render_many_point_light_product_frame(
        &server,
        viewport_size,
        RenderPipelineHandle::new(2),
        "many-point-deferred-baseline",
        many_point_light_product_extract(viewport_size, material, false),
    );
    let (deferred_lit, deferred_stats) = render_many_point_light_product_frame(
        &server,
        viewport_size,
        RenderPipelineHandle::new(2),
        "many-point-deferred-lit",
        many_point_light_product_extract(viewport_size, material, true),
    );

    assert_many_point_light_product_stats("forward", &forward_stats);
    assert_many_point_light_product_stats("deferred", &deferred_stats);
    assert!(
        deferred_stats
            .last_graph_executed_executor_ids
            .contains(&"lighting.deferred".to_string()),
        "deferred product submit should execute deferred lighting; executors={:?}",
        deferred_stats.last_graph_executed_executor_ids
    );

    let sample_origin = UVec2::new(viewport_size.x / 2 - 16, viewport_size.y / 2 - 16);
    let sample_size = UVec2::new(32, 32);
    let forward_baseline_luma =
        average_luma_in_region(&forward_baseline, sample_origin, sample_size);
    let forward_lit_luma = average_luma_in_region(&forward_lit, sample_origin, sample_size);
    let deferred_baseline_luma =
        average_luma_in_region(&deferred_baseline, sample_origin, sample_size);
    let deferred_lit_luma = average_luma_in_region(&deferred_lit, sample_origin, sample_size);

    assert!(
        forward_lit_luma > forward_baseline_luma + 12.0,
        "forward product capture should visibly shade the mesh with 64 point lights; baseline={forward_baseline_luma:.2}, lit={forward_lit_luma:.2}"
    );
    assert!(
        deferred_lit_luma > deferred_baseline_luma + 12.0,
        "deferred product capture should visibly shade the mesh with 64 point lights; baseline={deferred_baseline_luma:.2}, lit={deferred_lit_luma:.2}"
    );

    let forward_delta = forward_lit_luma - forward_baseline_luma;
    let deferred_delta = deferred_lit_luma - deferred_baseline_luma;
    let stronger_delta = forward_delta.max(deferred_delta);
    let weaker_delta = forward_delta.min(deferred_delta);
    assert!(
        weaker_delta >= stronger_delta * 0.45,
        "forward/deferred point-light contribution should stay in the same product range; forward_delta={forward_delta:.2}, deferred_delta={deferred_delta:.2}"
    );
}

#[test]
fn render_product_hundred_point_lights_report_local_density_stats() {
    let viewport_size = UVec2::new(160, 120);
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let material = register_many_point_light_product_material_asset(
        asset_manager.as_ref(),
        "res://materials/hundred_point_lights_density.zmaterial",
        many_point_light_product_material(),
    );
    let dense_extract = hundred_point_light_density_extract(
        viewport_size,
        material,
        HundredPointLightPlacement::Dense,
    );
    let spread_extract = hundred_point_light_density_extract(
        viewport_size,
        material,
        HundredPointLightPlacement::Spread,
    );
    let dense_grid = build_light_grid_for_frame(&dense_extract, viewport_size, true);
    let spread_grid = build_light_grid_for_frame(&spread_extract, viewport_size, true);

    assert_eq!(dense_grid.params.light_count, 128);
    assert_eq!(spread_grid.params.light_count, 128);
    assert!(
        dense_grid.stats.peak_lights_per_cluster >= 96,
        "dense hundred-light grid should create a high local-density cluster; stats={:?}",
        dense_grid.stats
    );
    assert!(
        spread_grid.stats.non_empty_cluster_count > 0,
        "spread hundred-light grid should still produce visible light-grid occupancy; stats={:?}",
        spread_grid.stats
    );
    assert!(
        dense_grid.stats.average_lights_per_cluster
            >= spread_grid.stats.average_lights_per_cluster * 8.0,
        "equal-count hundred-light grids should report density-driven average cluster load; dense={:?}, spread={:?}",
        dense_grid.stats,
        spread_grid.stats
    );
    assert!(
        dense_grid.stats.peak_lights_per_cluster
            >= spread_grid.stats.peak_lights_per_cluster.saturating_mul(2),
        "dense/spread grids should differ by local peak despite equal total light count; dense={:?}, spread={:?}",
        dense_grid.stats,
        spread_grid.stats
    );

    let server = WgpuRenderFramework::new(asset_manager).unwrap();
    let (_, dense_stats) = render_many_point_light_product_frame(
        &server,
        viewport_size,
        RenderPipelineHandle::new(1),
        "hundred-point-dense",
        dense_extract,
    );
    let (_, spread_stats) = render_many_point_light_product_frame(
        &server,
        viewport_size,
        RenderPipelineHandle::new(1),
        "hundred-point-spread",
        spread_extract,
    );

    assert_hundred_point_light_density_stats("dense", &dense_stats);
    assert_hundred_point_light_density_stats("spread", &spread_stats);
    assert!(
        dense_stats.last_light_grid_average_lights_per_cluster_milli
            >= spread_stats
                .last_light_grid_average_lights_per_cluster_milli
                .saturating_mul(8),
        "equal-count hundred-light product submits should report density-driven average cluster load; dense={:?}, spread={:?}",
        dense_stats,
        spread_stats
    );
    assert!(
        dense_stats.last_light_grid_peak_lights_per_cluster
            >= spread_stats
                .last_light_grid_peak_lights_per_cluster
                .saturating_mul(2),
        "equal-count hundred-light product submits should report local-density-driven peak cost; dense={:?}, spread={:?}",
        dense_stats,
        spread_stats
    );
}

fn many_point_light_extract() -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(20_000),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(
                        Vec3::new(0.0, 1.0, 10.0),
                        Vec3::ZERO,
                        Vec3::Y,
                    ),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: Vec::new(),
                directional_lights: Vec::new(),
                point_lights: (0..64).map(product_point_light).collect(),
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
}

fn product_point_light(index: u64) -> RenderPointLightSnapshot {
    let column = (index % 8) as f32;
    let row = (index / 8) as f32;
    RenderPointLightSnapshot {
        node_id: 2_000 + index,
        light_id: 20_000 + index,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::new(column * 0.5 - 1.75, row * 0.25 - 0.875, 0.0),
        color: Vec3::new(1.0, 0.9, 0.7),
        intensity: 2.0,
        range: 6.0,
        shadow: None,
    }
}

fn visible_product_point_light(index: u64) -> RenderPointLightSnapshot {
    let column = (index % 8) as f32;
    let row = (index / 8) as f32;
    RenderPointLightSnapshot {
        node_id: 30_000 + index,
        light_id: 30_000 + index,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position: Vec3::new(column * 0.22 - 0.77, row * 0.16 - 0.56, 2.15),
        color: Vec3::new(1.0, 0.86, 0.62),
        intensity: 0.08,
        range: 6.0,
        shadow: None,
    }
}

fn many_point_light_product_material() -> MaterialAsset {
    MaterialAsset {
        name: Some("ManyPointLightsProduct".to_string()),
        shader: AssetReference::from_locator(AssetUri::parse("builtin://shader/pbr.wgsl").unwrap()),
        base_color: [0.32, 0.34, 0.36, 1.0],
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
        property_values: Default::default(),
        texture_slots: Default::default(),
        validation_diagnostics: Vec::new(),
    }
}

fn register_many_point_light_product_material_asset(
    asset_manager: &ProjectAssetManager,
    locator: &str,
    material: MaterialAsset,
) -> ResourceId {
    let material_uri = AssetUri::parse(locator).unwrap();
    let material_id = ResourceId::from_locator(&material_uri);
    asset_manager
        .assets::<MaterialAsset>()
        .insert(
            ResourceRecord::new(material_id, ResourceKind::Material, material_uri),
            material,
        )
        .expect("material insert");
    material_id
}

fn many_point_light_product_extract(
    viewport_size: UVec2,
    material: ResourceId,
    include_lights: bool,
) -> RenderFrameExtract {
    let point_lights = if include_lights {
        (0..64).map(visible_product_point_light).collect()
    } else {
        Vec::new()
    };
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(if include_lights { 30_001 } else { 30_000 }),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(Vec3::new(0.0, 0.0, 4.0), Vec3::ZERO, Vec3::Y),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![many_point_light_product_mesh(material)],
                directional_lights: Vec::new(),
                point_lights,
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

#[derive(Clone, Copy)]
enum HundredPointLightPlacement {
    Dense,
    Spread,
}

fn hundred_point_light_density_extract(
    viewport_size: UVec2,
    material: ResourceId,
    placement: HundredPointLightPlacement,
) -> RenderFrameExtract {
    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(match placement {
            HundredPointLightPlacement::Dense => 40_000,
            HundredPointLightPlacement::Spread => 40_001,
        }),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::looking_at(Vec3::new(0.0, 0.0, 4.0), Vec3::ZERO, Vec3::Y),
                    ..ViewportCameraSnapshot::default()
                },
                meshes: vec![many_point_light_product_mesh(material)],
                directional_lights: Vec::new(),
                point_lights: (0..128)
                    .map(|index| hundred_point_light_density_light(index, placement))
                    .collect(),
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

fn hundred_point_light_density_light(
    index: u64,
    placement: HundredPointLightPlacement,
) -> RenderPointLightSnapshot {
    let column = (index % 16) as f32;
    let row = (index / 16) as f32;
    let (position, range) = match placement {
        HundredPointLightPlacement::Dense => (
            Vec3::new(column * 0.035 - 0.2625, row * 0.035 - 0.1225, 2.05),
            2.5,
        ),
        HundredPointLightPlacement::Spread => (
            Vec3::new(column * 0.34 - 2.55, row * 0.24 - 0.84, 1.85),
            0.45,
        ),
    };
    RenderPointLightSnapshot {
        node_id: 40_000 + index,
        light_id: 40_000 + index,
        layer_mask: RenderLayerSet::from_scene_schema_v1_mask(DEFAULT_RENDER_LAYER_MASK),
        position,
        color: Vec3::new(1.0, 0.84, 0.62),
        intensity: 0.04,
        range,
        shadow: None,
    }
}

fn many_point_light_product_mesh(material: ResourceId) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 30_100,
        stable_instance_key: 30_100 << 16,
        transform_revision: 0,
        transform: Transform {
            scale: Vec3::new(1.45, 1.45, 0.35),
            ..Transform::default()
        },
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

fn render_many_point_light_product_frame(
    server: &WgpuRenderFramework,
    viewport_size: UVec2,
    pipeline: RenderPipelineHandle,
    profile_name: &str,
    extract: RenderFrameExtract,
) -> (CapturedFrame, RenderStats) {
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(
            viewport,
            many_point_light_product_profile(profile_name, pipeline),
        )
        .unwrap();
    server.submit_frame_extract(viewport, extract).unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("many-point-light product frame should be capturable");
    let stats = server.query_stats().unwrap();
    server.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn many_point_light_product_profile(
    name: &str,
    pipeline: RenderPipelineHandle,
) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_pipeline_asset(pipeline)
        .with_clustered_lighting(true)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn assert_many_point_light_product_stats(label: &str, stats: &RenderStats) {
    assert!(
        stats
            .last_effective_features
            .contains(&"clustered_lighting".to_string()),
        "{label}: clustered lighting should stay enabled; features={:?}",
        stats.last_effective_features
    );
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"lighting.light-grid".to_string()),
        "{label}: light-grid executor should run; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert!(
        stats.last_light_grid_reported,
        "{label}: missing light-grid stats"
    );
    assert_eq!(stats.last_light_grid_light_count, 64);
    assert!(stats.last_light_grid_non_empty_tile_count > 0);
    assert!(stats.last_light_grid_non_empty_zbin_count > 0);
    assert!(stats.last_light_grid_non_empty_cluster_count > 0);
    assert!(
        stats.last_light_grid_peak_lights_per_cluster >= 32,
        "{label}: product light-grid should retain dense clusters beyond the old tiny fixed light set; peak={}",
        stats.last_light_grid_peak_lights_per_cluster
    );
}

fn assert_hundred_point_light_density_stats(label: &str, stats: &RenderStats) {
    assert!(
        stats
            .last_graph_executed_executor_ids
            .contains(&"lighting.light-grid".to_string()),
        "{label}: light-grid executor should run; executors={:?}",
        stats.last_graph_executed_executor_ids
    );
    assert!(
        stats.last_light_grid_reported,
        "{label}: missing light-grid stats"
    );
    assert_eq!(stats.last_light_grid_light_count, 128);
    assert!(stats.last_light_grid_non_empty_tile_count > 0);
    assert!(stats.last_light_grid_non_empty_zbin_count > 0);
    assert!(stats.last_light_grid_non_empty_cluster_count > 0);
    assert!(stats.last_light_grid_peak_lights_per_cluster > 0);
    assert!(stats.last_light_grid_average_lights_per_cluster_milli > 0);
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

fn assert_light_grid_reads(compiled: &crate::graphics::CompiledRenderPipeline, pass_name: &str) {
    for resource_name in [
        PostProcessGraphResourceNames::LIGHT_GRID_PARAMS,
        PostProcessGraphResourceNames::LIGHT_ZBINS,
        PostProcessGraphResourceNames::LIGHT_TILE_MASKS,
    ] {
        pass_resource_access(
            compiled,
            pass_name,
            resource_name,
            RenderGraphResourceAccessKind::Read,
        );
    }
}
