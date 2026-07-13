use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::{
    AdvancedProviderReport, AdvancedProviderStatus, AdvancedRenderDegradationReason,
    AdvancedRenderFeature, CapturedFrame, FallbackSkyboxKind, PreviewEnvironmentExtract,
    RenderCapabilitySummary, RenderFrameExtract, RenderFramework, RenderHybridGiExtract,
    RenderHybridGiPayloadSource, RenderLayerSet, RenderMeshSnapshot, RenderQualityProfile,
    RenderSceneGeometryExtract, RenderSceneSnapshot, RenderStats, RenderViewportDescriptor,
    RenderVirtualGeometryCluster, RenderVirtualGeometryExtract, RenderVirtualGeometryPage,
    RenderVirtualGeometryPayloadSource, RenderWorldSnapshotHandle, ViewportCameraSnapshot,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Real, Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use crate::graphics::resource_limits::HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE;
use crate::graphics::runtime::WgpuRenderFramework;
use crate::scene::world::World;

use super::plugin_render_feature_fixtures::{
    pluginized_wgpu_render_framework, pluginized_wgpu_render_framework_with_advanced_providers,
};

const HZB_WALL_HIDDEN_INSTANCE_COUNT: usize = 64;

#[test]
fn render_product_advanced_submits_vg_hgi_only_with_runtime_providers() {
    let server = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(viewport, advanced_quality_profile("advanced-providers"))
        .unwrap();

    server
        .submit_frame_extract(viewport, advanced_product_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert!(stats
        .last_effective_features
        .contains(&"virtual_geometry".to_string()));
    assert!(stats
        .last_effective_features
        .contains(&"hybrid_gi".to_string()));
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 5);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 4);
    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::Authored
    );
    assert_eq!(
        stats.last_hybrid_gi_payload_source,
        RenderHybridGiPayloadSource::SceneRepresentation
    );
    assert_eq!(
        advanced_provider_report(&stats, AdvancedRenderFeature::VirtualGeometry).status,
        AdvancedProviderStatus::Ready
    );
    assert_eq!(
        advanced_provider_report(&stats, AdvancedRenderFeature::HybridGlobalIllumination).status,
        AdvancedProviderStatus::Ready
    );
}

#[test]
fn render_product_advanced_degrades_without_runtime_providers() {
    let server = pluginized_wgpu_render_framework();
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    server
        .set_quality_profile(viewport, advanced_quality_profile("advanced-no-provider"))
        .unwrap();

    server
        .submit_frame_extract(viewport, advanced_product_extract())
        .unwrap();
    let stats = server.query_stats().unwrap();

    assert!(!stats
        .last_effective_features
        .contains(&"virtual_geometry".to_string()));
    assert!(!stats
        .last_effective_features
        .contains(&"hybrid_gi".to_string()));
    assert_eq!(stats.last_virtual_geometry_graph_executed_pass_count, 0);
    assert_eq!(stats.last_hybrid_gi_graph_executed_pass_count, 0);
    assert_eq!(
        stats.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::None
    );
    assert_eq!(
        stats.last_hybrid_gi_payload_source,
        RenderHybridGiPayloadSource::None
    );

    for feature in [
        AdvancedRenderFeature::VirtualGeometry,
        AdvancedRenderFeature::HybridGlobalIllumination,
    ] {
        let report = advanced_provider_report(&stats, feature);
        assert!(report.requested);
        assert_eq!(report.provider_id, None);
        assert_eq!(report.status, AdvancedProviderStatus::Degraded);
        assert!(report
            .degradations
            .iter()
            .any(|degradation| degradation.reason
                == AdvancedRenderDegradationReason::ProviderMissing));
    }
}

#[test]
fn render_product_hzb_occlusion_wall_scene() {
    let viewport_size = UVec2::new(320, 240);
    let (occlusion_frame, stats) =
        render_hzb_occlusion_wall_scene(hzb_occlusion_wall_capabilities(), viewport_size);
    let (fallback_frame, fallback_stats) = render_hzb_occlusion_wall_scene(
        hzb_occlusion_wall_cpu_fallback_capabilities(),
        viewport_size,
    );

    assert!(stats.last_hzb_occlusion_reported);
    assert!(stats.last_hzb_occlusion_history_available);
    assert!(stats.last_hzb_occlusion_readback_available);
    assert!(stats.last_hzb_occlusion_indirect_args_readback_available);
    assert!(
        stats.last_hzb_occlusion_tested_instance_count >= HZB_WALL_HIDDEN_INSTANCE_COUNT,
        "HZB occlusion should test the hidden wall-scene instances; tested={}, hidden={}",
        stats.last_hzb_occlusion_tested_instance_count,
        HZB_WALL_HIDDEN_INSTANCE_COUNT
    );
    assert!(
        stats.last_hzb_occlusion_culled_instance_count >= HZB_WALL_HIDDEN_INSTANCE_COUNT,
        "the wall should occlude the hidden instances on the second frame; culled={}, hidden={}",
        stats.last_hzb_occlusion_culled_instance_count,
        HZB_WALL_HIDDEN_INSTANCE_COUNT
    );
    assert_eq!(
        stats.last_visibility_occlusion_culled_count,
        stats.last_hzb_occlusion_culled_instance_count
    );
    assert!(stats.last_hzb_occlusion_compacted_draw_count > 0);
    assert!(
        stats.last_hzb_occlusion_compacted_draw_count <= stats.last_hzb_occlusion_readback_arg_count,
        "compact replay should submit no more draws than the readback arg capacity; compacted={}, readback={}",
        stats.last_hzb_occlusion_compacted_draw_count,
        stats.last_hzb_occlusion_readback_arg_count
    );
    assert!(stats.last_hzb_occlusion_zero_instance_arg_count > 0);
    assert!(
        stats.last_hzb_occlusion_remaining_instance_count
            < stats.last_hzb_occlusion_tested_instance_count,
        "occlusion culling should reduce submitted instances; remaining={}, tested={}",
        stats.last_hzb_occlusion_remaining_instance_count,
        stats.last_hzb_occlusion_tested_instance_count
    );
    assert!(
        !fallback_stats.last_hzb_occlusion_reported,
        "capability fallback should not execute hzb-occlusion-cull"
    );
    assert_eq!(fallback_stats.last_visibility_occlusion_culled_count, 0);
    assert_captured_frames_equal(
        &occlusion_frame,
        &fallback_frame,
        "HZB occlusion should match the occlusion-disabled product baseline",
    );
}

#[test]
fn render_product_hzb_occlusion_respects_storage_buffer_limit_fallback() {
    let viewport_size = UVec2::new(320, 240);
    let (limit_fallback_frame, limit_fallback_stats) = render_hzb_occlusion_wall_scene(
        hzb_occlusion_wall_low_storage_buffer_capabilities(),
        viewport_size,
    );
    let (cpu_fallback_frame, cpu_fallback_stats) = render_hzb_occlusion_wall_scene(
        hzb_occlusion_wall_cpu_fallback_capabilities(),
        viewport_size,
    );

    assert!(
        limit_fallback_stats.capabilities.supports_storage_buffers,
        "the test isolates the per-stage storage-buffer count gate, not storage-buffer support"
    );
    assert!(
        limit_fallback_stats
            .capabilities
            .gpu_driven_submission_supported(),
        "the test keeps GPU-driven submission otherwise available"
    );
    assert_eq!(
        limit_fallback_stats
            .capabilities
            .max_storage_buffers_per_shader_stage,
        HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE - 1
    );
    assert!(
        !limit_fallback_stats.last_hzb_occlusion_reported,
        "storage-buffer capacity fallback should not execute hzb-occlusion-cull"
    );
    assert_eq!(
        limit_fallback_stats.last_visibility_occlusion_culled_count, 0,
        "CPU visibility fallback should not report HZB occlusion culls"
    );
    assert_eq!(limit_fallback_stats.last_hzb_occlusion_tested_arg_count, 0);
    assert_eq!(
        limit_fallback_stats.last_hzb_occlusion_compacted_draw_count,
        0
    );
    assert!(!cpu_fallback_stats.last_hzb_occlusion_reported);
    assert_captured_frames_equal(
        &limit_fallback_frame,
        &cpu_fallback_frame,
        "storage-buffer-limit fallback should match the CPU visibility baseline",
    );
}

pub(super) fn advanced_quality_profile(name: &str) -> RenderQualityProfile {
    RenderQualityProfile::new(name)
        .with_virtual_geometry(true)
        .with_hybrid_global_illumination(true)
}

pub(super) fn advanced_product_extract() -> crate::core::framework::render::RenderFrameExtract {
    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(UVec2::new(320, 240));
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            virtual_geometry_cluster(mesh, 15, 150, 1, Vec3::new(100.0, 0.0, 0.0), 9.0),
            virtual_geometry_cluster(mesh, 30, 300, 0, Vec3::ZERO, 8.0),
            virtual_geometry_cluster(mesh, 20, 200, 1, Vec3::new(0.1, 0.0, 0.0), 5.0),
            virtual_geometry_cluster(mesh, 10, 100, 2, Vec3::new(0.2, 0.0, 0.0), 2.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_geometry_page(100, false),
            virtual_geometry_page(150, false),
            virtual_geometry_page(200, true),
            virtual_geometry_page(300, false),
            virtual_geometry_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    extract.lighting.hybrid_global_illumination = Some(RenderHybridGiExtract {
        enabled: true,
        mode: Default::default(),
        profile: Default::default(),
        quality: Default::default(),
        trace_budget: 2,
        card_budget: 1,
        voxel_budget: 2,
        debug_view: Default::default(),
    });
    extract
}

fn virtual_geometry_cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level,
        parent_cluster_id: None,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}

fn virtual_geometry_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}

pub(super) fn advanced_provider_report(
    stats: &RenderStats,
    feature: AdvancedRenderFeature,
) -> &AdvancedProviderReport {
    stats
        .last_advanced_provider_reports
        .iter()
        .find(|report| report.feature == feature)
        .expect("advanced provider report should be recorded")
}

fn hzb_occlusion_wall_quality_profile() -> RenderQualityProfile {
    RenderQualityProfile::new("hzb-occlusion-wall")
        .with_clustered_lighting(false)
        .with_screen_space_ambient_occlusion(false)
        .with_temporal_history(false)
        .with_bloom(false)
        .with_color_grading(false)
        .with_anti_alias(false)
}

fn hzb_occlusion_wall_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: "hzb-occlusion-wall-test".to_string(),
        supports_offscreen: true,
        supports_async_compute: true,
        supports_storage_buffers: true,
        max_storage_buffers_per_shader_stage:
            HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE,
        supports_indirect_draw: true,
        supports_multi_draw_indirect: true,
        supports_indirect_first_instance: true,
        supports_buffer_readback: true,
        supports_fxaa: true,
        ..RenderCapabilitySummary::default()
    }
}

fn hzb_occlusion_wall_cpu_fallback_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: "hzb-occlusion-wall-cpu-baseline".to_string(),
        supports_multi_draw_indirect: false,
        ..hzb_occlusion_wall_capabilities()
    }
}

fn hzb_occlusion_wall_low_storage_buffer_capabilities() -> RenderCapabilitySummary {
    RenderCapabilitySummary {
        backend_name: "hzb-occlusion-wall-storage-buffer-limit".to_string(),
        max_storage_buffers_per_shader_stage:
            HZB_OCCLUSION_REQUIRED_STORAGE_BUFFERS_PER_SHADER_STAGE - 1,
        ..hzb_occlusion_wall_capabilities()
    }
}

fn render_hzb_occlusion_wall_scene(
    capabilities: RenderCapabilitySummary,
    viewport_size: UVec2,
) -> (CapturedFrame, RenderStats) {
    let server = WgpuRenderFramework::new(Arc::new(ProjectAssetManager::default())).unwrap();
    server.override_capabilities_for_tests(capabilities);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .unwrap();
    server
        .set_quality_profile(viewport, hzb_occlusion_wall_quality_profile())
        .unwrap();

    server
        .submit_frame_extract(viewport, hzb_occlusion_wall_extract(viewport_size))
        .unwrap();
    server
        .submit_frame_extract(viewport, hzb_occlusion_wall_extract(viewport_size))
        .unwrap();
    let frame = server
        .capture_frame(viewport)
        .unwrap()
        .expect("wall-scene second frame should be capturable");
    let stats = server.query_stats().unwrap();
    server.destroy_viewport(viewport).unwrap();
    (frame, stats)
}

fn assert_captured_frames_equal(actual: &CapturedFrame, expected: &CapturedFrame, label: &str) {
    assert_eq!(actual.width, expected.width, "{label}: width mismatch");
    assert_eq!(actual.height, expected.height, "{label}: height mismatch");
    if let Some(index) = first_pixel_mismatch(&actual.rgba, &expected.rgba) {
        let byte = index * 4;
        panic!(
            "{label}: pixel {index} mismatch, actual={:?}, expected={:?}",
            &actual.rgba[byte..byte + 4],
            &expected.rgba[byte..byte + 4]
        );
    }
}

fn first_pixel_mismatch(actual: &[u8], expected: &[u8]) -> Option<usize> {
    assert_eq!(
        actual.len(),
        expected.len(),
        "captured frame byte lengths should match"
    );
    actual
        .chunks_exact(4)
        .zip(expected.chunks_exact(4))
        .position(|(actual, expected)| actual != expected)
}

fn hzb_occlusion_wall_extract(viewport_size: UVec2) -> RenderFrameExtract {
    let mut meshes = Vec::with_capacity(HZB_WALL_HIDDEN_INSTANCE_COUNT + 1);
    meshes.push(hzb_wall_mesh(
        91_000,
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(4.5, 3.5, 0.25),
    ));
    for index in 0..HZB_WALL_HIDDEN_INSTANCE_COUNT {
        let row = index / 8;
        let column = index % 8;
        let x = (column as Real - 3.5) * 0.32;
        let y = (row as Real - 3.5) * 0.32;
        meshes.push(hzb_hidden_mesh(
            92_000 + index as u64,
            Vec3::new(x, y, -4.0),
        ));
    }

    RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(910),
        RenderSceneSnapshot {
            scene: RenderSceneGeometryExtract {
                camera: ViewportCameraSnapshot {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 4.0)),
                    ..ViewportCameraSnapshot::default()
                },
                meshes,
                directional_lights: Vec::new(),
                point_lights: Vec::new(),
                spot_lights: Vec::new(),
                ambient_lights: Vec::new(),
                rect_lights: Vec::new(),
            },
            overlays: Default::default(),
            environment: crate::core::framework::render::EnvironmentExtract::default(),
            preview: PreviewEnvironmentExtract {
                lighting_enabled: false,
                skybox_enabled: false,
                fallback_skybox: FallbackSkyboxKind::None,
                clear_color: Vec4::ZERO,
            },
            virtual_geometry_debug: None,
        },
    )
    .with_viewport_size(viewport_size)
}

fn hzb_wall_mesh(node_id: u64, translation: Vec3, scale: Vec3) -> RenderMeshSnapshot {
    hzb_occlusion_mesh(
        node_id,
        translation,
        scale,
        "builtin://material/hzb-wall",
        Vec4::new(0.75, 0.76, 0.8, 1.0),
    )
}

fn hzb_hidden_mesh(node_id: u64, translation: Vec3) -> RenderMeshSnapshot {
    hzb_occlusion_mesh(
        node_id,
        translation,
        Vec3::new(0.18, 0.18, 0.18),
        "builtin://material/hzb-hidden",
        Vec4::new(0.0, 0.8, 0.55, 1.0),
    )
}

fn hzb_occlusion_mesh(
    node_id: u64,
    translation: Vec3,
    scale: Vec3,
    material_label: &str,
    tint: Vec4,
) -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id,
        stable_instance_key: node_id << 16,
        transform_revision: 0,
        transform: Transform {
            translation,
            scale,
            ..Transform::default()
        },
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            material_label,
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint,
        mobility: Mobility::Static,
        static_state: Default::default(),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}
