use std::collections::BTreeMap;

use super::{
    morph_shape_signature, morphed_mesh_asset_primitive, skinned_gpu_source_candidate_available,
};
use crate::asset::{
    AssetUri, MeshAsset, MeshAttributeValues, MeshIndices, MeshMorphTargetAsset,
    MESH_ATTRIBUTE_POSITION,
};
use crate::core::framework::render::RenderMeshTopology;
use crate::core::math::Vec3;
use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteUniform;

#[test]
fn morphed_mesh_asset_primitive_ignores_zero_weights_for_static_direct_mesh_fallback() {
    let mesh = morph_test_mesh();

    assert!(morphed_mesh_asset_primitive(&mesh, &[0.0]).is_none());
}

#[test]
fn morphed_mesh_asset_primitive_applies_nonzero_weights_for_dynamic_direct_mesh() {
    let mesh = morph_test_mesh();

    let primitive = morphed_mesh_asset_primitive(&mesh, &[0.5]).expect("morphed primitive");

    assert!(Vec3::from_array(primitive.vertices[0].position)
        .abs_diff_eq(Vec3::new(1.0, 0.0, 0.5), 1.0e-6));
    assert_eq!(primitive.indices, vec![0, 1, 2]);
}

#[test]
fn morph_shape_signature_tracks_mesh_and_weights() {
    let mesh_a = crate::core::resource::ResourceId::from_stable_label("mesh-a");
    let mesh_b = crate::core::resource::ResourceId::from_stable_label("mesh-b");
    let first = morph_shape_signature(mesh_a, &[0.25, 0.0]);

    assert_eq!(first, morph_shape_signature(mesh_a, &[0.25, 0.0]));
    assert_ne!(first, morph_shape_signature(mesh_a, &[0.5, 0.0]));
    assert_ne!(first, morph_shape_signature(mesh_b, &[0.25, 0.0]));
}

#[test]
fn skinned_gpu_source_candidate_requires_palette() {
    let uniform = SkinnedMeshJointPaletteUniform::from_matrices(&[])
        .expect("empty palette should fit the fixed skinned ABI");

    assert!(skinned_gpu_source_candidate_available(Some(&uniform)));
    assert!(
        !skinned_gpu_source_candidate_available(None),
        "a source mesh is not enough without a shader-visible palette"
    );
}

fn morph_test_mesh() -> MeshAsset {
    let mut mesh = MeshAsset::new(
        AssetUri::parse("res://meshes/direct-morph.zmesh").unwrap(),
        RenderMeshTopology::TriangleList,
        BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        )]),
        Some(MeshIndices::U32(vec![0, 1, 2])),
    )
    .unwrap();
    mesh.morph_targets = vec![MeshMorphTargetAsset {
        name: Some("Lift".to_string()),
        attributes: BTreeMap::from([(
            MESH_ATTRIBUTE_POSITION.to_string(),
            MeshAttributeValues::Float32x3(vec![[0.0, 0.0, 1.0]; 3]),
        )]),
    }];
    mesh
}
