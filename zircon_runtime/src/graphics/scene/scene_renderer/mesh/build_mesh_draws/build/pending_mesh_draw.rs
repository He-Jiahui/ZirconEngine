use std::sync::Arc;

use crate::asset::ModelPrimitiveAsset;
use crate::core::framework::render::{
    RenderMeshBounds, RenderMeshLodSelection, RenderMeshStaticState,
};
use crate::core::framework::scene::EntityId;
use crate::core::framework::scene::Mobility;
use crate::graphics::scene::gpu_scene::{GpuMorphDelta, GpuMorphWeight};
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::MeshCommandSortInput;
use bytemuck::{Pod, Zeroable};

use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteStorage;

use super::pending_material_draw::PendingMaterialDraw;

#[derive(Clone)]
pub(super) enum PendingMeshGeometry {
    Prepared(Arc<GpuMeshResource>),
    Dynamic(ModelPrimitiveAsset),
    CpuMorphed(ModelPrimitiveAsset),
    GpuMorphed(Arc<GpuMeshResource>),
}

#[derive(Clone)]
pub(super) enum PendingSkinnedGpuSource {
    Prepared(Arc<GpuMeshResource>),
    CpuMorphed {
        primitive: ModelPrimitiveAsset,
        morph_shape_signature: u64,
    },
}

impl PendingSkinnedGpuSource {
    pub(super) fn uses_cpu_morphed_source(&self) -> bool {
        matches!(self, Self::CpuMorphed { .. })
    }

    pub(super) fn morph_shape_signature(&self) -> Option<u64> {
        match self {
            Self::Prepared(_) => None,
            Self::CpuMorphed {
                morph_shape_signature,
                ..
            } => Some(*morph_shape_signature),
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct PendingMorphPayload {
    pub(super) vertex_count: u32,
    pub(super) target_count: u32,
    pub(super) deltas: Vec<GpuMorphDelta>,
    pub(super) weights: Vec<GpuMorphWeight>,
    pub(super) previous_weights: Vec<GpuMorphWeight>,
}

#[derive(Clone)]
pub(super) struct PendingMeshDraw {
    pub(super) mesh: PendingMeshGeometry,
    pub(super) local_bounds: RenderMeshBounds,
    pub(super) source_entity: EntityId,
    /// Render-instance identity from scene extraction; distinct primitives may share an entity.
    pub(super) stable_instance_key: u64,
    pub(super) source_draw_ordinal: u32,
    pub(super) transform_revision: u64,
    pub(super) mobility: Mobility,
    pub(super) static_state: RenderMeshStaticState,
    pub(super) material: PendingMaterialDraw,
    pub(super) morph_payload: Option<Arc<PendingMorphPayload>>,
    pub(super) source_morph_weights: Option<Vec<f32>>,
    pub(super) morph_payload_slot: Option<u32>,
    pub(super) mesh_lod: Option<RenderMeshLodSelection>,
    pub(super) model_matrix: [[f32; 4]; 4],
    /// Affine classification shared by raster-state selection and GPUScene upload.
    pub(super) normal_transform_flags: u32,
    pub(super) skinned: bool,
    pub(super) skinned_palette_signature: Option<u64>,
    pub(super) skinned_joint_palette: Option<SkinnedMeshJointPaletteStorage>,
    pub(super) previous_skinned_joint_palette: Option<SkinnedMeshJointPaletteStorage>,
    // Holds the original prepared mesh for the guarded shader-skinning path.
    // CPU-skinned dynamic fallback draws leave this empty to avoid double skinning.
    pub(super) skinned_gpu_source: Option<PendingSkinnedGpuSource>,
    pub(super) resolved_skinned_gpu_source: Option<Arc<GpuMeshResource>>,
    pub(super) previous_skinned_gpu_source: Option<Arc<GpuMeshResource>>,
    pub(super) command_sort_input: MeshCommandSortInput,
    pub(super) first_index: u32,
    pub(super) draw_index_count: u32,
    pub(super) indirect_draw_ref: Option<VirtualGeometryIndirectDrawRef>,
}

impl PendingMeshDraw {
    pub(super) fn hzb_bounds_are_temporally_stable(&self) -> bool {
        !self.skinned
            && hzb_geometry_bounds_are_temporally_stable(&self.mesh)
            && morph_bounds_are_temporally_stable(self.morph_payload.as_deref())
    }
}

fn hzb_geometry_bounds_are_temporally_stable(mesh: &PendingMeshGeometry) -> bool {
    !matches!(mesh, PendingMeshGeometry::CpuMorphed(_))
}

fn morph_bounds_are_temporally_stable(payload: Option<&PendingMorphPayload>) -> bool {
    payload.map_or(true, |payload| payload.weights == payload.previous_weights)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::graphics::scene::gpu_scene::GpuMorphWeight;

    use crate::asset::ModelPrimitiveAsset;

    use super::{
        PendingMeshGeometry, PendingMorphPayload, hzb_geometry_bounds_are_temporally_stable,
        morph_bounds_are_temporally_stable,
    };

    #[test]
    fn equal_current_and_previous_morph_weights_are_temporally_stable() {
        let weights = vec![GpuMorphWeight::new(0.5)];
        let payload = Arc::new(PendingMorphPayload {
            vertex_count: 1,
            target_count: 1,
            deltas: Vec::new(),
            weights: weights.clone(),
            previous_weights: weights,
        });

        assert!(morph_bounds_are_temporally_stable(Some(&payload)));
    }

    #[test]
    fn changed_morph_weights_force_temporal_hzb_visibility() {
        let payload = PendingMorphPayload {
            vertex_count: 1,
            target_count: 1,
            deltas: Vec::new(),
            weights: vec![GpuMorphWeight::new(0.75)],
            previous_weights: vec![GpuMorphWeight::new(0.25)],
        };

        assert!(!morph_bounds_are_temporally_stable(Some(&payload)));
    }

    #[test]
    fn cpu_morphed_geometry_forces_temporal_hzb_visibility() {
        let primitive = ModelPrimitiveAsset {
            vertices: Vec::new(),
            indices: Vec::new(),
            mesh: None,
            mesh_sdf: None,
            virtual_geometry: None,
        };

        assert!(!hzb_geometry_bounds_are_temporally_stable(
            &PendingMeshGeometry::CpuMorphed(primitive.clone())
        ));
        assert!(hzb_geometry_bounds_are_temporally_stable(
            &PendingMeshGeometry::Dynamic(primitive)
        ));
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct VirtualGeometryIndirectDrawRef {
    pub(super) segment_key: VirtualGeometryIndirectSegmentKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct VirtualGeometryIndirectSegmentKey {
    pub(super) submission_index: u32,
    pub(super) instance_index: Option<u32>,
    pub(super) entity: EntityId,
    pub(super) stable_instance_key: u64,
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
