use crate::core::framework::scene::EntityId;

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
    visibility_virtual_geometry_cluster::VisibilityVirtualGeometryCluster,
    visibility_virtual_geometry_draw_segment::VisibilityVirtualGeometryDrawSegment,
    visibility_virtual_geometry_feedback::VisibilityVirtualGeometryFeedback,
    visibility_virtual_geometry_page_upload_plan::VisibilityVirtualGeometryPageUploadPlan,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VisibilityContext {
    pub renderable_entities: Vec<EntityId>,
    pub static_entities: Vec<EntityId>,
    pub dynamic_entities: Vec<EntityId>,
    pub visible_entities: Vec<EntityId>,
    pub culled_entities: Vec<EntityId>,
    pub batches: Vec<VisibilityBatch>,
    pub visible_batches: Vec<VisibilityBatch>,
    pub visible_instances: Vec<EntityId>,
    pub draw_commands: Vec<VisibilityDrawCommand>,
    pub bvh_instances: Vec<VisibilityBvhInstance>,
    pub bvh_update_plan: VisibilityBvhUpdatePlan,
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
}
