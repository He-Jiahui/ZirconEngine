use std::collections::BTreeMap;
use std::fs;

use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{
    AssetImporter, AssetUri, ImportedAsset, MeshAsset, MeshAssetManagementRecord,
    MeshAssetManagementRecordSet, MeshAttributeFormat, MeshAttributeSummary, MeshAttributeValues,
    MeshIndexFormat, MeshIndices, MeshMorphTargetAsset, MeshMorphTargetAttributeSummary,
    MeshSkinAsset, MeshValidationError, MeshVertex, ModelPrimitiveAsset, VirtualGeometryAsset,
    ZMeshDocument, MESH_ATTRIBUTE_COLOR, MESH_ATTRIBUTE_JOINT_INDEX, MESH_ATTRIBUTE_JOINT_WEIGHT,
    MESH_ATTRIBUTE_NORMAL, MESH_ATTRIBUTE_POSITION, MESH_ATTRIBUTE_TANGENT, MESH_ATTRIBUTE_UV0,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::math::{Vec2, Vec3};
use crate::core::resource::ResourceId;

mod normal_generation;
mod tangent_generation;

#[test]
fn zmesh_document_roundtrip_preserves_mesh_payload() {
    let document = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]));

    let encoded = document.to_toml_string().unwrap();
    let decoded = ZMeshDocument::from_toml_str(&encoded).unwrap();
    let mesh = decoded
        .into_mesh_asset(AssetUri::parse("res://meshes/triangle.zmesh").unwrap())
        .unwrap();

    assert_eq!(mesh.topology, RenderMeshTopology::TriangleList);
    assert_eq!(mesh.vertex_count().unwrap(), 3);
    assert_eq!(mesh.index_count(), 3);
    assert!(mesh.asset_usage.main_world);
    assert!(mesh.asset_usage.render_world);
    assert_eq!(
        mesh.virtual_geometry
            .as_ref()
            .unwrap()
            .debug
            .source_hint
            .as_deref(),
        Some("zmesh-roundtrip")
    );
    assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
}

#[test]
fn zmesh_document_roundtrip_preserves_morph_targets_and_skin_inverse_bindposes() {
    let mut document = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]));
    document.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Smile".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_TANGENT.to_string(),
            MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 1.0]; 3]),
        )]),
    }];
    document.skin = Some(MeshSkinAsset {
        inverse_bind_matrices: vec![identity_matrix()],
    });

    let encoded = document.to_toml_string().unwrap();
    let decoded = ZMeshDocument::from_toml_str(&encoded).unwrap();
    let mesh = decoded
        .into_mesh_asset(AssetUri::parse("res://meshes/skinned.zmesh").unwrap())
        .unwrap();

    assert_eq!(mesh.morph_targets.len(), 1);
    assert_eq!(mesh.morph_targets[0].name.as_deref(), Some("Smile"));
    assert_eq!(
        mesh.morph_targets[0]
            .attributes
            .get(MESH_ATTRIBUTE_TANGENT)
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        mesh.skin.as_ref().unwrap().inverse_bind_matrices,
        vec![identity_matrix()]
    );
}

#[test]
fn mesh_asset_rejects_missing_position_attribute() {
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/no-position.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::new(),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::MissingPositionAttribute
    );
}

#[test]
fn mesh_asset_rejects_attribute_length_mismatch() {
    let mut attributes = triangle_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_NORMAL.to_string(),
        MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/bad-normal.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U32(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::AttributeLengthMismatch {
            attribute: MESH_ATTRIBUTE_NORMAL.to_string(),
            expected: 3,
            actual: 1,
        }
    );
}

#[test]
fn mesh_asset_rejects_builtin_attribute_format_mismatch() {
    let invalid_attributes = vec![
        (
            MESH_ATTRIBUTE_NORMAL,
            MeshAttributeValues::Float32x2(vec![[0.0, 1.0]; 3]),
            "float32x3",
        ),
        (
            MESH_ATTRIBUTE_TANGENT,
            MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0]; 3]),
            "float32x4",
        ),
        (
            MESH_ATTRIBUTE_UV0,
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0]; 3]),
            "float32x2",
        ),
        (
            MESH_ATTRIBUTE_COLOR,
            MeshAttributeValues::Float32x3(vec![[1.0, 1.0, 1.0]; 3]),
            "float32x4",
        ),
        (
            MESH_ATTRIBUTE_JOINT_INDEX,
            MeshAttributeValues::Uint32x4(vec![[0, 0, 0, 0]; 3]),
            "uint16x4",
        ),
        (
            MESH_ATTRIBUTE_JOINT_WEIGHT,
            MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0]; 3]),
            "float32x4",
        ),
    ];

    for (attribute, values, expected) in invalid_attributes {
        let mut attributes = triangle_attributes();
        attributes.insert(attribute.to_string(), values);
        let mesh = MeshAsset {
            uri: AssetUri::parse("res://meshes/bad-builtin-format.zmesh").unwrap(),
            topology: RenderMeshTopology::TriangleList,
            attributes,
            indices: Some(MeshIndices::U32(vec![0, 1, 2])),
            asset_usage: Default::default(),
            morph_targets: Vec::new(),
            skin: None,
            virtual_geometry: None,
        };

        assert_eq!(
            mesh.validate().unwrap_err(),
            MeshValidationError::InvalidAttributeFormat {
                attribute: attribute.to_string(),
                expected,
            }
        );
    }
}

#[test]
fn mesh_asset_allows_custom_attribute_formats_when_lengths_match() {
    let mut attributes = triangle_attributes();
    attributes.insert(
        "temperature".to_string(),
        MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [0.5, 0.0], [1.0, 0.0]]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/custom-attribute.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U32(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(mesh.validate(), Ok(()));
}

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
fn mesh_asset_to_morphed_model_primitive_applies_weighted_position_and_normal_deltas() {
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

    assert!(
        Vec3::from_array(primitive.vertices[0].position)
            .abs_diff_eq(Vec3::new(1.0, 0.0, 0.5), 1.0e-6)
    );
    assert!(
        Vec3::from_array(primitive.vertices[0].normal)
            .abs_diff_eq(Vec3::new(0.0, 1.0, 1.0).normalize(), 1.0e-6)
    );
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
fn mesh_asset_rejects_out_of_range_indices() {
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/bad-index.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: triangle_attributes(),
        indices: Some(MeshIndices::U32(vec![0, 1, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        mesh.validate().unwrap_err(),
        MeshValidationError::IndexOutOfRange {
            max_index: 3,
            vertex_count: 3,
        }
    );
}

#[test]
fn mesh_asset_rejects_incomplete_list_topology_elements() {
    let invalid_cases = vec![
        (RenderMeshTopology::TriangleList, None, 3, 4),
        (
            RenderMeshTopology::TriangleList,
            Some(MeshIndices::U32(vec![0, 1, 2, 0])),
            3,
            4,
        ),
        (RenderMeshTopology::LineList, None, 2, 3),
        (
            RenderMeshTopology::LineList,
            Some(MeshIndices::U16(vec![0, 1, 2])),
            2,
            3,
        ),
    ];

    for (topology, indices, required_multiple, actual_elements) in invalid_cases {
        let mut attributes = triangle_attributes();
        if indices.is_none() {
            let positions = (0..actual_elements)
                .map(|index| [index as f32, 0.0, 0.0])
                .collect::<Vec<_>>();
            attributes.insert(
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(positions),
            );
            attributes.insert(
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; actual_elements]),
            );
            attributes.insert(
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0]; actual_elements]),
            );
        }
        let mesh = MeshAsset {
            uri: AssetUri::parse("res://meshes/bad-topology.zmesh").unwrap(),
            topology,
            attributes,
            indices,
            asset_usage: Default::default(),
            morph_targets: Vec::new(),
            skin: None,
            virtual_geometry: None,
        };

        assert_eq!(
            mesh.validate().unwrap_err(),
            MeshValidationError::IncompleteTopologyElement {
                topology,
                required_multiple,
                actual_elements,
            }
        );
    }
}

#[test]
fn mesh_asset_reports_index_format_without_expanding_indices() {
    let indexed_u16 = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/indexed-u16.zmesh").unwrap())
        .unwrap();
    let indexed_u32 = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/indexed-u32.zmesh").unwrap())
        .unwrap();
    let unindexed = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]));
    let unindexed = ZMeshDocument {
        indices: None,
        ..unindexed
    }
    .into_mesh_asset(AssetUri::parse("res://meshes/unindexed.zmesh").unwrap())
    .unwrap();

    assert_eq!(indexed_u16.index_format(), Some(MeshIndexFormat::U16));
    assert_eq!(indexed_u32.index_format(), Some(MeshIndexFormat::U32));
    assert_eq!(unindexed.index_format(), None);
}

#[test]
fn mesh_asset_reports_draw_element_and_primitive_counts_without_descriptor() {
    let indexed_triangle = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/indexed-triangle.zmesh").unwrap())
        .unwrap();
    let unindexed_triangle = ZMeshDocument {
        indices: None,
        ..sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
    }
    .into_mesh_asset(AssetUri::parse("res://meshes/unindexed-triangle.zmesh").unwrap())
    .unwrap();
    let indexed_lines = MeshAsset {
        uri: AssetUri::parse("res://meshes/lines.zmesh").unwrap(),
        topology: RenderMeshTopology::LineList,
        attributes: quad_attributes(),
        indices: Some(MeshIndices::U16(vec![0, 1, 2, 3])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let triangle_strip = MeshAsset {
        uri: AssetUri::parse("res://meshes/strip.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleStrip,
        attributes: quad_attributes(),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let invalid_triangle = MeshAsset {
        uri: AssetUri::parse("res://meshes/incomplete.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: quad_attributes(),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(indexed_triangle.draw_element_count().unwrap(), 3);
    assert_eq!(indexed_triangle.render_primitive_count().unwrap(), 1);
    assert_eq!(
        indexed_triangle.render_primitive_count().unwrap(),
        indexed_triangle.render_mesh_descriptor().primitive_count
    );
    assert_eq!(unindexed_triangle.draw_element_count().unwrap(), 3);
    assert_eq!(unindexed_triangle.render_primitive_count().unwrap(), 1);
    assert_eq!(indexed_lines.draw_element_count().unwrap(), 4);
    assert_eq!(indexed_lines.render_primitive_count().unwrap(), 2);
    assert_eq!(
        indexed_lines.render_primitive_count().unwrap(),
        indexed_lines.render_mesh_descriptor().primitive_count
    );
    assert_eq!(triangle_strip.draw_element_count().unwrap(), 4);
    assert_eq!(triangle_strip.render_primitive_count().unwrap(), 2);
    assert_eq!(
        triangle_strip.render_primitive_count().unwrap(),
        triangle_strip.render_mesh_descriptor().primitive_count
    );
    assert_eq!(invalid_triangle.draw_element_count().unwrap(), 4);
    assert_eq!(
        invalid_triangle.render_primitive_count().unwrap_err(),
        MeshValidationError::IncompleteTopologyElement {
            topology: RenderMeshTopology::TriangleList,
            required_multiple: 3,
            actual_elements: 4,
        }
    );
}

#[test]
fn mesh_asset_reports_attribute_summaries_without_value_inspection() {
    let mut attributes = triangle_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_COLOR.to_string(),
        MeshAttributeValues::Float32x4(vec![[1.0, 1.0, 1.0, 1.0]; 3]),
    );
    attributes.insert(
        "temperature".to_string(),
        MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [0.5, 0.0], [1.0, 0.0]]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/attribute-summary.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U16(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(MeshAttributeFormat::Float32x2.as_str(), "float32x2");
    assert_eq!(MeshAttributeFormat::Uint32x4.as_str(), "uint32x4");
    assert_eq!(
        mesh.attribute_summaries(),
        vec![
            MeshAttributeSummary {
                name: MESH_ATTRIBUTE_COLOR.to_string(),
                format: MeshAttributeFormat::Float32x4,
                len: 3,
                is_builtin: true,
            },
            MeshAttributeSummary {
                name: MESH_ATTRIBUTE_NORMAL.to_string(),
                format: MeshAttributeFormat::Float32x3,
                len: 3,
                is_builtin: true,
            },
            MeshAttributeSummary {
                name: MESH_ATTRIBUTE_POSITION.to_string(),
                format: MeshAttributeFormat::Float32x3,
                len: 3,
                is_builtin: true,
            },
            MeshAttributeSummary {
                name: "temperature".to_string(),
                format: MeshAttributeFormat::Float32x2,
                len: 3,
                is_builtin: false,
            },
            MeshAttributeSummary {
                name: MESH_ATTRIBUTE_UV0.to_string(),
                format: MeshAttributeFormat::Float32x2,
                len: 3,
                is_builtin: true,
            },
        ]
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

#[test]
fn mesh_asset_overview_reports_editor_ready_mesh_summary() {
    let mut mesh = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/overview.zmesh").unwrap())
        .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Smile".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.1]; 3]),
        )]),
    }];
    mesh.skin = Some(MeshSkinAsset {
        inverse_bind_matrices: vec![identity_matrix(), identity_matrix()],
    });

    let overview = mesh.overview().unwrap();

    assert_eq!(overview.uri, mesh.uri);
    assert_eq!(overview.topology, RenderMeshTopology::TriangleList);
    assert_eq!(overview.vertex_count, 3);
    assert_eq!(overview.index_count, 3);
    assert_eq!(overview.index_format, Some(MeshIndexFormat::U16));
    assert_eq!(overview.draw_element_count, 3);
    assert_eq!(overview.render_primitive_count, 1);
    assert_eq!(overview.attribute_count, 3);
    assert_eq!(overview.attributes, mesh.attribute_summaries());
    assert_eq!(overview.morph_target_count, 1);
    assert_eq!(overview.morph_target_attribute_count, 1);
    assert_eq!(
        overview.morph_target_attributes,
        mesh.morph_target_attribute_summaries()
    );
    assert!(overview.has_skin);
    assert_eq!(overview.inverse_bind_matrix_count, 2);
    assert!(overview.has_virtual_geometry_payload);
    assert_eq!(overview.asset_usage, mesh.asset_usage);
    assert_eq!(overview.bounds, mesh.bounds().unwrap());
}

#[test]
fn mesh_asset_management_record_wraps_id_and_strict_overview() {
    let mesh = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/management-record.zmesh").unwrap())
        .unwrap();
    let mesh_id = ResourceId::from_locator(&mesh.uri);
    let overview = mesh.overview().unwrap();

    let record: MeshAssetManagementRecord = mesh.management_record(mesh_id).unwrap();

    assert_eq!(record.mesh_id, mesh_id);
    assert_eq!(record.overview, overview);
}

#[test]
fn mesh_asset_management_record_set_summarizes_valid_and_invalid_rows() {
    let valid_mesh = sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/record-set-valid.zmesh").unwrap())
        .unwrap();
    let invalid_mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/record-set-invalid.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes: BTreeMap::new(),
        indices: None,
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };
    let valid_id = ResourceId::from_locator(&valid_mesh.uri);
    let invalid_id = ResourceId::from_locator(&invalid_mesh.uri);

    let record_set = MeshAssetManagementRecordSet::from_results(vec![
        (invalid_id, invalid_mesh.management_record(invalid_id)),
        (valid_id, valid_mesh.management_record(valid_id)),
    ]);

    assert_eq!(record_set.records.len(), 1);
    assert_eq!(record_set.records[0].mesh_id, valid_id);
    assert_eq!(record_set.failures.len(), 1);
    assert_eq!(record_set.failures[0].mesh_id, invalid_id);
    assert!(record_set.failures[0]
        .diagnostic
        .contains("missing required position attribute"));
    let summary = &record_set.summary;
    assert_eq!(summary.mesh_count, 2);
    assert_eq!(summary.valid_mesh_count, 1);
    assert_eq!(summary.invalid_mesh_count, 1);
    assert_eq!(summary.vertex_count, 3);
    assert_eq!(summary.index_count, 3);
    assert_eq!(summary.draw_element_count, 3);
    assert_eq!(summary.render_primitive_count, 1);
    assert_eq!(summary.attribute_count, 3);
    assert_eq!(summary.morph_target_count, 0);
    assert_eq!(summary.morph_target_attribute_count, 0);
    assert_eq!(summary.skinned_mesh_count, 0);
    assert_eq!(summary.inverse_bind_matrix_count, 0);
    assert_eq!(summary.virtual_geometry_mesh_count, 1);
}

#[test]
fn model_primitive_converts_to_mesh_asset_with_builtin_attributes() {
    let primitive = ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::ZERO, Vec3::Z, Vec2::ZERO)
                .with_skinning([0, 1, 0, 0], [0.75, 0.25, 0.0, 0.0]),
            MeshVertex::new(Vec3::X, Vec3::Z, Vec2::X),
            MeshVertex::new(Vec3::Y, Vec3::Z, Vec2::Y),
        ],
        indices: vec![0, 1, 2],
        mesh: None,
        virtual_geometry: Some(sample_virtual_geometry()),
    };

    let mesh = MeshAsset::from_model_primitive(
        AssetUri::parse("res://models/triangle.obj#Mesh0/Primitive0").unwrap(),
        &primitive,
    );

    assert_eq!(mesh.vertex_count().unwrap(), 3);
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_POSITION));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_NORMAL));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_UV0));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_JOINT_INDEX));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_JOINT_WEIGHT));
    assert_eq!(mesh.to_model_primitive().unwrap(), primitive);
}

#[test]
fn mesh_render_descriptor_uses_bounds_topology_and_indices() {
    let mesh = sample_zmesh_document(MeshIndices::U32(vec![0, 1, 2]))
        .into_mesh_asset(AssetUri::parse("res://meshes/triangle.zmesh").unwrap())
        .unwrap();

    let descriptor = mesh.render_mesh_descriptor();

    assert_eq!(descriptor.topology, RenderMeshTopology::TriangleList);
    assert_eq!(descriptor.vertex_count, 3);
    assert_eq!(descriptor.index_count, 3);
    assert_eq!(descriptor.primitive_count, 1);
    assert_eq!(descriptor.bounds.min, [0.0, 0.0, 0.0]);
    assert_eq!(descriptor.bounds.max, [1.0, 1.0, 0.0]);
    assert!(descriptor.suitable_for_2d);
    assert!(descriptor.has_virtual_geometry_payload);
}

#[test]
fn mesh_asset_bounds_can_be_read_without_render_descriptor() {
    let mut attributes = triangle_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_POSITION.to_string(),
        MeshAttributeValues::Float32x3(vec![[-2.0, 1.0, -1.0], [4.0, -3.0, 2.0], [1.0, 5.0, 3.0]]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/bounds.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U32(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    let bounds = mesh.bounds().unwrap();

    assert_eq!(bounds.min, [-2.0, -3.0, -1.0]);
    assert_eq!(bounds.max, [4.0, 5.0, 3.0]);
    assert_eq!(bounds.center, [1.0, 1.0, 1.0]);
    assert!((bounds.radius - 5.3851647).abs() < 0.000001);
    assert_eq!(mesh.render_mesh_descriptor().bounds, bounds);
}

#[test]
fn mesh_asset_try_render_descriptor_reports_validation_errors() {
    let mut attributes = triangle_attributes();
    attributes.insert(
        MESH_ATTRIBUTE_UV0.to_string(),
        MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 0.0]; 3]),
    );
    let mesh = MeshAsset {
        uri: AssetUri::parse("res://meshes/bad-descriptor.zmesh").unwrap(),
        topology: RenderMeshTopology::TriangleList,
        attributes,
        indices: Some(MeshIndices::U32(vec![0, 1, 2])),
        asset_usage: Default::default(),
        morph_targets: Vec::new(),
        skin: None,
        virtual_geometry: None,
    };

    assert_eq!(
        mesh.try_render_mesh_descriptor().unwrap_err(),
        MeshValidationError::InvalidAttributeFormat {
            attribute: MESH_ATTRIBUTE_UV0.to_string(),
            expected: "float32x2",
        }
    );
    assert_eq!(mesh.render_mesh_descriptor().vertex_count, 3);
}

#[test]
fn default_importer_routes_zmesh_to_mesh_asset() {
    let root = unique_temp_project_root("zmesh_import");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("triangle.zmesh");
    fs::write(
        &path,
        sample_zmesh_document(MeshIndices::U16(vec![0, 1, 2]))
            .to_toml_string()
            .unwrap(),
    )
    .unwrap();

    let imported = AssetImporter::default()
        .import_from_source(
            &path,
            &AssetUri::parse("res://meshes/triangle.zmesh").unwrap(),
        )
        .unwrap();

    match imported {
        ImportedAsset::Mesh(mesh) => {
            assert_eq!(mesh.vertex_count().unwrap(), 3);
            assert_eq!(mesh.to_model_primitive().unwrap().indices, vec![0, 1, 2]);
        }
        other => panic!("unexpected imported asset: {other:?}"),
    }

    let _ = fs::remove_dir_all(root);
}

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
