use std::collections::BTreeSet;

use crate::core::framework::scene::EntityId;

use super::super::super::declarations::{
    VisibilityBatch, VisibilityBvhInstance, VisibilityHistoryEntry, VisibilityRelevanceEntry,
};
use super::super::super::view_context::FrameVisibility;

pub(super) struct BatchingResult {
    pub(super) frame_visibility: FrameVisibility,
    pub(super) renderable_entities: BTreeSet<EntityId>,
    pub(super) static_entities: BTreeSet<EntityId>,
    pub(super) dynamic_entities: BTreeSet<EntityId>,
    pub(super) visible_entities: BTreeSet<EntityId>,
    pub(super) culled_entities: BTreeSet<EntityId>,
    pub(super) primitive_relevance: Vec<VisibilityRelevanceEntry>,
    pub(super) batches: Vec<VisibilityBatch>,
    pub(super) bvh_instances: Vec<VisibilityBvhInstance>,
    pub(super) history_entries: Vec<VisibilityHistoryEntry>,
}
