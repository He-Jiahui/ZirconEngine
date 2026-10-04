use std::collections::BTreeMap;

use super::*;
use crate::asset::{AssetUri, MESH_ATTRIBUTE_JOINT_WEIGHT, MESH_ATTRIBUTE_POSITION};
use crate::core::framework::render::RenderMeshTopology;

#[test]
fn morph_bounds_cover_positive_and_negative_weighted_delta_extrema() {
    let deformation = PreparedGeometryDeformation {
        morph_target_delta_bounds: vec![
            RenderMeshBounds::from_min_max([-2.0, -4.0, -6.0], [4.0, 6.0, 8.0]),
            RenderMeshBounds::from_min_max([-3.0, -5.0, -7.0], [2.0, 4.0, 6.0]),
        ],
        has_skinning: false,
    };

    let bounds = deformation.local_bounds_for_morph_weights(
        RenderMeshBounds::from_min_max([10.0, 20.0, 30.0], [12.0, 22.0, 32.0]),
        &[0.5, -0.25],
    );

    assert_eq!(bounds.min, [8.5, 17.0, 25.5]);
    assert_eq!(bounds.max, [14.75, 26.25, 37.75]);
}

#[test]
fn mesh_skinning_detection_does_not_build_a_model_primitive() {
    let source = include_str!("../prepared_geometry_deformation.rs")
        .split_once("#[cfg(test)]")
        .expect("production source and tests must remain separated")
        .0;

    assert!(!source.contains("to_model_primitive"));
}

#[test]
fn mesh_skinning_detection_reads_joint_weight_attributes_directly() {
    let mut attributes = BTreeMap::new();
    attributes.insert(
        MESH_ATTRIBUTE_POSITION.to_string(),
        MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
    );
    attributes.insert(
        MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
        MeshAttributeValues::Float32x4(vec![
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/skinned.zmesh").expect("valid test URI"),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(PreparedGeometryDeformation::from_mesh_asset(&mesh).has_skinning());
}

#[test]
fn invalid_mesh_attributes_do_not_enable_skinning() {
    let mut attributes = BTreeMap::new();
    attributes.insert(
        MESH_ATTRIBUTE_POSITION.to_string(),
        MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0]]),
    );
    attributes.insert(
        MESH_ATTRIBUTE_JOINT_WEIGHT.to_string(),
        MeshAttributeValues::Float32x4(vec![[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/invalid-skinned.zmesh").expect("valid test URI"),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert!(!PreparedGeometryDeformation::from_mesh_asset(&mesh).has_skinning());
}
