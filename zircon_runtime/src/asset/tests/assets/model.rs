use crate::asset::{
    AssetReference, AssetUri, MeshVertex, ModelAsset, ModelAssetManagementRecord,
    ModelAssetManagementRecordSet, ModelPrimitiveAsset, ModelPrimitiveOverview,
    VirtualGeometryAsset, VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryPageDependencyAsset, VirtualGeometryRootClusterRangeAsset,
};
use crate::core::framework::render::{RenderMeshKind, RenderMeshTopology};
use crate::core::math::{Vec2, Vec3};
use crate::core::resource::ResourceId;

#[test]
fn model_asset_toml_roundtrip_preserves_virtual_geometry_payload() {
    let asset = ModelAsset {
        uri: AssetUri::parse("res://models/nanite_teapot.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: vec![
                MeshVertex {
                    position: Vec3::ZERO.to_array(),
                    normal: Vec3::Y.to_array(),
                    uv: Vec2::ZERO.to_array(),
                    joint_indices: [0, 1, 0, 0],
                    joint_weights: [0.75, 0.25, 0.0, 0.0],
                },
                MeshVertex::new(Vec3::X, Vec3::Y, Vec2::X),
                MeshVertex::new(Vec3::Z, Vec3::Y, Vec2::Y),
            ],
            indices: vec![0, 1, 2],
            mesh: Some(asset_reference(
                "res://models/nanite_teapot.model.toml#Mesh0/Primitive0",
            )),
            virtual_geometry: Some(sample_virtual_geometry_asset()),
        }],
    };

    let document = asset.to_toml_string().unwrap();
    let loaded = ModelAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, asset);
}

#[test]
fn model_asset_overview_reports_root_and_primitive_mesh_summary() {
    let asset = ModelAsset {
        uri: AssetUri::parse("res://models/overview.model.toml").unwrap(),
        primitives: vec![
            ModelPrimitiveAsset {
                vertices: triangle_vertices([0.0, 0.0, 0.0]),
                indices: vec![0, 1, 2],
                mesh: Some(asset_reference(
                    "res://models/overview.model.toml#Mesh0/Primitive0",
                )),
                virtual_geometry: None,
            },
            ModelPrimitiveAsset {
                vertices: triangle_vertices([10.0, 0.0, 2.0]),
                indices: vec![0, 1, 2, 2, 1, 0],
                mesh: None,
                virtual_geometry: Some(sample_virtual_geometry_asset()),
            },
        ],
    };

    let overview = asset.overview();

    assert_eq!(overview.uri, asset.uri);
    assert_eq!(overview.primitive_count, 2);
    assert_eq!(overview.vertex_count, 6);
    assert_eq!(overview.index_count, 9);
    assert_eq!(overview.mesh_reference_count, 1);
    assert_eq!(overview.render_primitive_count, 3);
    assert!(overview.has_virtual_geometry_payload);
    assert_eq!(overview.bounds.min, [0.0, 0.0, 0.0]);
    assert_eq!(overview.bounds.max, [11.0, 1.0, 2.0]);
    assert_eq!(overview.primitives.len(), 2);
    assert_eq!(
        overview.primitives[0],
        ModelPrimitiveOverview {
            primitive_index: 0,
            mesh: Some(asset_reference(
                "res://models/overview.model.toml#Mesh0/Primitive0",
            )),
            topology: RenderMeshTopology::TriangleList,
            primitive_kind: RenderMeshKind::Planar2d,
            suitable_for_2d: true,
            suitable_for_3d: true,
            vertex_count: 3,
            index_count: 3,
            render_primitive_count: 1,
            has_virtual_geometry_payload: false,
            bounds: asset.primitives[0].render_mesh_descriptor().bounds,
        }
    );
    assert_eq!(overview.primitives[1].primitive_index, 1);
    assert_eq!(overview.primitives[1].mesh, None);
    assert_eq!(
        overview.primitives[1].primitive_kind,
        RenderMeshKind::Spatial3d
    );
    assert_eq!(overview.primitives[1].render_primitive_count, 2);
    assert!(overview.primitives[1].has_virtual_geometry_payload);
}

#[test]
fn model_asset_overview_handles_empty_model_roots() {
    let asset = ModelAsset {
        uri: AssetUri::parse("res://models/empty.model.toml").unwrap(),
        primitives: Vec::new(),
    };

    let overview = asset.overview();

    assert_eq!(overview.uri, asset.uri);
    assert_eq!(overview.primitive_count, 0);
    assert_eq!(overview.vertex_count, 0);
    assert_eq!(overview.index_count, 0);
    assert_eq!(overview.mesh_reference_count, 0);
    assert_eq!(overview.render_primitive_count, 0);
    assert!(!overview.has_virtual_geometry_payload);
    assert!(overview.primitives.is_empty());
    assert_eq!(overview.bounds.radius, 0.0);
}

#[test]
fn model_asset_management_record_wraps_id_and_overview() {
    let asset = ModelAsset {
        uri: AssetUri::parse("res://models/managed.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: triangle_vertices([0.0, 0.0, 1.0]),
            indices: vec![0, 1, 2],
            mesh: Some(asset_reference(
                "res://models/managed.model.toml#Mesh0/Primitive0",
            )),
            virtual_geometry: None,
        }],
    };
    let model_id = ResourceId::from_stable_label("model:managed");

    let record = asset.management_record(model_id);

    assert_eq!(
        record,
        ModelAssetManagementRecord {
            model_id,
            overview: asset.overview(),
        }
    );
    assert_eq!(record.overview.primitive_count, 1);
    assert_eq!(record.overview.primitives[0].primitive_index, 0);
}

#[test]
fn model_asset_management_record_set_sorts_and_summarizes_records() {
    let flat_model = ModelAsset {
        uri: AssetUri::parse("res://models/flat.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: triangle_vertices([0.0, 0.0, 0.0]),
            indices: vec![0, 1, 2],
            mesh: Some(asset_reference(
                "res://models/flat.model.toml#Mesh0/Primitive0",
            )),
            virtual_geometry: None,
        }],
    };
    let virtualized_model = ModelAsset {
        uri: AssetUri::parse("res://models/virtualized.model.toml").unwrap(),
        primitives: vec![ModelPrimitiveAsset {
            vertices: triangle_vertices([10.0, 0.0, 0.0]),
            indices: vec![0, 1, 2, 2, 1, 0],
            mesh: Some(asset_reference(
                "res://models/virtualized.model.toml#Mesh0/Primitive0",
            )),
            virtual_geometry: Some(sample_virtual_geometry_asset()),
        }],
    };
    let flat_id = ResourceId::from_stable_label("model:flat-record-set");
    let virtualized_id = ResourceId::from_stable_label("model:virtualized-record-set");

    let record_set = ModelAssetManagementRecordSet::from_records(vec![
        virtualized_model.management_record(virtualized_id),
        flat_model.management_record(flat_id),
    ]);

    let mut expected_ids = vec![flat_id, virtualized_id];
    expected_ids.sort();
    let record_ids = record_set
        .records
        .iter()
        .map(|record| record.model_id)
        .collect::<Vec<_>>();
    assert_eq!(record_ids, expected_ids);
    assert_eq!(record_set.records.len(), 2);
    let summary = &record_set.summary;
    assert_eq!(summary.model_count, 2);
    assert_eq!(summary.primitive_count, 2);
    assert_eq!(summary.vertex_count, 6);
    assert_eq!(summary.index_count, 9);
    assert_eq!(summary.render_primitive_count, 3);
    assert_eq!(summary.mesh_referenced_model_count, 2);
    assert_eq!(summary.mesh_reference_count, 2);
    assert_eq!(summary.virtual_geometry_model_count, 1);
    assert_eq!(summary.virtual_geometry_primitive_count, 1);
}

#[test]
fn model_asset_direct_references_deduplicate_primitive_meshes() {
    let shared_mesh = asset_reference("res://models/shared.model.toml#Mesh0/Primitive0");
    let asset = ModelAsset {
        uri: AssetUri::parse("res://models/shared.model.toml").unwrap(),
        primitives: vec![
            ModelPrimitiveAsset {
                vertices: triangle_vertices([0.0, 0.0, 0.0]),
                indices: vec![0, 1, 2],
                mesh: Some(shared_mesh.clone()),
                virtual_geometry: None,
            },
            ModelPrimitiveAsset {
                vertices: triangle_vertices([1.0, 0.0, 0.0]),
                indices: vec![0, 1, 2],
                mesh: Some(shared_mesh.clone()),
                virtual_geometry: None,
            },
        ],
    };

    assert_eq!(asset.direct_references(), vec![shared_mesh]);
}

fn triangle_vertices(offset: [f32; 3]) -> Vec<MeshVertex> {
    [
        Vec3::new(offset[0], offset[1], offset[2]),
        Vec3::new(offset[0] + 1.0, offset[1], offset[2]),
        Vec3::new(offset[0], offset[1] + 1.0, offset[2]),
    ]
    .into_iter()
    .map(|position| MeshVertex::new(position, Vec3::Y, Vec2::ZERO))
    .collect()
}

fn asset_reference(uri: &str) -> AssetReference {
    AssetReference::from_locator(AssetUri::parse(uri).unwrap())
}

fn sample_virtual_geometry_asset() -> VirtualGeometryAsset {
    VirtualGeometryAsset {
        hierarchy_buffer: vec![
            VirtualGeometryHierarchyNodeAsset {
                node_id: 0,
                parent_node_id: None,
                child_node_ids: vec![1, 2],
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
                cluster_count: 2,
                page_id: 10,
                mip_level: 10,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.25,
            },
            VirtualGeometryHierarchyNodeAsset {
                node_id: 2,
                parent_node_id: Some(0),
                child_node_ids: Vec::new(),
                cluster_start: 2,
                cluster_count: 1,
                page_id: 30,
                mip_level: 10,
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 1.0,
                screen_space_error: 0.2,
            },
        ],
        cluster_headers: vec![
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 100,
                page_id: 10,
                hierarchy_node_id: 1,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: [0.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.2,
            },
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 200,
                page_id: 20,
                hierarchy_node_id: 1,
                lod_level: 9,
                parent_cluster_id: Some(100),
                bounds_center: [0.5, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.1,
            },
            VirtualGeometryClusterHeaderAsset {
                cluster_id: 300,
                page_id: 30,
                hierarchy_node_id: 2,
                lod_level: 10,
                parent_cluster_id: Some(100),
                bounds_center: [1.0, 0.0, 0.0],
                bounds_radius: 0.5,
                screen_space_error: 0.15,
            },
        ],
        cluster_page_headers: vec![
            VirtualGeometryClusterPageHeaderAsset {
                page_id: 10,
                start_offset: 0,
                payload_size_bytes: 32,
            },
            VirtualGeometryClusterPageHeaderAsset {
                page_id: 20,
                start_offset: 32,
                payload_size_bytes: 32,
            },
            VirtualGeometryClusterPageHeaderAsset {
                page_id: 30,
                start_offset: 64,
                payload_size_bytes: 32,
            },
        ],
        cluster_page_data: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
        root_page_table: vec![10, 30],
        page_dependencies: vec![
            VirtualGeometryPageDependencyAsset {
                page_id: 10,
                parent_page_id: None,
                child_page_ids: vec![20],
            },
            VirtualGeometryPageDependencyAsset {
                page_id: 20,
                parent_page_id: Some(10),
                child_page_ids: Vec::new(),
            },
            VirtualGeometryPageDependencyAsset {
                page_id: 30,
                parent_page_id: None,
                child_page_ids: Vec::new(),
            },
        ],
        root_cluster_ranges: vec![VirtualGeometryRootClusterRangeAsset {
            node_id: 0,
            cluster_start: 0,
            cluster_count: 3,
        }],
        debug: VirtualGeometryDebugMetadataAsset {
            mesh_name: Some("NaniteTeapot".to_string()),
            source_hint: Some("unit-test".to_string()),
            notes: vec!["teaching payload".to_string()],
        },
    }
}
