use super::*;

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
    let primitive = mesh.to_model_primitive().unwrap();
    assert_eq!(primitive.indices, vec![0, 1, 2]);
    assert_eq!(primitive.vertices[0].tangent, [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(primitive.vertices[0].color, [1.0, 1.0, 1.0, 1.0]);
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
