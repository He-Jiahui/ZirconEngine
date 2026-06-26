use std::collections::BTreeMap;
use std::fs;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetImporter, AssetUri, ImportedAsset, MeshAsset, MeshAssetManagementRecord,
    MeshAssetManagementRecordSet, MeshAttributeFormat, MeshAttributeSummary, MeshAttributeValues,
    MeshIndexFormat, MeshIndices, MeshMorphTargetAsset, MeshSkinAsset, MeshValidationError,
    MeshVertex, ModelPrimitiveAsset, VirtualGeometryAsset, ZMeshDocument, MESH_ATTRIBUTE_COLOR,
    MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_NORMAL,
    MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0, MESH_ATTRIBUTE_UV1,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::math::{Vec2, Vec3};
use crate::core::resource::ResourceId;

mod conversion_import;
mod document_roundtrip;
mod morph_targets;
mod normal_generation;
mod summaries;
mod tangent_generation;
mod validation;

fn sample_zmesh_document(indices: MeshIndices) -> ZMeshDocument {
    ZMeshDocument {
        version: crate::asset::ZMESH_DOCUMENT_VERSION,
        name: Some("Triangle".to_string()),
        topology: RenderMeshTopology::TriangleList,
        attributes: triangle_attributes(),
        indices: Some(indices),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: Some(sample_virtual_geometry()),
    }
}

fn triangle_attributes() -> BTreeMap<String, MeshAttributeValues> {
    BTreeMap::from([
        (
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        ),
        (
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]]),
        ),
        (
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
        ),
    ])
}

fn quad_attributes() -> BTreeMap<String, MeshAttributeValues> {
    BTreeMap::from([
        (
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]),
        ),
        (
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 4]),
        ),
        (
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]),
        ),
    ])
}

fn sample_virtual_geometry() -> VirtualGeometryAsset {
    VirtualGeometryAsset {
        debug: crate::asset::VirtualGeometryDebugMetadataAsset {
            mesh_name: Some("Triangle".to_string()),
            source_hint: Some("zmesh-roundtrip".to_string()),
            notes: vec!["unit-test".to_string()],
        },
        ..Default::default()
    }
}

fn identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
