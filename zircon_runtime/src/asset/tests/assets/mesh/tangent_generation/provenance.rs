use super::*;

#[test]
fn combined_generation_rolls_back_splits_when_a_later_morph_fails() {
    let mut mesh = mikktspace_corner_split_mesh();
    let source_count = mesh.vertex_count().unwrap();
    let mut base_only = mesh.clone();
    assert!(base_only.try_generate_missing_tangents().unwrap());
    assert!(
        base_only.vertex_count().unwrap() > source_count,
        "fixture must split"
    );
    mesh.morph_targets.push(MeshMorphTargetAsset {
        name: Some("invalid second normal stream".into()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_NORMAL.to_string(),
            MeshAttributeValues::Float32x2(vec![[0.0; 2]; source_count]),
        )]),
    });
    let before = mesh.clone();
    let error = mesh
        .try_generate_missing_tangents_and_morph_frames_for_uv(MESH_ATTRIBUTE_UV0, false)
        .unwrap_err();
    assert_eq!(
        error,
        MeshValidationError::InvalidAttributeFormat {
            attribute: format!("morph_targets[1].{MESH_ATTRIBUTE_NORMAL}"),
            expected: "float32x3",
        }
    );
    assert_eq!(
        mesh, before,
        "indices, split attributes and earlier target must roll back"
    );
}

#[test]
fn authored_basis_equal_to_the_fallback_is_not_an_undefined_base() {
    let mut mesh = MeshAsset::new(
        AssetUri::parse("res://meshes/authored-collapsed.zmesh").unwrap(),
        RenderMeshTopology::TriangleList,
        BTreeMap::from([
            (
                MESH_ATTRIBUTE_POSITION.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0; 3]; 3]),
            ),
            (
                MESH_ATTRIBUTE_NORMAL.to_string(),
                MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
            ),
            (
                MESH_ATTRIBUTE_UV0.to_string(),
                MeshAttributeValues::Float32x2(vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]]),
            ),
            (
                MESH_ATTRIBUTE_TANGENT.to_string(),
                MeshAttributeValues::Float32x4(vec![[1.0, 0.0, 0.0, 1.0]; 3]),
            ),
        ]),
        None,
    )
    .unwrap();
    mesh.morph_targets.push(MeshMorphTargetAsset {
        name: Some("expand".into()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        )]),
    });
    let before = mesh.clone();
    assert!(!mesh
        .try_generate_missing_tangents_and_morph_frames_for_uv(MESH_ATTRIBUTE_UV0, false)
        .unwrap());
    assert_eq!(
        mesh.try_rebuild_morph_tangent_frames_for_uv(false, Some(MESH_ATTRIBUTE_UV0)),
        Err(MeshValidationError::MorphTangentHandednessMismatch {
            target_index: 0,
            vertex_index: 0
        })
    );
    assert_eq!(mesh, before);
}
