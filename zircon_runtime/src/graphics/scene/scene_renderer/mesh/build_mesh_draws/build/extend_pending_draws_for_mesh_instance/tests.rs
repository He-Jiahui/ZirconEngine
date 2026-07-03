use std::collections::BTreeMap;

use super::super::pending_mesh_draw::PendingSkinnedGpuSource;
use super::{
    direct_skinned_gpu_source, morph_shape_signature, morphed_mesh_asset_primitive,
    skinned_gpu_source_candidate_available,
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

#[test]
fn direct_skinned_gpu_source_uses_prepared_mesh_when_morph_payload_is_available() {
    let Some(source_mesh) = test_gpu_mesh() else {
        return;
    };
    let uniform = SkinnedMeshJointPaletteUniform::from_matrices(&[])
        .expect("empty palette should fit the fixed skinned ABI");

    let source = direct_skinned_gpu_source(
        Some(&uniform),
        crate::core::resource::ResourceId::from_stable_label("mesh-gpu-morph"),
        source_mesh.clone(),
        test_primitive(),
        true,
        &[0.5],
    )
    .expect("shader-visible skinned source");

    match source {
        PendingSkinnedGpuSource::Prepared(selected) => {
            assert!(std::sync::Arc::ptr_eq(&selected, &source_mesh));
        }
        PendingSkinnedGpuSource::CpuMorphed { .. } => {
            panic!("morph payload availability should keep the original prepared source")
        }
    }
}

#[test]
fn direct_skinned_gpu_source_keeps_cpu_morphed_fallback_without_morph_payload() {
    let Some(source_mesh) = test_gpu_mesh() else {
        return;
    };
    let uniform = SkinnedMeshJointPaletteUniform::from_matrices(&[])
        .expect("empty palette should fit the fixed skinned ABI");

    let source = direct_skinned_gpu_source(
        Some(&uniform),
        crate::core::resource::ResourceId::from_stable_label("mesh-cpu-morph"),
        source_mesh,
        test_primitive(),
        false,
        &[0.5],
    )
    .expect("shader-visible skinned source");

    assert!(matches!(source, PendingSkinnedGpuSource::CpuMorphed { .. }));
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

fn test_primitive() -> crate::asset::ModelPrimitiveAsset {
    crate::asset::ModelPrimitiveAsset {
        vertices: Vec::new(),
        indices: Vec::new(),
        mesh: None,
        virtual_geometry: None,
    }
}

fn test_gpu_mesh() -> Option<std::sync::Arc<crate::graphics::scene::resources::GpuMeshResource>> {
    let backend = crate::graphics::backend::RenderBackend::new_offscreen()
        .inspect_err(|error| eprintln!("skipping direct skinned GPU source test: {error:?}"))
        .ok()?;
    Some(std::sync::Arc::new(
        crate::graphics::scene::resources::GpuMeshResource::from_asset(
            &backend.device,
            crate::asset::ModelPrimitiveAsset {
                vertices: vec![
                    crate::asset::MeshVertex::new(
                        Vec3::ZERO,
                        Vec3::Z,
                        crate::core::math::Vec2::ZERO,
                    ),
                    crate::asset::MeshVertex::new(Vec3::X, Vec3::Z, crate::core::math::Vec2::ZERO),
                    crate::asset::MeshVertex::new(Vec3::Y, Vec3::Z, crate::core::math::Vec2::ZERO),
                ],
                indices: vec![0, 1, 2],
                mesh: None,
                virtual_geometry: None,
            },
        ),
    ))
}
