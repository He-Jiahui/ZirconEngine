use std::collections::{BTreeMap, HashMap, HashSet};

use crate::scene::EntityId;

/// Incremental parent-to-children projection used by affected-row mutations.
/// Dense component rows remain the hierarchy authority; this topology keeps the
/// stable root and child ordering needed by subtree-local derived-state work.
#[derive(Debug, Default)]
pub(super) struct HierarchyTopology {
    roots: BTreeMap<usize, EntityId>,
    children_by_parent: HashMap<EntityId, BTreeMap<usize, EntityId>>,
    parent_by_entity: HashMap<EntityId, Option<EntityId>>,
    indexed_entities: HashSet<EntityId>,
    generation: u64,
    dirty: bool,
}

impl PartialEq for HierarchyTopology {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl HierarchyTopology {
    pub(super) fn update_parent(
        &mut self,
        entity: EntityId,
        stable_order: usize,
        previous_parent: Option<EntityId>,
        current_parent: Option<EntityId>,
    ) {
        let mut changed = false;
        if previous_parent != current_parent {
            if let Some(previous_parent) = previous_parent {
                self.remove_child(previous_parent, stable_order, entity);
            } else {
                self.roots.remove(&stable_order);
            }
            self.insert_parent(entity, stable_order, current_parent);
            changed = true;
        } else if !self.indexed_entities.contains(&entity) {
            self.insert_parent(entity, stable_order, current_parent);
            changed = true;
        }
        changed |= self.indexed_entities.insert(entity);
        self.parent_by_entity.insert(entity, current_parent);
        if changed {
            self.mark_structural_change();
        }
    }

    pub(super) fn remove_entity(
        &mut self,
        entity: EntityId,
        stable_order: usize,
        parent: Option<EntityId>,
    ) {
        if let Some(parent) = parent {
            self.remove_child(parent, stable_order, entity);
        } else {
            self.roots.remove(&stable_order);
        }
        self.indexed_entities.remove(&entity);
        self.parent_by_entity.remove(&entity);
        self.children_by_parent.remove(&entity);
        self.mark_structural_change();
    }

    pub(super) fn children_of(
        &self,
        parent: EntityId,
    ) -> impl DoubleEndedIterator<Item = EntityId> + '_ {
        debug_assert!(!self.dirty);
        self.children_by_parent
            .get(&parent)
            .into_iter()
            .flat_map(|children| children.values().copied())
    }

    pub(super) fn parent_of(&self, entity: EntityId) -> Option<EntityId> {
        debug_assert!(!self.dirty);
        self.parent_by_entity.get(&entity).copied().flatten()
    }

    pub(super) fn roots(&self) -> impl DoubleEndedIterator<Item = EntityId> + '_ {
        debug_assert!(!self.dirty);
        self.roots.values().copied()
    }

    pub(super) fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(super) fn mark_current(&mut self) {
        self.dirty = false;
    }

    pub(super) fn is_current_for_entity_count(&self, entity_count: usize) -> bool {
        !self.dirty
            && self.indexed_entities.len() == entity_count
            && self.parent_by_entity.len() == entity_count
    }

    pub(super) fn needs_source_rebuild(&self, entity_count: usize) -> bool {
        !self.is_current_for_entity_count(entity_count)
    }

    pub(super) const fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn rebuild(
        &mut self,
        rows: impl IntoIterator<Item = (EntityId, usize, Option<EntityId>)>,
    ) {
        self.roots.clear();
        self.children_by_parent.clear();
        self.parent_by_entity.clear();
        self.indexed_entities.clear();
        for (entity, stable_order, parent) in rows {
            self.insert_parent(entity, stable_order, parent);
            self.parent_by_entity.insert(entity, parent);
            self.indexed_entities.insert(entity);
        }
        self.dirty = false;
        self.mark_structural_change();
    }

    fn insert_parent(&mut self, entity: EntityId, stable_order: usize, parent: Option<EntityId>) {
        if let Some(parent) = parent {
            let replaced = self
                .children_by_parent
                .entry(parent)
                .or_default()
                .insert(stable_order, entity);
            debug_assert!(replaced.is_none() || replaced == Some(entity));
        } else {
            let replaced = self.roots.insert(stable_order, entity);
            debug_assert!(replaced.is_none() || replaced == Some(entity));
        }
    }

    fn remove_child(&mut self, parent: EntityId, stable_order: usize, entity: EntityId) {
        let remove_bucket = if let Some(children) = self.children_by_parent.get_mut(&parent) {
            let removed = children.remove(&stable_order);
            debug_assert!(removed.is_none() || removed == Some(entity));
            children.is_empty()
        } else {
            false
        };
        if remove_bucket {
            self.children_by_parent.remove(&parent);
        }
    }

    fn mark_structural_change(&mut self) {
        self.generation = self.generation.saturating_add(1);
    }
}

#[cfg(test)]
#[path = "tests/hierarchy_topology.rs"]
mod tests;
