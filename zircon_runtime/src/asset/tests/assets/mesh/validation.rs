use super::*;

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
        mesh_sdf: None,
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
        mesh_sdf: None,
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
            MESH_ATTRIBUTE_UV1,
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
            mesh_sdf: None,
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
        mesh_sdf: None,
        virtual_geometry: None,
    };

    assert_eq!(mesh.validate(), Ok(()));
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
        mesh_sdf: None,
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
            mesh_sdf: None,
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
