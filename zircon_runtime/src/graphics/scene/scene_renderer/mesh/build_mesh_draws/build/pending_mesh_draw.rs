use std::sync::Arc;

use crate::asset::ModelPrimitiveAsset;
use crate::core::framework::render::{RenderMeshLodSelection, RenderMeshStaticState};
use crate::core::framework::scene::EntityId;
use crate::core::framework::scene::Mobility;
use crate::core::math::Vec4;
use crate::graphics::scene::resources::{GpuMaterialUniformResource, GpuMeshResource};
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MaterialTextureSet;
use bytemuck::{Pod, Zeroable};

use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteUniform;

pub(super) enum PendingMeshGeometry {
    Prepared(Arc<GpuMeshResource>),
    Dynamic(ModelPrimitiveAsset),
}

#[derive(Clone)]
pub(super) enum PendingSkinnedGpuSource {
    Prepared(Arc<GpuMeshResource>),
    CpuMorphed(ModelPrimitiveAsset),
}

impl PendingSkinnedGpuSource {
    pub(super) fn uses_cpu_morphed_source(&self) -> bool {
        matches!(self, Self::CpuMorphed(_))
    }
}

pub(super) struct PendingMeshDraw {
    pub(super) mesh: PendingMeshGeometry,
    pub(super) source_entity: EntityId,
    pub(super) source_draw_ordinal: u32,
    pub(super) transform_revision: u64,
    pub(super) mobility: Mobility,
    pub(super) static_state: RenderMeshStaticState,
    pub(super) material_textures: MaterialTextureSet,
    pub(super) material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) standard_material_uniform: Arc<GpuMaterialUniformResource>,
    pub(super) pipeline_key: crate::graphics::scene::resources::PipelineKey,
    pub(super) mesh_lod: Option<RenderMeshLodSelection>,
    pub(super) cast_shadows: bool,
    pub(super) receive_shadows: bool,
    pub(super) model_matrix: [[f32; 4]; 4],
    pub(super) previous_model_matrix: [[f32; 4]; 4],
    pub(super) has_previous_motion_vector_transform: bool,
    pub(super) draw_tint: Vec4,
    pub(super) skinned: bool,
    pub(super) skinned_joint_palette: Option<SkinnedMeshJointPaletteUniform>,
    pub(super) previous_skinned_joint_palette: Option<SkinnedMeshJointPaletteUniform>,
    // Holds the original prepared mesh for the guarded shader-skinning path.
    // CPU-skinned dynamic fallback draws leave this empty to avoid double skinning.
    pub(super) skinned_gpu_source: Option<PendingSkinnedGpuSource>,
    pub(super) first_index: u32,
    pub(super) draw_index_count: u32,
    pub(super) indirect_draw_ref: Option<VirtualGeometryIndirectDrawRef>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct VirtualGeometryIndirectDrawRef {
    #[allow(dead_code)]
    pub(super) mesh_index_count: u32,
    #[allow(dead_code)]
    pub(super) mesh_signature: u64,
    pub(super) segment_key: VirtualGeometryIndirectSegmentKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct VirtualGeometryIndirectSegmentKey {
    pub(super) submission_index: u32,
    pub(super) instance_index: Option<u32>,
    pub(super) entity: EntityId,
    pub(super) page_id: u32,
    pub(super) cluster_start_ordinal: u32,
    pub(super) cluster_span_count: u32,
    pub(super) cluster_total_count: u32,
    pub(super) lineage_depth: u32,
    pub(super) lod_level: u8,
    pub(super) frontier_rank: u32,
    pub(super) submission_slot: Option<u32>,
    pub(super) state: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct VirtualGeometryIndirectSegmentInput {
    pub(super) cluster_start_ordinal: u32,
    pub(super) cluster_span_count: u32,
    pub(super) cluster_total_count: u32,
    pub(super) page_id: u32,
    pub(super) submission_slot: u32,
    pub(super) state: u32,
    pub(super) lineage_depth: u32,
    pub(super) lod_level: u32,
    pub(super) frontier_rank: u32,
    pub(super) submission_index: u32,
    pub(super) instance_index: u32,
    pub(super) entity_lo: u32,
    pub(super) entity_hi: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct VirtualGeometryIndirectDrawRefInput {
    pub(super) mesh_index_count: u32,
    pub(super) segment_index: u32,
    pub(super) segment_draw_ref_count: u32,
    pub(super) submission_token: u32,
}

#[allow(dead_code)]
pub(super) fn segment_input(
    segment_key: VirtualGeometryIndirectSegmentKey,
) -> VirtualGeometryIndirectSegmentInput {
    VirtualGeometryIndirectSegmentInput {
        cluster_start_ordinal: segment_key.cluster_start_ordinal,
        cluster_span_count: segment_key.cluster_span_count,
        cluster_total_count: segment_key.cluster_total_count,
        page_id: segment_key.page_id,
        submission_slot: segment_key.submission_slot.unwrap_or_default(),
        state: segment_key.state,
        lineage_depth: segment_key.lineage_depth,
        lod_level: u32::from(segment_key.lod_level),
        frontier_rank: segment_key.frontier_rank,
        submission_index: segment_key.submission_index,
        instance_index: segment_key.instance_index.unwrap_or(u32::MAX),
        entity_lo: segment_key.entity as u32,
        entity_hi: (segment_key.entity >> 32) as u32,
    }
}

#[allow(dead_code)]
pub(super) fn draw_ref_input(
    mesh_index_count: u32,
    segment_index: u32,
    segment_draw_ref_count: u32,
    submission_token: u32,
) -> VirtualGeometryIndirectDrawRefInput {
    VirtualGeometryIndirectDrawRefInput {
        mesh_index_count,
        segment_index,
        segment_draw_ref_count,
        submission_token,
    }
}
