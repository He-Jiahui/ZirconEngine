use super::*;
use crate::core::math::{Vec2, Vec3};

use crate::asset::{
    AssetReference, MeshVertex, ModelAsset, ModelPrimitiveAsset, VirtualGeometryAsset,
    VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryPageDependencyAsset, VirtualGeometryRootClusterRangeAsset,
};

#[test]
fn asset_manager_imports_model_toml_with_virtual_geometry_payload() {
    let root = unique_temp_project_root("asset_manager_model_toml");
    let paths = ProjectPaths::from_root(&root).unwrap();
    paths.ensure_layout().unwrap();
    ProjectManifest::new(
        "Sandbox",
        AssetUri::parse("res://scenes/main.scene.toml").unwrap(),
        1,
    )
    .save(paths.manifest_path())
    .unwrap();

    write_valid_wgsl(paths.assets_root().join("shaders").join("pbr.wgsl"));
    write_checker_png(paths.assets_root().join("textures").join("checker.png"));
    write_triangle_obj(paths.assets_root().join("models").join("triangle.obj"));
    write_default_material(paths.assets_root().join("materials").join("grid.zmaterial"));
    write_default_scene(paths.assets_root().join("scenes").join("main.scene.toml"));

    let authored_model = sample_virtual_geometry_model_asset();
    let model_path = paths
        .assets_root()
        .join("models")
        .join("nanite_teapot.model.toml");
    fs::write(&model_path, authored_model.to_toml_string().unwrap()).unwrap();
    let mut expected_model = authored_model;
    expected_model.primitives[0].mesh = Some(AssetReference::from_locator(
        AssetUri::parse("res://models/nanite_teapot.model.toml#Mesh0/Primitive0").unwrap(),
    ));

    let manager = project_asset_manager_with_first_wave_plugin_fixtures();
    manager
        .open_project(root.to_string_lossy().as_ref())
        .unwrap();

    let status = manager
        .asset_status("res://models/nanite_teapot.model.toml")
        .expect("model.toml status");
    assert!(status.imported);
    assert_eq!(status.kind, ResourceKind::Model);

    let model_id = manager
        .resolve_asset_id(&AssetUri::parse("res://models/nanite_teapot.model.toml").unwrap())
        .expect("model.toml asset id");
    let loaded = manager.load_model_asset(model_id).unwrap();

    assert_eq!(loaded, expected_model);

    let _ = fs::remove_dir_all(root);
}

fn sample_virtual_geometry_model_asset() -> ModelAsset {
    ModelAsset {
        uri: AssetUri::parse("res://models/nanite_teapot.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: vec![
                MeshVertex::new(Vec3::ZERO, Vec3::Y, Vec2::ZERO),
                MeshVertex::new(Vec3::X, Vec3::Y, Vec2::X),
                MeshVertex::new(Vec3::Z, Vec3::Y, Vec2::Y),
            ],
            indices: vec![0, 1, 2],
            mesh: None,
            virtual_geometry: Some(VirtualGeometryAsset {
                hierarchy_buffer: vec![
                    VirtualGeometryHierarchyNodeAsset {
                        node_id: 0,
                        parent_node_id: None,
                        child_node_ids: vec![1],
                        cluster_start: 0,
                        cluster_count: 0,
                        page_id: 0,
                        mip_level: 0,
                        bounds_center: [0.0, 0.0, 0.0],
                        bounds_radius: 2.0,
                        screen_space_error: 1.0,
                    },
                    VirtualGeometryHierarchyNodeAsset {
                        node_id: 1,
                        parent_node_id: Some(0),
                        child_node_ids: Vec::new(),
                        cluster_start: 0,
                        cluster_count: 1,
                        page_id: 10,
                        mip_level: 10,
                        bounds_center: [0.0, 0.0, 0.0],
                        bounds_radius: 1.0,
                        screen_space_error: 0.25,
                    },
                ],
                cluster_headers: vec![VirtualGeometryClusterHeaderAsset {
                    cluster_id: 100,
                    page_id: 10,
                    hierarchy_node_id: 1,
                    lod_level: 10,
                    parent_cluster_id: None,
                    bounds_center: [0.0, 0.0, 0.0],
                    bounds_radius: 0.5,
                    screen_space_error: 0.2,
                }],
                cluster_page_headers: vec![VirtualGeometryClusterPageHeaderAsset {
                    page_id: 10,
                    start_offset: 0,
                    payload_size_bytes: 32,
                }],
                cluster_page_data: vec![vec![1, 2, 3, 4]],
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
                    mesh_name: Some("NaniteTeapot".to_string()),
                    source_hint: Some("pipeline-test".to_string()),
                    notes: vec!["cooked vg payload".to_string()],
                },
            }),
        }],
    }
}
