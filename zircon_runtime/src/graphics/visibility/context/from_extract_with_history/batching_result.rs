use std::collections::HashSet;

use crate::core::framework::scene::EntityId;

use super::super::super::declarations::{
    VisibilityBatch, VisibilityBvhInstance, VisibilityHistoryEntry, VisibilityRelevanceEntry,
};

pub(super) struct BatchingResult {
    pub(super) renderable_entities: EntitySet,
    pub(super) static_entities: EntitySet,
    pub(super) dynamic_entities: EntitySet,
    pub(super) primitive_relevance: Vec<VisibilityRelevanceEntry>,
    pub(super) batches: Vec<VisibilityBatch>,
    pub(super) bvh_instances: Vec<VisibilityBvhInstance>,
    pub(super) history_entries: Vec<VisibilityHistoryEntry>,
}

pub(super) struct EntitySet {
    membership: Option<HashSet<EntityId>>,
    values: Vec<EntityId>,
    input_was_sorted: bool,
    input_was_reverse_sorted: bool,
    last_value: Option<EntityId>,
}

impl EntitySet {
    pub(super) fn with_capacity(capacity: usize) -> Self {
        Self {
            membership: None,
            values: Vec::with_capacity(capacity),
            input_was_sorted: true,
            input_was_reverse_sorted: true,
            last_value: None,
        }
    }

    pub(super) fn insert(&mut self, entity: EntityId) {
        if let Some(membership) = &mut self.membership {
            if !membership.insert(entity) {
                return;
            }
        } else {
            if self.last_value == Some(entity) {
                return;
            }
            let input_was_sorted = self.input_was_sorted
                && self
                    .last_value
                    .is_none_or(|last_value| last_value <= entity);
            let input_was_reverse_sorted = self.input_was_reverse_sorted
                && self
                    .last_value
                    .is_none_or(|last_value| last_value >= entity);
            if !input_was_sorted && !input_was_reverse_sorted {
                self.input_was_sorted = false;
                self.input_was_reverse_sorted = false;
                let mut membership = HashSet::with_capacity(self.values.len() + 1);
                membership.extend(self.values.iter().copied());
                if !membership.insert(entity) {
                    self.membership = Some(membership);
                    return;
                }
                self.membership = Some(membership);
            }
        }
        if let Some(last_value) = self.last_value {
            self.input_was_sorted &= last_value <= entity;
            self.input_was_reverse_sorted &= last_value >= entity;
        }
        self.last_value = Some(entity);
        self.values.push(entity);
    }

    fn into_sorted_vec(mut self) -> Vec<EntityId> {
        if self.input_was_reverse_sorted && !self.input_was_sorted {
            self.values.reverse();
        } else if !self.input_was_sorted {
            self.values.sort_unstable();
        }
        self.values
    }
}

pub(super) fn sorted_entity_ids(entities: EntitySet) -> Vec<EntityId> {
    entities.into_sorted_vec()
}

#[cfg(test)]
#[path = "tests/batching_result_optimization_tests.rs"]
mod optimization_tests;
