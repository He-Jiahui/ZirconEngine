use crate::core::framework::render::{
    AdvancedProviderReport, AdvancedProviderStatus, AdvancedRenderDegradationReason,
    AdvancedRenderFeature, RenderFramework, RenderHybridGiExtract, RenderHybridGiPayloadSource,
    RenderHybridGiProbe, RenderHybridGiTraceRegion, RenderQualityProfile, RenderStats,
    RenderViewportDescriptor, RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometryPage, RenderVirtualGeometryPayloadSource,
};
use crate::core::math::{UVec2, Vec3};
use crate::scene::world::World;

use super::plugin_render_feature_fixtures::{
    pluginized_wgpu_render_framework, pluginized_wgpu_render_framework_with_advanced_providers,
};

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
        RenderHybridGiPayloadSource::Authored
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
        quality: Default::default(),
        trace_budget: 0,
        card_budget: 1,
        voxel_budget: 2,
        debug_view: Default::default(),
        probe_budget: 1,
        tracing_budget: 1,
        probes: vec![
            hybrid_gi_probe(mesh, 30, false, Vec3::ZERO, 128),
            hybrid_gi_probe(mesh, 20, true, Vec3::new(0.1, 0.0, 0.0), 64),
            hybrid_gi_probe(mesh, 10, false, Vec3::new(100.0, 0.0, 0.0), 32),
        ],
        trace_regions: vec![
            hybrid_gi_trace_region(mesh, 40, Vec3::ZERO, 8.0),
            hybrid_gi_trace_region(mesh, 50, Vec3::new(0.1, 0.0, 0.0), 5.0),
            hybrid_gi_trace_region(mesh, 60, Vec3::new(100.0, 0.0, 0.0), 10.0),
        ],
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

fn hybrid_gi_probe(
    entity: u64,
    probe_id: u32,
    resident: bool,
    position: Vec3,
    ray_budget: u32,
) -> RenderHybridGiProbe {
    RenderHybridGiProbe {
        entity,
        probe_id,
        position,
        radius: 0.5,
        parent_probe_id: None,
        resident,
        ray_budget,
    }
}

fn hybrid_gi_trace_region(
    entity: u64,
    region_id: u32,
    bounds_center: Vec3,
    screen_coverage: f32,
) -> RenderHybridGiTraceRegion {
    RenderHybridGiTraceRegion {
        entity,
        region_id,
        bounds_center,
        bounds_radius: 0.5,
        screen_coverage,
        rt_lighting_rgb: [0, 0, 0],
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
