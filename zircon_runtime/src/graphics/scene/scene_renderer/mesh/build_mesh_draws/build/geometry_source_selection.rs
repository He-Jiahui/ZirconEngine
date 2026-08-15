use std::sync::Arc;

use crate::asset::ModelPrimitiveAsset;
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};

use super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry, PendingSkinnedGpuSource};

#[derive(Clone)]
pub(super) enum PendingMeshSourceSelection {
    Prepared(Arc<GpuMeshResource>),
    Dynamic {
        primitive: ModelPrimitiveAsset,
        geometry_source: MeshDrawGeometrySource,
    },
    GpuMorphed {
        mesh: Arc<GpuMeshResource>,
    },
    SkinnedGpu {
        mesh: Arc<GpuMeshResource>,
        geometry_source: MeshDrawGeometrySource,
        source_uses_cpu_morphed_source: bool,
    },
}

impl PendingMeshSourceSelection {
    pub(super) fn geometry_source(&self) -> MeshDrawGeometrySource {
        match self {
            Self::Prepared(_) => MeshDrawGeometrySource::Prepared,
            Self::Dynamic {
                geometry_source, ..
            } => *geometry_source,
            Self::GpuMorphed { .. } => MeshDrawGeometrySource::DynamicGpuMorphedSource,
            Self::SkinnedGpu {
                geometry_source, ..
            } => *geometry_source,
        }
    }
}

pub(super) fn pending_mesh_source_selection(
    pending_draw: &PendingMeshDraw,
    skinned_gpu_skinning_enabled: bool,
) -> PendingMeshSourceSelection {
    if skinned_gpu_skinning_enabled {
        if let Some(source) = pending_draw.skinned_gpu_source.as_ref() {
            let mesh = pending_draw
                .resolved_skinned_gpu_source
                .as_ref()
                .expect("enabled skinned GPU source should be resolved")
                .clone();
            return PendingMeshSourceSelection::SkinnedGpu {
                mesh,
                geometry_source: skinned_gpu_source_geometry_source(
                    source,
                    pending_draw.morph_payload_slot.is_some(),
                ),
                source_uses_cpu_morphed_source: source.uses_cpu_morphed_source(),
            };
        }
    }

    match &pending_draw.mesh {
        PendingMeshGeometry::Prepared(mesh) => PendingMeshSourceSelection::Prepared(mesh.clone()),
        PendingMeshGeometry::GpuMorphed(mesh) => {
            PendingMeshSourceSelection::GpuMorphed { mesh: mesh.clone() }
        }
        PendingMeshGeometry::Dynamic(primitive) => PendingMeshSourceSelection::Dynamic {
            primitive: primitive.clone(),
            geometry_source: MeshDrawGeometrySource::Dynamic,
        },
        PendingMeshGeometry::CpuMorphed(primitive) => PendingMeshSourceSelection::Dynamic {
            primitive: primitive.clone(),
            geometry_source: MeshDrawGeometrySource::DynamicCpuMorphedSource,
        },
    }
}

pub(super) fn pending_mesh_draw_geometry_source(
    pending_draw: &PendingMeshDraw,
    skinned_gpu_skinning_enabled: bool,
) -> MeshDrawGeometrySource {
    pending_mesh_geometry_source(
        &pending_draw.mesh,
        pending_draw.skinned_gpu_source.as_ref(),
        skinned_gpu_skinning_enabled,
        pending_draw.morph_payload_slot.is_some(),
    )
}

pub(super) fn pending_mesh_geometry_source(
    mesh: &PendingMeshGeometry,
    skinned_gpu_source: Option<&PendingSkinnedGpuSource>,
    skinned_gpu_skinning_enabled: bool,
    has_morph_payload_slot: bool,
) -> MeshDrawGeometrySource {
    match mesh {
        PendingMeshGeometry::Prepared(_) if skinned_gpu_skinning_enabled => skinned_gpu_source
            .map(|source| skinned_gpu_source_geometry_source(source, has_morph_payload_slot))
            .unwrap_or(MeshDrawGeometrySource::Prepared),
        PendingMeshGeometry::Prepared(_) => MeshDrawGeometrySource::Prepared,
        PendingMeshGeometry::Dynamic(_) if skinned_gpu_skinning_enabled => skinned_gpu_source
            .map(|source| skinned_gpu_source_geometry_source(source, has_morph_payload_slot))
            .unwrap_or(MeshDrawGeometrySource::Dynamic),
        PendingMeshGeometry::Dynamic(_) => MeshDrawGeometrySource::Dynamic,
        PendingMeshGeometry::CpuMorphed(_) if skinned_gpu_skinning_enabled => skinned_gpu_source
            .map(|source| skinned_gpu_source_geometry_source(source, has_morph_payload_slot))
            .unwrap_or(MeshDrawGeometrySource::DynamicCpuMorphedSource),
        PendingMeshGeometry::CpuMorphed(_) => MeshDrawGeometrySource::DynamicCpuMorphedSource,
        PendingMeshGeometry::GpuMorphed(_) => MeshDrawGeometrySource::DynamicGpuMorphedSource,
    }
}

pub(super) fn pending_mesh_draw_queue_profile(
    pending_draw: &PendingMeshDraw,
    skinned_gpu_skinning_enabled: bool,
) -> MeshDrawQueueProfile {
    MeshDrawQueueProfile::new(
        MeshDrawQueuePhase::from_pipeline_flags(
            pending_draw.pipeline_key.is_transparent(),
            pending_draw.pipeline_key.is_alpha_mask(),
        ),
        pending_mesh_draw_geometry_source(pending_draw, skinned_gpu_skinning_enabled),
        pending_draw.mobility,
        pending_draw.indirect_draw_ref.is_some(),
        skinned_gpu_skinning_enabled,
        pending_draw.mesh_lod.is_some(),
    )
}

pub(super) fn pending_draw_has_enabled_skinned_gpu_source(pending_draw: &PendingMeshDraw) -> bool {
    pending_draw.pipeline_key.uses_fallback_shader()
        && pending_draw.resolved_skinned_gpu_source.is_some()
}

pub(super) fn skinned_gpu_source_geometry_source(
    source: &PendingSkinnedGpuSource,
    has_morph_payload_slot: bool,
) -> MeshDrawGeometrySource {
    match source {
        PendingSkinnedGpuSource::Prepared(_) if has_morph_payload_slot => {
            MeshDrawGeometrySource::DynamicGpuSkinnedMorphedSource
        }
        PendingSkinnedGpuSource::Prepared(_) => MeshDrawGeometrySource::DynamicGpuSkinningSource,
        PendingSkinnedGpuSource::CpuMorphed { .. } => {
            MeshDrawGeometrySource::DynamicCpuMorphedGpuSkinningSource
        }
    }
}

#[cfg(test)]
mod tests {
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
}
