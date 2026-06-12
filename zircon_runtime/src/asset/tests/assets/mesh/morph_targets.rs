use std::collections::BTreeMap;

use crate::asset::{
    AssetUri, MeshAttributeFormat, MeshAttributeSummary, MeshAttributeValues, MeshIndices,
    MeshMorphTargetAsset, MeshMorphTargetAttributeSummary, MeshValidationError,
    MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT,
};
use crate::core::math::Vec3;

use super::sample_zmesh_document;

#[test]
fn mesh_asset_rejects_morph_target_attribute_length_mismatch() {
    let mut mesh = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/bad-morph.zmesh").unwrap())
        .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Short".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.1]]),
        )]),
    }];

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::MorphTargetAttributeLengthMismatch {
            target_index: 0,
            attribute: MESH_ATTRIBUTE_POSITION.to_string(),
            expected: 3,
            actual: 1,
        }
    );
}

#[test]
fn mesh_asset_to_morphed_model_primitive_applies_weighted_position_normal_tangent_and_color_deltas()
{
    let mut mesh = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/morphed.zmesh").unwrap())
        .unwrap();
    mesh.morph_targets = vec![
        MeshMorphTargetAsset {
            name: Some("Lift".to_string()),
            attributes: BTreeMap::from([
                (
                    MESH_ATTRIBUTE_POSITION.to_string(),
                    MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
                ),
                (
                    MESH_ATTRIBUTE_NORMAL.to_string(),
                    MeshAttributeValues::Float32x3(vec![[0.0, 1.0, 0.0]; 3]),
                ),
                (
                    MESH_ATTRIBUTE_TANGENT.to_string(),
                    MeshAttributeValues::Float32x3(vec![[0.0, 1.0, 0.0]; 3]),
                ),
                (
                    MESH_ATTRIBUTE_COLOR.to_string(),
                    MeshAttributeValues::Float32x4(vec![[0.2, -0.4, 0.1, -0.6]; 3]),
                ),
            ]),
        },
        MeshMorphTargetAsset {
            name: Some("Slide".to_string()),
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0]; 3]),
            )]),
        },
    ];

    let primitive = mesh.to_morphed_model_primitive(&[0.5, 1.0]).unwrap();

    assert!(Vec3::from_array(primitive.vertices[0].position)
        .abs_diff_eq(Vec3::new(1.0, 0.0, 0.5), 1.0e-6));
    assert!(Vec3::from_array(primitive.vertices[0].normal)
        .abs_diff_eq(Vec3::new(0.0, 0.5, 1.0).normalize(), 1.0e-6));
    assert!(Vec3::from_array([
        primitive.vertices[0].tangent[0],
        primitive.vertices[0].tangent[1],
        primitive.vertices[0].tangent[2],
    ])
    .abs_diff_eq(Vec3::new(1.0, 0.5, 0.0).normalize(), 1.0e-6));
    assert_eq!(primitive.vertices[0].tangent[3], 1.0);
    assert!(primitive.vertices[0]
        .color
        .iter()
        .zip([1.1, 0.8, 1.05, 0.7])
        .all(|(actual, expected)| (*actual - expected).abs() <= 1.0e-6));
    assert_eq!(primitive.indices, vec![0, 1, 2]);
    assert_eq!(primitive.virtual_geometry, mesh.virtual_geometry);
}

#[test]
fn mesh_asset_to_morphed_model_primitive_rejects_active_position_delta_with_wrong_format() {
    let mut mesh = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/bad-morph-format.zmesh").unwrap())
        .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("WrongFormat".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Uint16x4(vec![[0, 0, 0, 0]; 3]),
        )]),
    }];

    assert_eq!(
        mesh.to_morphed_model_primitive(&[1.0]).unwrap_err(),
        MeshValidationError::InvalidAttributeFormat {
            attribute: format!("morph_targets[0].{MESH_ATTRIBUTE_POSITION}"),
            expected: "float32x3",
        }
    );
}

#[test]
fn mesh_asset_to_morphed_model_primitive_rejects_active_color_delta_with_wrong_format() {
    let mut mesh = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/bad-morph-color.zmesh").unwrap())
        .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("WrongColorFormat".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_COLOR.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0]; 3]),
        )]),
    }];

    assert_eq!(
        mesh.to_morphed_model_primitive(&[1.0]).unwrap_err(),
        MeshValidationError::InvalidAttributeFormat {
            attribute: format!("morph_targets[0].{MESH_ATTRIBUTE_COLOR}"),
            expected: "float32x4",
        }
    );
}

#[test]
fn mesh_asset_reports_morph_target_attribute_summaries() {
    let mut mesh = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/morph-summary.zmesh").unwrap())
        .unwrap();
    mesh.morph_targets = vec![
        MeshMorphTargetAsset {
            name: Some("Smile".to_string()),
            attributes: BTreeMap::from([
                (
                    MESH_ATTRIBUTE_POSITION.to_string(),
                    MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.1]; 3]),
                ),
                (
                    "custom_delta".to_string(),
                    MeshAttributeValues::Uint32x4(vec![[1, 2, 3, 4]; 3]),
                ),
            ]),
        },
        MeshMorphTargetAsset {
            name: None,
            attributes: BTreeMap::from([(
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            )]),
        },
    ];

    assert_eq!(
        mesh.morph_target_attribute_summaries(),
        vec![
            MeshMorphTargetAttributeSummary {
                target_index: 0,
                target_name: Some("Smile".to_string()),
                attribute: MeshAttributeSummary {
                    name: "custom_delta".to_string(),
                    format: MeshAttributeFormat::Uint32x4,
                    len: 3,
                    is_builtin: false,
                },
            },
            MeshMorphTargetAttributeSummary {
                target_index: 0,
                target_name: Some("Smile".to_string()),
                attribute: MeshAttributeSummary {
                    name: MESH_ATTRIBUTE_POSITION.to_string(),
                    format: MeshAttributeFormat::Float32x3,
                    len: 3,
                    is_builtin: true,
                },
            },
            MeshMorphTargetAttributeSummary {
                target_index: 1,
                target_name: None,
                attribute: MeshAttributeSummary {
                    name: MESH_ATTRIBUTE_NORMAL.to_string(),
                    format: MeshAttributeFormat::Float32x3,
                    len: 3,
                    is_builtin: true,
                },
            },
        ]
    );
}
