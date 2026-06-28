use crate::core::framework::render::{
    GeometryExtract, ProjectionMode, RenderFrameExtract, RenderFramework, RenderLayerSet,
    RenderMeshSnapshot, RenderMeshStaticState, RenderQualityProfile, RenderStats,
    RenderViewportDescriptor, RenderVirtualGeometryCluster, RenderVirtualGeometryExtract,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryPage,
    RenderVirtualGeometryPayloadSource, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source, RenderWorldSnapshotHandle,
};
use crate::core::framework::scene::Mobility;
use crate::core::math::{Transform, UVec2, Vec3, Vec4};
use crate::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};

use super::super::plugin_render_feature_fixtures::pluginized_wgpu_render_framework_with_advanced_providers;
use super::super::render_product_submit::snapshot_with_projection_for_mesh_cache_tests;

#[test]
fn render_product_virtual_geometry_extract_stays_out_of_pre_mesh_cache() {
    let framework = pluginized_wgpu_render_framework_with_advanced_providers();
    let viewport = framework
        .create_viewport(RenderViewportDescriptor::new(UVec2::new(320, 240)))
        .unwrap();
    framework
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("static-cache-virtual-geometry-residual")
                .with_virtual_geometry(true)
                .with_screen_space_ambient_occlusion(false),
        )
        .unwrap();

    framework
        .submit_frame_extract(viewport, static_cache_virtual_geometry_extract(597))
        .unwrap();
    let first = framework.query_stats().unwrap();
    assert_eq!(
        first.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::Authored
    );
    assert!(
        first.last_virtual_geometry_indirect_draw_count >= 1,
        "authored virtual geometry should still produce GPU-driven execution draws",
    );
    assert!(
        first.last_virtual_geometry_indirect_buffer_count >= 1,
        "authored virtual geometry should create mesh-level WGPU indirect buffers",
    );
    assert!(
        first.last_virtual_geometry_indirect_args_count >= 1,
        "authored virtual geometry should populate indexed indirect args",
    );
    assert!(
        first.last_virtual_geometry_indirect_segment_count >= 1,
        "authored virtual geometry should record executable indirect segments",
    );
    assert_virtual_geometry_execution_stats_visible(&first);
    assert_eq!(
        first.last_mesh_pending_static_command_cache_draw_candidate_count, 0,
        "virtual-geometry visibility carrier meshes must not be advertised as static mesh command-cache candidates",
    );
    assert_eq!(
        first.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(first.last_mesh_cached_command_hit_count, 0);
    assert!(
        first.last_mesh_dynamic_command_count
            >= first.last_virtual_geometry_execution_segment_count,
        "virtual-geometry execution commands should remain on the dynamic indirect replay path",
    );

    framework
        .submit_frame_extract(viewport, static_cache_virtual_geometry_extract(598))
        .unwrap();
    let second = framework.query_stats().unwrap();
    assert_eq!(
        second.last_virtual_geometry_payload_source,
        RenderVirtualGeometryPayloadSource::Authored
    );
    assert!(
        second.last_virtual_geometry_indirect_draw_count >= 1,
        "virtual geometry remains GPU-driven across frames instead of being absorbed by MeshDraw cache",
    );
    assert!(
        second.last_virtual_geometry_indirect_buffer_count >= 1,
        "virtual geometry keeps mesh-level WGPU indirect buffers across frames",
    );
    assert!(
        second.last_virtual_geometry_indirect_args_count >= 1,
        "virtual geometry keeps indexed indirect args across frames",
    );
    assert!(
        second.last_virtual_geometry_indirect_segment_count >= 1,
        "virtual geometry keeps executable indirect segments across frames",
    );
    assert_virtual_geometry_execution_stats_visible(&second);
    assert_eq!(
        second.last_mesh_pending_static_command_cache_draw_candidate_count,
        0,
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_draw_count,
        0,
    );
    assert_eq!(
        second.last_mesh_pre_mesh_draw_static_command_cache_skipped_phase_count,
        0,
    );
    assert_eq!(second.last_mesh_cached_command_hit_count, 0);
    assert_eq!(second.last_mesh_command_cache_miss_count, 0);
    assert!(
        second.last_mesh_dynamic_command_count
            >= second.last_virtual_geometry_execution_segment_count,
        "virtual-geometry execution commands should remain on the dynamic indirect replay path",
    );
    assert!(
        second.last_mesh_command_rebuild_count >= second.last_mesh_dynamic_command_count,
        "dynamic virtual-geometry indirect commands should be reflected in command rebuild stats",
    );
}

fn static_cache_virtual_geometry_extract(world: u64) -> RenderFrameExtract {
    let mut extract = RenderFrameExtract::from_snapshot(
        RenderWorldSnapshotHandle::new(world),
        snapshot_with_projection_for_mesh_cache_tests(ProjectionMode::Perspective),
    );
    extract.geometry = GeometryExtract::from_meshes(
        extract.view.core_pipeline,
        vec![static_cache_virtual_geometry_visibility_mesh()],
    );
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            static_cache_virtual_geometry_cluster(803, 15, 150, 1, Vec3::new(100.0, 0.0, 0.0), 9.0),
            static_cache_virtual_geometry_cluster(803, 30, 300, 0, Vec3::ZERO, 8.0),
            static_cache_virtual_geometry_cluster(803, 20, 200, 1, Vec3::new(0.1, 0.0, 0.0), 5.0),
            static_cache_virtual_geometry_cluster(803, 10, 100, 2, Vec3::new(0.2, 0.0, 0.0), 2.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            static_cache_virtual_geometry_page(100, false),
            static_cache_virtual_geometry_page(150, false),
            static_cache_virtual_geometry_page(200, true),
            static_cache_virtual_geometry_page(300, false),
            static_cache_virtual_geometry_page(500, true),
        ],
        page_dependencies: Vec::new(),
        instances: Vec::new(),
        debug: Default::default(),
    });
    extract
}

fn static_cache_virtual_geometry_visibility_mesh() -> RenderMeshSnapshot {
    RenderMeshSnapshot {
        node_id: 803,
        stable_instance_key: 803 << 16,
        transform_revision: 1,
        transform: Transform::default(),
        model: ResourceHandle::<ModelMarker>::new(ResourceId::from_stable_label("builtin://cube")),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(ResourceId::from_stable_label(
            "builtin://material/default",
        )),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::default(),
        render_layer_mask: RenderLayerSet::from_scene_schema_v1_mask(u32::MAX),
    }
}

fn assert_virtual_geometry_execution_stats_visible(stats: &RenderStats) {
    assert!(
        stats.last_virtual_geometry_execution_segment_count >= 1,
        "product stats should expose executable virtual-geometry segments",
    );
    assert!(
        stats.last_virtual_geometry_execution_page_count >= 1,
        "product stats should retain the resident/requested page set used by VG execution",
    );
    assert_eq!(
        stats.last_virtual_geometry_execution_missing_segment_count, 0,
        "the product fixture keeps executable segments on resident or requested pages",
    );
    assert!(
        stats.last_virtual_geometry_execution_resident_segment_count
            + stats.last_virtual_geometry_execution_pending_segment_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "resident and pending execution buckets should cover executable VG segments",
    );
    assert_eq!(
        stats.last_virtual_geometry_selected_cluster_source,
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
    );
    assert_eq!(
        stats.last_virtual_geometry_visbuffer64_source,
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
    );
    assert_eq!(
        stats.last_virtual_geometry_hardware_rasterization_source,
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
    );
    assert!(
        stats.last_virtual_geometry_selected_cluster_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "selected cluster stats should cover every executable VG segment",
    );
    assert!(
        stats.last_virtual_geometry_visbuffer64_entry_count
            >= stats.last_virtual_geometry_execution_segment_count,
        "visbuffer64 stats should cover every executable VG segment",
    );
    assert_eq!(
        stats.last_virtual_geometry_hardware_rasterization_record_count,
        stats.last_virtual_geometry_execution_segment_count,
    );
}

fn static_cache_virtual_geometry_cluster(
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

fn static_cache_virtual_geometry_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}
