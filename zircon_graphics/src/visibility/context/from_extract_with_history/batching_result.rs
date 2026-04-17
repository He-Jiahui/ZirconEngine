use std::collections::BTreeSet;

use zircon_scene::EntityId;

use super::super::super::declarations::{
    VisibilityBatch, VisibilityBvhInstance, VisibilityHistoryEntry,
};

pub(super) struct BatchingResult {
    pub(super) renderable_entities: BTreeSet<EntityId>,
    pub(super) static_entities: BTreeSet<EntityId>,
    pub(super) dynamic_entities: BTreeSet<EntityId>,
    pub(super) visible_entities: BTreeSet<EntityId>,
    pub(super) culled_entities: BTreeSet<EntityId>,
    pub(super) batches: Vec<VisibilityBatch>,
    pub(super) visible_batches: Vec<VisibilityBatch>,
    pub(super) bvh_instances: Vec<VisibilityBvhInstance>,
    pub(super) history_entries: Vec<VisibilityHistoryEntry>,
}
