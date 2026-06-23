use std::collections::BTreeSet;

use crate::core::framework::scene::EntityId;

use super::super::view_context::FrameVisibility;
use super::super::{VisibilityStaticIndex, VisibilityStaticIndexReport};
use super::{
    visibility_batch::VisibilityBatch, visibility_bvh_instance::VisibilityBvhInstance,
    visibility_bvh_update_plan::VisibilityBvhUpdatePlan,
    visibility_draw_command::VisibilityDrawCommand,
    visibility_history_snapshot::VisibilityHistorySnapshot,
    visibility_hybrid_gi_feedback::VisibilityHybridGiFeedback,
    visibility_hybrid_gi_probe::VisibilityHybridGiProbe,
    visibility_hybrid_gi_update_plan::VisibilityHybridGiUpdatePlan,
    visibility_instance_upload_plan::VisibilityInstanceUploadPlan,
    visibility_particle_upload_plan::VisibilityParticleUploadPlan,
    visibility_relevance_entry::VisibilityRelevanceEntry,
    visibility_virtual_geometry_cluster::VisibilityVirtualGeometryCluster,
    visibility_virtual_geometry_draw_segment::VisibilityVirtualGeometryDrawSegment,
    visibility_virtual_geometry_feedback::VisibilityVirtualGeometryFeedback,
    visibility_virtual_geometry_page_upload_plan::VisibilityVirtualGeometryPageUploadPlan,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityContext {
    pub frame_visibility: FrameVisibility,
    pub renderable_entities: Vec<EntityId>,
    pub static_entities: Vec<EntityId>,
    pub dynamic_entities: Vec<EntityId>,
    pub primitive_relevance: Vec<VisibilityRelevanceEntry>,
    pub batches: Vec<VisibilityBatch>,
    pub visible_instances: Vec<EntityId>,
    pub draw_commands: Vec<VisibilityDrawCommand>,
    pub bvh_instances: Vec<VisibilityBvhInstance>,
    pub bvh_update_plan: VisibilityBvhUpdatePlan,
    pub static_index_report: VisibilityStaticIndexReport,
    pub history_snapshot: VisibilityHistorySnapshot,
    pub instance_upload_plan: VisibilityInstanceUploadPlan,
    pub particle_upload_plan: VisibilityParticleUploadPlan,
    pub hybrid_gi_active_probes: Vec<VisibilityHybridGiProbe>,
    pub hybrid_gi_update_plan: VisibilityHybridGiUpdatePlan,
    pub hybrid_gi_feedback: VisibilityHybridGiFeedback,
    pub virtual_geometry_visible_clusters: Vec<VisibilityVirtualGeometryCluster>,
    pub virtual_geometry_draw_segments: Vec<VisibilityVirtualGeometryDrawSegment>,
    pub virtual_geometry_page_upload_plan: VisibilityVirtualGeometryPageUploadPlan,
    pub virtual_geometry_feedback: VisibilityVirtualGeometryFeedback,
    pub gpu_instancing_candidates: Vec<VisibilityBatch>,
    pub(crate) static_index: VisibilityStaticIndex,
}

impl VisibilityContext {
    pub(crate) fn static_index(&self) -> &VisibilityStaticIndex {
        &self.static_index
    }

    /// Main-view visibility is derived from `FrameVisibility` so there is only one
    /// authoritative per-view visibility store in the context.
    pub fn main_view_visible_entities(&self) -> Vec<EntityId> {
        self.main_view_visible_entity_set().into_iter().collect()
    }

    pub fn main_view_visible_entity_set(&self) -> BTreeSet<EntityId> {
        self.frame_visibility.main_view_visible_entity_set()
    }

    pub fn main_view_culled_entities(&self) -> Vec<EntityId> {
        let visible_entities = self.main_view_visible_entity_set();
        self.renderable_entities
            .iter()
            .copied()
            .filter(|entity| !visible_entities.contains(entity))
            .collect()
    }

    pub fn main_view_visible_batches(&self) -> Vec<VisibilityBatch> {
        Self::visible_batches_for_entities(&self.batches, &self.main_view_visible_entity_set())
    }

    pub(crate) fn visible_batches_for_entities(
        batches: &[VisibilityBatch],
        visible_entities: &BTreeSet<EntityId>,
    ) -> Vec<VisibilityBatch> {
        batches
            .iter()
            .filter_map(|batch| {
                let entities = batch
                    .entities
                    .iter()
                    .copied()
                    .filter(|entity| visible_entities.contains(entity))
                    .collect::<Vec<_>>();
                (!entities.is_empty()).then_some(VisibilityBatch {
                    key: batch.key.clone(),
                    entities,
                })
            })
            .collect()
    }
}
