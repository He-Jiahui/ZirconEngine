use crate::asset::ModelPrimitiveAsset;
use crate::core::math::{Vec2, Vec3};
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshDrawGeometrySource;

use super::super::pending_mesh_draw::{PendingMeshGeometry, PendingSkinnedGpuSource};

#[test]
fn direct_cpu_morphed_geometry_stays_static_shader_fallback() {
    let geometry = PendingMeshGeometry::CpuMorphed(test_primitive());

    assert_eq!(
        super::pending_mesh_geometry_source(&geometry, None, false, false),
        MeshDrawGeometrySource::DynamicCpuMorphedSource
    );
    assert_eq!(
        super::pending_mesh_geometry_source(&geometry, None, true, true),
        MeshDrawGeometrySource::DynamicCpuMorphedSource
    );
}

#[test]
fn gpu_morphed_geometry_uses_morphed_shader_source() {
    let Some(mesh) = test_gpu_mesh() else {
        return;
    };
    let geometry = PendingMeshGeometry::GpuMorphed(mesh);

    assert_eq!(
        super::pending_mesh_geometry_source(&geometry, None, false, true),
        MeshDrawGeometrySource::DynamicGpuMorphedSource
    );
}

#[test]
fn prepared_skinned_gpu_source_with_morph_slot_uses_skinned_morphed_shader_source() {
    let Some(mesh) = test_gpu_mesh() else {
        return;
    };
    let geometry = PendingMeshGeometry::CpuMorphed(test_primitive());
    let source = PendingSkinnedGpuSource::Prepared(mesh);

    assert_eq!(
        super::pending_mesh_geometry_source(&geometry, Some(&source), true, true),
        MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource
    );
}

#[test]
fn cpu_morphed_skinned_gpu_source_keeps_skinned_shader_fallback() {
    let Some(mesh) = test_gpu_mesh() else {
        return;
    };
    let geometry = PendingMeshGeometry::CpuMorphed(test_primitive());
    let source = PendingSkinnedGpuSource::CpuMorphed {
        primitive: test_primitive(),
        morph_shape_signature: 7,
    };
    drop(mesh);

    assert_eq!(
        super::pending_mesh_geometry_source(&geometry, Some(&source), true, true),
        MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource
    );
}

fn test_primitive() -> ModelPrimitiveAsset {
    ModelPrimitiveAsset {
        vertices: Vec::new(),
        indices: Vec::new(),
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}

fn test_gpu_mesh() -> Option<std::sync::Arc<GpuMeshResource>> {
    let backend = crate::graphics::backend::RenderBackend::new_offscreen()
        .inspect_err(|error| {
            eprintln!("skipping geometry source selection GPU mesh test: {error:?}")
        })
        .ok()?;
    Some(std::sync::Arc::new(GpuMeshResource::from_asset(
        &backend.device,
        ModelPrimitiveAsset {
            vertices: vec![
                crate::asset::MeshVertex::new(Vec3::ZERO, Vec3::Z, Vec2::ZERO),
                crate::asset::MeshVertex::new(Vec3::X, Vec3::Z, Vec2::ZERO),
                crate::asset::MeshVertex::new(Vec3::Y, Vec3::Z, Vec2::ZERO),
            ],
            indices: vec![0, 1, 2],
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        },
    )))
}
