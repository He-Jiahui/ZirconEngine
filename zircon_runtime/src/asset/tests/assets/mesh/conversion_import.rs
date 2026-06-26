use super::*;

#[test]
fn model_primitive_converts_to_mesh_asset_with_builtin_attributes() {
    let primitive = ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::ZERO, Vec3::Z, Vec2::ZERO)
                .with_uv1(Vec2::new(0.5, 0.25))
                .with_tangent([1.0, 0.0, 0.0, -1.0])
                .with_color([0.25, 0.5, 0.75, 1.0])
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
    assert_eq!(mesh.attributes.len(), 8);
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_POSITION));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_NORMAL));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_UV0));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_UV1));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_TANGENT));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_COLOR));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_JOINT_INDEX));
    assert!(mesh.attributes.contains_key(MESH_ATTRIBUTE_JOINT_WEIGHT));
    assert_eq!(
        mesh.attributes[MESH_ATTRIBUTE_UV1],
        MeshAttributeValues::Float32x2(vec![[0.5, 0.25], [0.0, 0.0], [0.0, 0.0]])
    );
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
