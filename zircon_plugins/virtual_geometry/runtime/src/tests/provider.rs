use super::*;
use zircon_runtime::asset::{
    AssetUri, ModelPrimitiveAsset, VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset,
    VirtualGeometryClusterPageHeaderAsset, VirtualGeometryDebugMetadataAsset,
    VirtualGeometryHierarchyNodeAsset, VirtualGeometryPageDependencyAsset,
    VirtualGeometryRootClusterRangeAsset,
};
use zircon_runtime::core::framework::render::{
    render_mesh_stable_instance_key, render_mesh_transform_revision, RenderLayerSet,
    RenderMeshStaticState, RendererCommon,
};
use zircon_runtime::core::math::{Transform, Vec3, Vec4};
use zircon_runtime::core::resource::{MaterialMarker, ModelMarker, ResourceHandle};
use zircon_runtime::scene::components::{default_render_layer_mask, Mobility};

#[test]
fn provider_builds_neutral_extract_output_from_cooked_model_meshes() {
    let provider = PluginVirtualGeometryRuntimeProvider;
    let model_id = ResourceId::from_stable_label("res://models/provider-vg.model.toml");
    let material_id = ResourceId::from_stable_label("builtin://material/default");
    let model = cooked_model_asset();
    let node_id = 44;
    let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let mesh = RenderMeshSnapshot {
        node_id,
        stable_instance_key: render_mesh_stable_instance_key(node_id, 0),
        transform_revision: render_mesh_transform_revision(&transform),
        transform,
        model: ResourceHandle::<ModelMarker>::new(model_id),
        mesh: None,
        material: ResourceHandle::<MaterialMarker>::new(material_id),
        mesh_lod: None,
        morph_weights: Vec::new(),
        tint: Vec4::ONE,
        mobility: Mobility::Dynamic,
        static_state: RenderMeshStaticState::from_transform_static(false),
        common: RendererCommon {
            layer_mask: RenderLayerSet::from_scene_schema_v1_mask(default_render_layer_mask()),
            is_static: false,
            ..RendererCommon::default()
        },
    };
    let mut load_model = |requested_id| (requested_id == model_id).then(|| model.clone());

    let output = provider
        .build_extract_from_meshes(
            &[mesh],
            Some(RenderVirtualGeometryDebugState {
                forced_mip: Some(10),
                visualize_bvh: true,
                ..RenderVirtualGeometryDebugState::default()
            }),
            &mut load_model,
        )
        .expect("provider should build automatic VG output from cooked model data");

    assert_eq!(output.extract().instances.len(), 1);
    assert_eq!(output.extract().instances[0].source_model, Some(model_id));
    assert_eq!(output.extract().debug.forced_mip, Some(10));
    assert_eq!(output.extract().pages.len(), 1);
    assert_eq!(output.extract().page_dependencies.len(), 1);
    assert!(!output.cpu_reference_instances().is_empty());
    assert!(!output.bvh_visualization_instances().is_empty());
    assert!(output.resident_page_payloads().is_empty());
}

fn cooked_model_asset() -> ModelAsset {
    ModelAsset {
        uri: AssetUri::parse("res://models/provider-vg.model.toml")
            .expect("model uri should parse"),
        primitives: vec![ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: Some(VirtualGeometryAsset {
                hierarchy_buffer: vec![VirtualGeometryHierarchyNodeAsset {
                    node_id: 0,
                    parent_node_id: None,
                    child_node_ids: Vec::new(),
                    cluster_start: 0,
                    cluster_count: 1,
                    page_id: 10,
                    mip_level: 10,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 1.0,
                    screen_space_error: 0.25,
                }],
                cluster_headers: vec![VirtualGeometryClusterHeaderAsset {
                    cluster_id: 7,
                    page_id: 10,
                    hierarchy_node_id: 0,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 1.0,
                    screen_space_error: 0.25,
                }],
                cluster_page_headers: vec![VirtualGeometryClusterPageHeaderAsset {
                    page_id: 10,
                    start_offset: 0,
                    payload_size_bytes: 64,
                }],
                cluster_page_data: vec![vec![0; 64]],
                root_page_table: vec![10],
                page_dependencies: vec![VirtualGeometryPageDependencyAsset {
                    page_id: 10,
                    parent_page_id: None,
                    child_page_ids: Vec::new(),
                }],
                root_cluster_ranges: vec![VirtualGeometryRootClusterRangeAsset {
                    node_id: 0,
                    cluster_start: 0,
                    cluster_count: 1,
                }],
                debug: VirtualGeometryDebugMetadataAsset {
                    mesh_name: Some("ProviderCookedMesh".to_string()),
                    source_hint: Some("provider-test".to_string()),
                    notes: Vec::new(),
                },
            }),
        }],
    }
}
