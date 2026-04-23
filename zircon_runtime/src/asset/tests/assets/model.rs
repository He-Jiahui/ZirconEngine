use crate::asset::{
    AssetUri, MeshVertex, ModelAsset, ModelPrimitiveAsset, VirtualGeometryAsset,
    VirtualGeometryClusterHeaderAsset, VirtualGeometryClusterPageHeaderAsset,
    VirtualGeometryDebugMetadataAsset, VirtualGeometryHierarchyNodeAsset,
    VirtualGeometryRootClusterRangeAsset,
};
use crate::core::math::{Vec2, Vec3};

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
            virtual_geometry: Some(sample_virtual_geometry_asset()),
        }],
    };

    let document = asset.to_toml_string().unwrap();
    let loaded = ModelAsset::from_toml_str(&document).unwrap();

    assert_eq!(loaded, asset);
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
