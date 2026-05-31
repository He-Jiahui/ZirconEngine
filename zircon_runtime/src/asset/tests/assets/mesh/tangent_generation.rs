use std::collections::BTreeMap;

use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshValidationError,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::render::RenderMeshTopology;

#[test]
fn mesh_asset_generates_missing_tangents_for_unindexed_triangle_list() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/unindexed-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: quad_unindexed_attributes(),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert!(mesh.try_generate_missing_tangents().unwrap());

    assert_tangents_approx(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        &vec![[1.0, 0.0, 0.0, 1.0]; 6],
    );
    assert_eq!(mesh.validate(), Ok(()));
}

#[test]
fn mesh_asset_generates_missing_tangents_for_indexed_triangle_list() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/indexed-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: quad_indexed_attributes(),
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 2, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert!(mesh.try_generate_missing_tangents().unwrap());

    assert_tangents_approx(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        &[[1.0, 0.0, 0.0, 1.0]; 4],
    );
    assert_eq!(mesh.validate(), Ok(()));
}

#[test]
fn mesh_asset_does_not_overwrite_existing_tangents() {
    let mut attributes = quad_indexed_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_TANGENT.to_string(),
        MeshAttributeValues::Float32x4(vec![[0.0, 1.0, 0.0, -1.0]; 4]),
    );
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/existing-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 2, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert!(!mesh.try_generate_missing_tangents().unwrap());
    assert_eq!(
        mesh.attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .as_float32x4()
            .unwrap(),
        [[0.0, 1.0, 0.0, -1.0]; 4].as_slice()
    );
}

#[test]
fn mesh_asset_rejects_tangent_generation_for_missing_inputs_or_topology() {
    let mut missing_normal = MeshAsset {
        uri: AssetUri::parse("res://meshes/tangents-no-normal.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]),
            ),
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let mut missing_uv = MeshAsset {
        uri: AssetUri::parse("res://meshes/tangents-no-uv.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [0.0, 1.0, 0.0],
                ]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let mut lines = MeshAsset {
        uri: AssetUri::parse("res://meshes/line-tangents.zmesh").unwrap(),
        topology: RenderMeshTopology::LineList,
        attributes: BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 2]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [1.0, 0.0]]),
            ),
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        missing_normal.try_generate_missing_tangents().unwrap_err(),
        MeshValidationError::TangentGenerationMissingAttribute {
            attribute: MESH_ATTRIBUTE_NORMAL,
        }
    );
    assert_eq!(
        missing_uv.try_generate_missing_tangents().unwrap_err(),
        MeshValidationError::TangentGenerationMissingAttribute {
            attribute: MESH_ATTRIBUTE_UV0,
        }
    );
    assert_eq!(
        lines.try_generate_missing_tangents().unwrap_err(),
        MeshValidationError::TangentGenerationRequiresTriangleList {
            topology: RenderMeshTopology::LineList,
        }
    );
}

fn quad_indexed_attributes() -> BTreeMap<String, MeshAttributeValues> {
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

fn quad_unindexed_attributes() -> BTreeMap<String, MeshAttributeValues> {
    BTreeMap::from([
        (
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ]),
        ),
        (
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 6]),
        ),
        (
            MESH_ATTRIBUTE_UV0.to_string(),
            MeshAttributeValues::Float32x2(vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
            ]),
        ),
    ])
}

fn assert_tangents_approx(actual: &[[f32; 4]], expected: &[[f32; 4]]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        for component in 0..4 {
            assert!(
                (actual[component] - expected[component]).abs() < 0.000001,
                "tangent component {component} expected {:?} but got {:?}",
                expected,
                actual
            );
        }
    }
}
