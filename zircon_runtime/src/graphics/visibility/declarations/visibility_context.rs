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
    /// Stable render-instance keys selected for the main view draw commands.
    pub visible_instances: Vec<u64>,
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
    pub(crate) dynamic_index: VisibilityStaticIndex,
}

impl VisibilityContext {
    pub(crate) fn static_index(&self) -> &VisibilityStaticIndex {
        &self.static_index
    }

    pub(crate) fn dynamic_index(&self) -> &VisibilityStaticIndex {
        &self.dynamic_index
    }

    /// Main-view visibility is derived from `FrameVisibility` so there is only one
    /// authoritative per-view visibility store in the context.
    pub fn main_view_visible_entities(&self) -> Vec<EntityId> {
        self.main_view_visible_entity_set().into_iter().collect()
    }

    pub fn main_view_visible_entity_set(&self) -> BTreeSet<EntityId> {
        self.frame_visibility.main_view_visible_entity_set()
    }

    pub fn main_view_visible_stable_instance_keys(&self) -> Vec<u64> {
        self.frame_visibility
            .main_view_visible_stable_instance_key_set()
            .into_iter()
            .collect()
    }

    pub fn main_view_culled_entities(&self) -> Vec<EntityId> {
        let visible_entities = self.main_view_visible_entity_set();
        self.renderable_entities
            .iter()
            .copied()
            .filter(|entity| !visible_entities.contains(entity))
            .collect()
    }

    pub fn main_view_culled_stable_instance_keys(&self) -> Vec<u64> {
        let visible_stable_instance_keys = self
            .frame_visibility
            .main_view_visible_stable_instance_key_set();
        self.bvh_instances
            .iter()
            .map(|instance| instance.stable_instance_key)
            .filter(|stable_instance_key| {
                !visible_stable_instance_keys.contains(stable_instance_key)
            })
            .collect()
    }

    pub fn main_view_visible_batches(&self) -> Vec<VisibilityBatch> {
        Self::visible_batches_for_stable_instance_keys(
            &self.batches,
            &self
                .frame_visibility
                .main_view_visible_stable_instance_key_set(),
        )
    }

    pub(crate) fn visible_batches_for_stable_instance_keys(
        batches: &[VisibilityBatch],
        visible_stable_instance_keys: &BTreeSet<u64>,
    ) -> Vec<VisibilityBatch> {
        batches
            .iter()
            .filter_map(|batch| {
                let members = batch
                    .stable_instance_keys
                    .iter()
                    .zip(batch.entities.iter())
                    .filter(|(stable_instance_key, _)| {
                        visible_stable_instance_keys.contains(stable_instance_key)
                    })
                    .map(|(stable_instance_key, entity)| (*stable_instance_key, *entity))
                    .collect::<Vec<_>>();
                (!members.is_empty()).then(|| {
                    let (stable_instance_keys, entities) = members.into_iter().unzip();
                    VisibilityBatch {
                        key: batch.key.clone(),
                        stable_instance_keys,
                        entities,
                    }
                })
            })
            .collect()
    }
}
