use std::collections::BTreeMap;

use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndexFormat, MeshIndices, MeshValidationError,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION,
};
use crate::core::framework::render::RenderMeshTopology;

#[test]
fn mesh_asset_generates_missing_flat_normals_for_unindexed_triangle_list() {
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/generated-normals.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: position_only_attributes(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert!(mesh.try_generate_missing_flat_normals().unwrap());

    let expected = [
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ];
    let normals = mesh
        .attributes
        .get(MESH_ATTRIBUTE_NORMAL)
        .unwrap()
        .as_float32x3()
        .unwrap();
    assert_eq!(normals, expected.as_slice());
    assert_eq!(mesh.validate(), Ok(()));
    assert!(mesh
        .attribute_summaries()
        .iter()
        .any(|summary| summary.name == MESH_ATTRIBUTE_NORMAL && summary.len == 6));
}

#[test]
fn mesh_asset_does_not_overwrite_existing_normals() {
    let mut attributes =
        position_only_attributes(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
    attributes.insert(
        MESH_ATTRIBUTE_NORMAL.to_string(),
        MeshAttributeValues::Float32x3(vec![[0.0, 0.0, -1.0]; 3]),
    );
    let mut mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/existing-normals.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert!(!mesh.try_generate_missing_flat_normals().unwrap());
    assert_eq!(
        mesh.attributes
            .get(MESH_ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float32x3()
            .unwrap(),
        [[0.0, 0.0, -1.0]; 3].as_slice()
    );
}

#[test]
fn mesh_asset_rejects_flat_normal_generation_for_unsupported_mesh_shapes() {
    let mut indexed = MeshAsset {
        uri: AssetUri::parse("res://meshes/indexed-normals.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: position_only_attributes(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]),
        indices: Some(MeshIndices::U16(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let mut lines = MeshAsset {
        uri: AssetUri::parse("res://meshes/line-normals.zmesh").unwrap(),
        topology: RenderMeshTopology::LineList,
        attributes: position_only_attributes(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        indexed.try_generate_missing_flat_normals().unwrap_err(),
        MeshValidationError::FlatNormalGenerationRequiresUnindexedMesh
    );
    assert_eq!(
        lines.try_generate_missing_flat_normals().unwrap_err(),
        MeshValidationError::NormalGenerationRequiresTriangleList {
            topology: RenderMeshTopology::LineList,
        }
    );
}

#[test]
fn mesh_asset_generates_missing_smooth_normals_for_indexed_triangle_list() {
    let mut mesh = indexed_smooth_normal_mesh("res://meshes/smooth-normals.zmesh");

    assert!(mesh.try_generate_missing_smooth_normals().unwrap());

    assert_normals_approx(
        mesh.attributes
            .get(MESH_ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float32x3()
            .unwrap(),
        &smooth_normal_expected_values(),
    );
    assert_eq!(mesh.index_format(), Some(MeshIndexFormat::U16));
    assert_eq!(mesh.validate(), Ok(()));
}

#[test]
fn mesh_asset_generate_missing_normals_defaults_by_indexing() {
    let mut unindexed = MeshAsset {
        uri: AssetUri::parse("res://meshes/default-flat-normals.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: position_only_attributes(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let mut indexed = indexed_smooth_normal_mesh("res://meshes/default-smooth-normals.zmesh");

    assert!(unindexed.try_generate_missing_normals().unwrap());
    assert!(indexed.try_generate_missing_normals().unwrap());

    assert_eq!(
        unindexed
            .attributes
            .get(MESH_ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float32x3()
            .unwrap(),
        [[0.0, 0.0, 1.0]; 3].as_slice()
    );
    assert_normals_approx(
        indexed
            .attributes
            .get(MESH_ATTRIBUTE_NORMAL)
            .unwrap()
            .as_float32x3()
            .unwrap(),
        &smooth_normal_expected_values(),
    );
}

#[test]
fn mesh_asset_rejects_smooth_normal_generation_for_unsupported_mesh_shapes() {
    let mut unindexed = MeshAsset {
        uri: AssetUri::parse("res://meshes/unindexed-smooth-normals.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: position_only_attributes(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ]),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let mut lines = MeshAsset {
        uri: AssetUri::parse("res://meshes/indexed-line-smooth-normals.zmesh").unwrap(),
        topology: RenderMeshTopology::LineList,
        attributes: position_only_attributes(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]]),
        indices: Some(MeshIndices::U16(vec![0, 1])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        unindexed.try_generate_missing_smooth_normals().unwrap_err(),
        MeshValidationError::SmoothNormalGenerationRequiresIndexedMesh
    );
    assert_eq!(
        lines.try_generate_missing_smooth_normals().unwrap_err(),
        MeshValidationError::NormalGenerationRequiresTriangleList {
            topology: RenderMeshTopology::LineList,
        }
    );
}

fn position_only_attributes(positions: Vec<[f32; 3]>) -> BTreeMap<String, MeshAttributeValues> {
    BTreeMap::from([(
        MESH_ATTRIBUTE_POSITION.to_string(),
        MeshAttributeValues::Float32x3(positions),
    )])
}

fn indexed_smooth_normal_mesh(uri: &str) -> MeshAsset {
    MeshAsset {
        uri: AssetUri::parse(uri).unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: position_only_attributes(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]),
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 0, 3, 1])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    }
}

fn smooth_normal_expected_values() -> [[f32; 3]; 4] {
    let diagonal = std::f32::consts::FRAC_1_SQRT_2;
    [
        [0.0, diagonal, diagonal],
        [0.0, diagonal, diagonal],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0],
    ]
}

fn assert_normals_approx(actual: &[[f32; 3]], expected: &[[f32; 3]]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_vec3_approx(*actual, *expected);
    }
}

fn assert_vec3_approx(actual: [f32; 3], expected: [f32; 3]) {
    for component in 0..3 {
        assert!(
            (actual[component] - expected[component]).abs() < 0.000001,
            "normal component {component} expected {:?} but got {:?}",
            expected,
            actual
        );
    }
}
