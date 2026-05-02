use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage, RenderVirtualGeometryPageDependency,
};
use zircon_runtime::core::math::{Transform, Vec3};
use zircon_runtime::core::resource::ResourceId;

#[test]
fn default_virtual_geometry_extract_starts_without_instances_or_debug_overrides() {
    let extract = RenderVirtualGeometryExtract::default();

    assert!(extract.instances.is_empty());
    assert!(extract.page_dependencies.is_empty());
    assert_eq!(extract.debug, RenderVirtualGeometryDebugState::default());
}

#[test]
fn virtual_geometry_extract_preserves_instance_ranges_and_debug_state() {
    let model_id = ResourceId::from_stable_label("res://models/cooked.model.toml");
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 2,
        clusters: vec![RenderVirtualGeometryCluster {
            entity: 7,
            cluster_id: 100,
            page_id: 10,
            lod_level: 10,
            parent_cluster_id: None,
            hierarchy_node_id: None,
            bounds_center: Vec3::new(1.0, 2.0, 3.0),
            bounds_radius: 0.5,
            screen_space_error: 0.25,
        }],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 128,
        }],
        page_dependencies: vec![RenderVirtualGeometryPageDependency {
            page_id: 10,
            parent_page_id: None,
            child_page_ids: Vec::new(),
        }],
        instances: vec![RenderVirtualGeometryInstance {
            entity: 7,
            source_model: Some(model_id),
            transform: Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
            cluster_offset: 0,
            cluster_count: 1,
            page_offset: 0,
            page_count: 1,
            mesh_name: Some("NaniteTeapot".to_string()),
            source_hint: Some("integration-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(10),
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: true,
            print_leaf_clusters: true,
        },
    };

    let cloned = extract.clone();

    assert_eq!(cloned, extract);
    assert_eq!(extract.instances.len(), 1);
    assert_eq!(extract.instances[0].source_model, Some(model_id));
    assert_eq!(extract.page_dependencies[0].page_id, 10);
    assert_eq!(extract.instances[0].cluster_count, 1);
    assert_eq!(extract.instances[0].page_count, 1);
    assert_eq!(extract.debug.forced_mip, Some(10));
    assert!(extract.debug.freeze_cull);
    assert!(extract.debug.visualize_bvh);
    assert!(extract.debug.visualize_visbuffer);
    assert!(extract.debug.print_leaf_clusters);
}
