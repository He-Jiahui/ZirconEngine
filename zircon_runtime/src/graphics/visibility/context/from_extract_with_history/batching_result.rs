use std::collections::HashSet;

use crate::core::framework::scene::EntityId;

use super::super::super::declarations::{
    VisibilityBatch, VisibilityBvhInstance, VisibilityHistoryEntry, VisibilityRelevanceEntry,
};

pub(super) struct BatchingResult {
    pub(super) renderable_entities: HashSet<EntityId>,
    pub(super) static_entities: HashSet<EntityId>,
    pub(super) dynamic_entities: HashSet<EntityId>,
    pub(super) primitive_relevance: Vec<VisibilityRelevanceEntry>,
    pub(super) batches: Vec<VisibilityBatch>,
    pub(super) bvh_instances: Vec<VisibilityBvhInstance>,
    pub(super) history_entries: Vec<VisibilityHistoryEntry>,
}

pub(super) fn sorted_entity_ids(entities: HashSet<EntityId>) -> Vec<EntityId> {
    let mut entities = entities.into_iter().collect::<Vec<_>>();
    entities.sort_unstable();
    entities
}

#[cfg(test)]
mod optimization_tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn optimization_batch_20260826n_runtime09b_hash_entity_sets_preserve_sorted_output() {
        let entities = HashSet::from([41, 7, 19, 7, 2]);

        assert_eq!(sorted_entity_ids(entities), vec![2, 7, 19, 41]);
    }
}
