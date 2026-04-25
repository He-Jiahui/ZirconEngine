use zircon_runtime::core::framework::render::{
    RenderFrameExtract, RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExtract, RenderVirtualGeometryInstance, RenderVirtualGeometryPage,
    RenderWorldSnapshotHandle, SceneViewportExtractRequest, ViewportRenderSettings,
};
use zircon_runtime::core::math::{Transform, UVec2, Vec3};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceId};
use zircon_runtime::graphics::VisibilityContext;
use zircon_runtime::scene::world::World;

#[test]
fn visibility_context_uses_instance_ranges_and_forced_mip_for_virtual_geometry_selection() {
    let mut world = World::new();
    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut extract = world.to_render_frame_extract();
    extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 8,
        page_budget: 2,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, Vec3::new(0.0, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 20, 200, 10, Vec3::new(0.1, 0.0, 0.0), 8.0),
            virtual_cluster(mesh, 30, 300, 10, Vec3::new(0.2, 0.0, 0.0), 7.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![
            virtual_page(100, false),
            virtual_page(200, false),
            virtual_page(300, true),
        ],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: Some(ResourceId::from_stable_label(
                "res://models/virtual_geometry.model.toml",
            )),
            transform: Transform::default(),
            cluster_offset: 1,
            cluster_count: 2,
            page_offset: 1,
            page_count: 2,
            mesh_name: Some("NaniteContractMesh".to_string()),
            source_hint: Some("integration-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(10),
            ..RenderVirtualGeometryDebugState::default()
        },
    });

    let context = VisibilityContext::from(&extract);

    assert_eq!(
        context
            .virtual_geometry_visible_clusters
            .iter()
            .map(|cluster| {
                (
                    cluster.cluster_id,
                    cluster.page_id,
                    cluster.lod_level,
                    cluster.cluster_ordinal,
                    cluster.cluster_count,
                )
            })
            .collect::<Vec<_>>(),
        vec![(20, 200, 10, 0, 2), (30, 300, 10, 1, 2)]
    );
    assert_eq!(
        context.virtual_geometry_page_upload_plan.requested_pages,
        vec![200]
    );
}

#[test]
fn visibility_context_freeze_cull_preserves_previous_virtual_geometry_selection_and_requests() {
    let mut world = World::new();
    let mesh = world.spawn_mesh_node(
        model_handle("res://models/virtual_geometry.obj"),
        material_handle("res://materials/virtual_geometry.material.toml"),
    );
    world
        .update_transform(mesh, Transform::from_translation(Vec3::ZERO))
        .expect("mesh transform should update");

    let mut previous_extract = world.to_render_frame_extract();
    previous_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, Vec3::new(0.0, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 20, 200, 0, Vec3::new(0.1, 0.0, 0.0), 6.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![virtual_page(100, false), virtual_page(200, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: None,
            source_hint: None,
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    });
    let previous_context = VisibilityContext::from(&previous_extract);
    assert_eq!(
        previous_context
            .virtual_geometry_feedback
            .visible_cluster_ids,
        vec![10]
    );
    assert_eq!(
        previous_context.virtual_geometry_feedback.requested_pages,
        vec![100]
    );

    let mut current_extract = world.to_render_frame_extract();
    current_extract.geometry.virtual_geometry = Some(RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: vec![
            virtual_cluster(mesh, 10, 100, 0, Vec3::new(100.0, 0.0, 0.0), 9.0),
            virtual_cluster(mesh, 20, 200, 0, Vec3::new(0.1, 0.0, 0.0), 12.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![virtual_page(100, false), virtual_page(200, false)],
        instances: vec![RenderVirtualGeometryInstance {
            entity: mesh,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: None,
            source_hint: None,
        }],
        debug: RenderVirtualGeometryDebugState {
            freeze_cull: true,
            ..RenderVirtualGeometryDebugState::default()
        },
    });

    let current_context = VisibilityContext::from_extract_with_history(
        &current_extract,
        Some(&previous_context.history_snapshot),
    );

    assert_eq!(
        current_context
            .virtual_geometry_visible_clusters
            .iter()
            .map(|cluster| cluster.cluster_id)
            .collect::<Vec<_>>(),
        vec![10]
    );
    assert_eq!(
        current_context
            .virtual_geometry_page_upload_plan
            .requested_pages,
        vec![100]
    );
    assert_eq!(
        current_context
            .virtual_geometry_feedback
            .visible_cluster_ids,
        vec![10]
    );
}

#[test]
fn viewport_virtual_geometry_debug_roundtrips_into_render_frame_extract() {
    let world = World::new();
    let debug = RenderVirtualGeometryDebugState {
        forced_mip: Some(10),
        freeze_cull: true,
        visualize_bvh: true,
        visualize_visbuffer: true,
        print_leaf_clusters: true,
    };
    let request = SceneViewportExtractRequest {
        settings: ViewportRenderSettings::default(),
        active_camera_override: None,
        camera: None,
        viewport_size: Some(UVec2::new(320, 240)),
        virtual_geometry_debug: Some(debug),
    };

    let snapshot = world.build_viewport_render_packet(&request);
    assert_eq!(snapshot.virtual_geometry_debug, Some(debug));

    let extract = RenderFrameExtract::from_snapshot(RenderWorldSnapshotHandle::new(77), snapshot);
    assert_eq!(extract.geometry.virtual_geometry_debug, Some(debug));
    assert_eq!(
        extract.to_scene_snapshot().virtual_geometry_debug,
        Some(debug)
    );
}

fn model_handle(label: &str) -> ResourceHandle<ModelMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn material_handle(label: &str) -> ResourceHandle<MaterialMarker> {
    ResourceHandle::new(ResourceId::from_stable_label(label))
}

fn virtual_cluster(
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
        bounds_radius: 8.0,
        screen_space_error,
    }
}

fn virtual_page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 128,
    }
}
