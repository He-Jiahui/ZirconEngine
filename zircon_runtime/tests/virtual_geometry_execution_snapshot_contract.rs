use std::sync::Arc;

use zircon_runtime::asset::pipeline::manager::ProjectAssetManager;
use zircon_runtime::core::framework::render::{
    RenderFramework, RenderQualityProfile, RenderViewportDescriptor, RenderVirtualGeometryCluster,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExecutionState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryHardwareRasterizationRecord,
    RenderVirtualGeometryHardwareRasterizationSource, RenderVirtualGeometryInstance,
    RenderVirtualGeometryPage, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source, RenderVirtualGeometryVisBufferMark,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3};
use zircon_runtime::graphics::WgpuRenderFramework;
use zircon_runtime::scene::world::World;

#[test]
fn render_framework_visbuffer_marks_follow_execution_segments_not_missing_visibility_superset() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).expect("framework should initialize");
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-execution-snapshot")
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
        page_budget: 0,
        clusters: vec![
            virtual_geometry_cluster(mesh, 20, 200, 0, Vec3::ZERO, 9.0),
            virtual_geometry_cluster(mesh, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_geometry_page(200, false),
            virtual_geometry_page(300, true),
        ],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("ExecutionSnapshotContractMesh".to_string()),
            source_hint: Some("integration-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            visualize_visbuffer: true,
            ..RenderVirtualGeometryDebugState::default()
        },
    });

    server
        .submit_frame_extract(viewport, extract)
        .expect("virtual geometry submission should succeed");

    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot query should succeed")
        .expect("virtual geometry snapshot should be present");

    assert_eq!(
        snapshot.visible_cluster_ids,
        vec![20, 30],
        "expected visibility planning to keep both the missing and resident clusters in the visible frontier before execution-side filtering"
    );
    assert_eq!(
        snapshot
            .execution_segments
            .iter()
            .map(|segment| {
                (
                    segment.instance_index,
                    segment.cluster_start_ordinal,
                    segment.cluster_span_count,
                    segment.page_id,
                    segment.state,
                )
            })
            .collect::<Vec<_>>(),
        vec![(
            Some(0),
            1,
            1,
            300,
            RenderVirtualGeometryExecutionState::Resident,
        )],
        "expected the real execution subset to keep instance ownership from the authoritative cluster selection while dropping the missing page-200 cluster"
    );
    assert_eq!(
        snapshot.selected_clusters,
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected selected_clusters to be rebuilt from the real execution subset so the stored public cluster worklist matches the same authoritative post-render selection as execution_segments"
    );
    assert_eq!(
        snapshot.selected_clusters_source,
        RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        "expected the public execution snapshot to preserve explicit render-path provenance when executed cluster selections produced the authoritative selected-cluster worklist"
    );
    assert_eq!(
        snapshot.visbuffer_debug_marks,
        vec![RenderVirtualGeometryVisBufferMark {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
            color_rgba: visbuffer_color(30, 300, 0),
        }],
        "expected visbuffer debug marks to be rebuilt from the real execution subset so missing visibility-only clusters do not survive into the execution-facing snapshot"
    );
    assert_eq!(
        snapshot.hardware_rasterization_records,
        vec![RenderVirtualGeometryHardwareRasterizationRecord {
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            submission_index: 0,
            submission_page_id: 300,
            submission_lod_level: 0,
            entity_cluster_start_ordinal: 1,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: Some(0),
            submission_slot: Some(0),
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the public execution snapshot to expose a hardware-rasterization startup record stream sourced from the same execution-owned ClusterSelection subset that feeds the compat render path"
    );
    assert_eq!(
        snapshot.hardware_rasterization_source,
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        "expected the public execution snapshot to preserve explicit render-path provenance when executed cluster selections produced hardware-rasterization startup records"
    );
    assert_eq!(
        snapshot.visbuffer64_source,
        RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        "expected the public execution snapshot to preserve explicit render-path provenance when executed cluster selections produced VisBuffer64 entries"
    );
    assert_eq!(
        snapshot.visbuffer64_clear_value, 0,
        "expected the first VisBuffer64 abstraction layer to publish a stable clear value even before pixel-accurate hardware rasterization lands"
    );
    assert_eq!(
        snapshot.visbuffer64_entries,
        vec![RenderVirtualGeometryVisBuffer64Entry {
            entry_index: 0,
            packed_value: pack_visbuffer64_entry(Some(0), 30, 300, 0, RenderVirtualGeometryExecutionState::Resident),
            instance_index: Some(0),
            entity: mesh,
            cluster_id: 30,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the execution-facing snapshot to publish a 64-bit visibility result abstraction sourced from the same post-render cluster subset as selected_clusters and visbuffer debug marks"
    );
}

#[test]
fn render_framework_visbuffer64_source_reports_clear_only_for_empty_execution_selection_frames() {
    let asset_manager = Arc::new(ProjectAssetManager::default());
    let server = WgpuRenderFramework::new(asset_manager).expect("framework should initialize");
    let viewport_size = UVec2::new(320, 240);
    let viewport = server
        .create_viewport(RenderViewportDescriptor::new(viewport_size))
        .expect("viewport should be created");
    server
        .set_quality_profile(
            viewport,
            RenderQualityProfile::new("vg-clear-only-snapshot")
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
    let mut extract = world.to_render_frame_extract();
    extract.apply_viewport_size(viewport_size);
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 0,
        page_budget: 0,
        clusters: Vec::new(),
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        instances: Vec::new(),
        debug: RenderVirtualGeometryDebugState::default(),
    });

    server
        .submit_frame_extract(viewport, extract)
        .expect("virtual geometry submission should succeed");

    let snapshot = server
        .query_virtual_geometry_debug_snapshot()
        .expect("snapshot query should succeed")
        .expect("virtual geometry snapshot should be present");

    assert_eq!(
        snapshot.selected_clusters_source,
        RenderVirtualGeometrySelectedClusterSource::RenderPathClearOnly,
        "expected an enabled Virtual Geometry frame with no execution selections to keep the public snapshot on the explicit clear-only selected-cluster render-path source instead of collapsing that frame to Unavailable"
    );
    assert!(
        snapshot.selected_clusters.is_empty(),
        "expected the clear-only execution snapshot to keep selected clusters empty when the render path emitted no executed cluster selections"
    );
    assert_eq!(
        snapshot.visbuffer64_source,
        RenderVirtualGeometryVisBuffer64Source::RenderPathClearOnly,
        "expected an enabled Virtual Geometry frame with no execution selections to keep the public snapshot on the explicit clear-only render-path source instead of collapsing that frame to Unavailable"
    );
    assert!(
        snapshot.hardware_rasterization_records.is_empty(),
        "expected the clear-only execution snapshot to keep hardware-rasterization startup records empty when the render path emitted no executed cluster selections"
    );
    assert_eq!(
        snapshot.hardware_rasterization_source,
        RenderVirtualGeometryHardwareRasterizationSource::RenderPathClearOnly,
        "expected an enabled Virtual Geometry frame with no execution selections to keep the public snapshot on the explicit clear-only hardware-rasterization render-path source instead of collapsing that frame to Unavailable"
    );
    assert_eq!(
        snapshot.visbuffer64_clear_value,
        RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        "expected the clear-only execution snapshot to preserve the same published VisBuffer64 clear value as frames that also emit cluster entries"
    );
    assert!(
        snapshot.visbuffer64_entries.is_empty(),
        "expected the clear-only execution snapshot to keep the logical VisBuffer64 entry stream empty when the render path emitted no cluster writes"
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
        hierarchy_node_id: None,
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

fn visbuffer_color(cluster_id: u32, page_id: u32, lod_level: u8) -> [u8; 4] {
    let lod_level = u32::from(lod_level);
    [
        (32 + ((cluster_id * 17 + page_id * 13) % 192)) as u8,
        (32 + ((page_id * 11 + lod_level * 7) % 192)) as u8,
        (32 + ((cluster_id * 5 + lod_level * 19) % 192)) as u8,
        255,
    ]
}

fn pack_visbuffer64_entry(
    instance_index: Option<u32>,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    state: RenderVirtualGeometryExecutionState,
) -> u64 {
    const CLUSTER_BITS: u64 = 20;
    const PAGE_BITS: u64 = 20;
    const INSTANCE_BITS: u64 = 16;
    const LOD_BITS: u64 = 6;
    const CLUSTER_MASK: u64 = (1_u64 << CLUSTER_BITS) - 1;
    const PAGE_MASK: u64 = (1_u64 << PAGE_BITS) - 1;
    const INSTANCE_MASK: u64 = (1_u64 << INSTANCE_BITS) - 1;
    const LOD_MASK: u64 = (1_u64 << LOD_BITS) - 1;
    const PAGE_SHIFT: u64 = CLUSTER_BITS;
    const INSTANCE_SHIFT: u64 = PAGE_SHIFT + PAGE_BITS;
    const LOD_SHIFT: u64 = INSTANCE_SHIFT + INSTANCE_BITS;
    const STATE_SHIFT: u64 = LOD_SHIFT + LOD_BITS;

    let encoded_instance = u64::from(instance_index.unwrap_or(u16::MAX as u32)) & INSTANCE_MASK;
    let encoded_state = match state {
        RenderVirtualGeometryExecutionState::Resident => 0_u64,
        RenderVirtualGeometryExecutionState::PendingUpload => 1_u64,
        RenderVirtualGeometryExecutionState::Missing => 2_u64,
    };

    (u64::from(cluster_id) & CLUSTER_MASK)
        | ((u64::from(page_id) & PAGE_MASK) << PAGE_SHIFT)
        | (encoded_instance << INSTANCE_SHIFT)
        | ((u64::from(lod_level) & LOD_MASK) << LOD_SHIFT)
        | (encoded_state << STATE_SHIFT)
}
