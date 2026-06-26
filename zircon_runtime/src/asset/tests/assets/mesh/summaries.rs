use super::*;

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
