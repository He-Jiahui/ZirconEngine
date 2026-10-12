use std::sync::Arc;

use crate::asset::ModelPrimitiveAsset;
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::{
    MeshDrawGeometrySource, MeshDrawQueuePhase, MeshDrawQueueProfile,
};

use super::pending_mesh_draw::{PendingMeshDraw, PendingMeshGeometry, PendingSkinnedGpuSource};

/// 待绘制项在最终 MeshDraw 构造时采用的几何来源，也决定 shader 几何源标识。
/// 仅 fallback shader 且已解析 GPU 蒙皮源时可选蒙皮分支。
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
            pending_draw.material.pipeline_key.is_transparent(),
            pending_draw.material.pipeline_key.is_alpha_mask(),
        ),
        pending_mesh_draw_geometry_source(pending_draw, skinned_gpu_skinning_enabled),
        pending_draw.mobility,
        pending_draw.indirect_draw_ref.is_some(),
        skinned_gpu_skinning_enabled,
        pending_draw.mesh_lod.is_some(),
    )
}

pub(super) fn pending_draw_has_enabled_skinned_gpu_source(pending_draw: &PendingMeshDraw) -> bool {
    pending_draw.material.pipeline_key.uses_fallback_shader()
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
#[path = "tests/geometry_source_selection.rs"]
mod tests;
