use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    RenderFramework, RenderQualityProfile, RenderViewportDescriptor, RenderVirtualGeometryCluster,
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHardwareRasterizationSource,
    RenderVirtualGeometryInstance, RenderVirtualGeometryNodeAndClusterCullSource,
    RenderVirtualGeometryPage, RenderVirtualGeometrySelectedClusterSource,
    RenderVirtualGeometryVisBuffer64Source,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::scene::world::World;

#[test]
fn render_framework_stats_expose_virtual_geometry_instance_ranges_and_debug_state() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).expect("framework should initialize");
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-stats")
                .with_virtual_geometry(true)
                .with_hybrid_global_illumination(false)
                .with_clustered_lighting(false)
                .with_screen_space_ambient_occlusion(false)
                .with_history_resolve(false)
                .with_bloom(false)
                .with_color_grading(false)
                .with_reflection_probes(false)
                .with_baked_lighting(false)
                .with_particle_rendering(false)
                .with_async_compute(false),
        )
        .expect("quality profile should be accepted");

    let world = World::new();
    let mesh = world
        .nodes()
        .iter()
        .find(|node| node.mesh.is_some())
        .map(|node| node.id)
        .expect("default world should contain a renderable mesh");
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            virtual_geometry_cluster(mesh, 30, 300, 1, Vec3::ZERO, 8.0),
            virtual_geometry_cluster(mesh, 20, 200, 1, Vec3::new(0.1, 0.0, 0.0), 5.0),
        ],
        pages: vec![
            virtual_geometry_page(200, true),
            virtual_geometry_page(300, false),
        ],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("StatsContractMesh".to_string()),
            source_hint: Some("integration-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(1),
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: true,
        },
    });

    server
        .submit_frame_extract(viewport, extract)
        .expect("virtual geometry submission should succeed");

    let stats = server.query_stats().expect("stats should be queryable");
    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot should be queryable")
        .expect("virtual geometry snapshot should be present");
    assert_eq!(snapshot.cull_input.cluster_budget, 2);
    assert_eq!(snapshot.cull_input.page_budget, 1);
    assert_eq!(snapshot.cull_input.instance_count, 1);
    assert_eq!(snapshot.cull_input.cluster_count, 2);
    assert_eq!(snapshot.cull_input.page_count, 2);
    assert_eq!(snapshot.cull_input.visible_entity_count, 1);
    assert_eq!(snapshot.cull_input.debug, snapshot.debug);
    assert_eq!(stats.last_virtual_geometry_instance_count, 1);
    assert_eq!(stats.last_virtual_geometry_cluster_budget, 2);
    assert_eq!(stats.last_virtual_geometry_page_budget, 1);
    assert_eq!(stats.last_virtual_geometry_input_cluster_count, 2);
    assert_eq!(stats.last_virtual_geometry_input_page_count, 2);
    assert_eq!(stats.last_virtual_geometry_visible_entity_count, 1);
    assert_eq!(stats.last_virtual_geometry_forced_mip, Some(1));
    assert!(stats.last_virtual_geometry_freeze_cull);
    assert!(stats.last_virtual_geometry_visualize_bvh);
    assert!(stats.last_virtual_geometry_visualize_visbuffer);
    assert!(stats.last_virtual_geometry_print_leaf_clusters);
    assert_eq!(
        stats.last_virtual_geometry_visbuffer64_source,
        snapshot.visbuffer64_source,
        "expected stats to expose the same VisBuffer64 render-path provenance as the renderer-owned VG snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_visbuffer64_entry_count,
        snapshot.visbuffer64_entries.len(),
        "expected stats to expose the same VisBuffer64 packed-entry count as the renderer-owned VG snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_hardware_rasterization_source,
        snapshot.hardware_rasterization_source,
        "expected stats to expose the same hardware-rasterization render-path provenance as the renderer-owned VG snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_hardware_rasterization_record_count,
        snapshot.hardware_rasterization_records.len(),
        "expected stats to expose the same hardware-rasterization startup-record count as the renderer-owned VG snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_selected_cluster_count,
        snapshot.selected_clusters.len(),
        "expected stats to expose the same executed selected-cluster count as the renderer-owned VG snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_selected_cluster_source,
        snapshot.selected_clusters_source,
        "expected stats to expose the same selected-cluster render-path provenance as the renderer-owned VG snapshot"
    );
    assert_eq!(
        snapshot.cluster_selection_input_source,
        RenderVirtualGeometryClusterSelectionInputSource::PrepareDerivedFrameOwned,
        "expected runtime-frame VG snapshots to expose that their authoritative cluster-selection input was mirrored from prepare-owned truth instead of an explicit frame override"
    );
    assert_eq!(
        snapshot.cull_input.cluster_selection_input_source,
        snapshot.cluster_selection_input_source,
        "expected the new cull-input snapshot to carry the same authoritative cluster-selection provenance as the parent VG debug snapshot so a later NaniteGlobalStateBuffer bridge can consume one stable input DTO"
    );
    assert_eq!(
        stats.last_virtual_geometry_cluster_selection_input_source,
        snapshot.cluster_selection_input_source,
        "expected stats to expose the same cluster-selection input provenance as the renderer-owned VG snapshot so runtime preview and editor UI can debug the same authority seam"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_source,
        RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        "expected stats to expose the same first-pass NodeAndClusterCull startup provenance as the renderer-owned VG pass seam"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_record_count,
        1,
        "expected stats to expose the first-pass NodeAndClusterCull startup record count when a VG cull-input snapshot is present"
    );
    assert_eq!(
        stats.last_virtual_geometry_node_and_cluster_cull_instance_seed_count,
        snapshot.node_and_cluster_cull_instance_seeds.len(),
        "expected stats to expose the same NodeAndClusterCull root-seed worklist count as the renderer-owned VG snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_cluster_budget,
        snapshot.cull_input.cluster_budget as usize,
        "expected stats to mirror the same authoritative VG cluster budget as the renderer-owned cull-input snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_page_budget,
        snapshot.cull_input.page_budget as usize,
        "expected stats to mirror the same authoritative VG page budget as the renderer-owned cull-input snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_input_cluster_count,
        snapshot.cull_input.cluster_count as usize,
        "expected stats to mirror the same authored VG cluster input count as the renderer-owned cull-input snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_input_page_count,
        snapshot.cull_input.page_count as usize,
        "expected stats to mirror the same authored VG page input count as the renderer-owned cull-input snapshot"
    );
    assert_eq!(
        stats.last_virtual_geometry_visible_entity_count,
        snapshot.cull_input.visible_entity_count as usize,
        "expected stats to mirror the same visible-entity submit gate as the renderer-owned cull-input snapshot"
    );

    let mut cleared_extract = world.to_render_frame_extract();
    cleared_extract.apply_viewport_size(viewport_size);
    server
        .submit_frame_extract(viewport, cleared_extract)
        .expect("non-vg submission should clear vg stats");

    let cleared_stats = server.query_stats().expect("stats should be queryable");
    assert_eq!(cleared_stats.last_virtual_geometry_cluster_budget, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_page_budget, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_input_cluster_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_input_page_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_visible_entity_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_instance_count, 0);
    assert_eq!(cleared_stats.last_virtual_geometry_forced_mip, None);
    assert!(!cleared_stats.last_virtual_geometry_freeze_cull);
    assert!(!cleared_stats.last_virtual_geometry_visualize_bvh);
    assert!(!cleared_stats.last_virtual_geometry_visualize_visbuffer);
    assert!(!cleared_stats.last_virtual_geometry_print_leaf_clusters);
    assert_eq!(
        cleared_stats.last_virtual_geometry_visbuffer64_source,
        RenderVirtualGeometryVisBuffer64Source::Unavailable
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_visbuffer64_entry_count,
        0
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_hardware_rasterization_source,
        RenderVirtualGeometryHardwareRasterizationSource::Unavailable
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_hardware_rasterization_record_count,
        0
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_selected_cluster_count,
        0
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_selected_cluster_source,
        RenderVirtualGeometrySelectedClusterSource::Unavailable
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_cluster_selection_input_source,
        RenderVirtualGeometryClusterSelectionInputSource::Unavailable
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_node_and_cluster_cull_source,
        RenderVirtualGeometryNodeAndClusterCullSource::Unavailable
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_node_and_cluster_cull_record_count,
        0
    );
    assert_eq!(
        cleared_stats.last_virtual_geometry_node_and_cluster_cull_instance_seed_count,
        0
    );
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
