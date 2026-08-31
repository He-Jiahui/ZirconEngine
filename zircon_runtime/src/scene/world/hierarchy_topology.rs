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
        self.dirty || self.indexed_entities.len() != entity_count
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
mod tests {
    use super::HierarchyTopology;

    #[test]
    fn roots_preserve_stable_order_across_reparent_and_removal() {
        let mut topology = HierarchyTopology::default();
        topology.update_parent(10, 1, None, None);
        topology.update_parent(20, 2, None, None);
        topology.update_parent(30, 3, None, None);
        assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 20, 30]);

        topology.update_parent(20, 2, None, Some(10));
        assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 30]);
        assert_eq!(topology.children_of(10).collect::<Vec<_>>(), vec![20]);

        topology.update_parent(20, 2, Some(10), None);
        assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 20, 30]);

        topology.remove_entity(20, 2, None);
        assert_eq!(topology.roots().collect::<Vec<_>>(), vec![10, 30]);
    }

    #[test]
    fn parent_projection_tracks_structural_updates_and_rebuilds() {
        let mut topology = HierarchyTopology::default();
        topology.update_parent(10, 1, None, None);
        topology.update_parent(20, 2, None, Some(10));
        assert_eq!(topology.parent_of(10), None);
        assert_eq!(topology.parent_of(20), Some(10));

        topology.update_parent(20, 2, Some(10), None);
        assert_eq!(topology.parent_of(20), None);

        topology.mark_dirty();
        topology.rebuild([(10, 1, None), (20, 2, Some(10))]);
        assert_eq!(topology.parent_of(20), Some(10));

        topology.remove_entity(20, 2, Some(10));
        topology.mark_current();
        assert_eq!(topology.parent_of(20), None);
    }

    #[test]
    fn missing_parent_projection_row_forces_source_rebuild() {
        let mut topology = HierarchyTopology::default();
        topology.update_parent(10, 1, None, None);
        topology.update_parent(20, 2, None, Some(10));
        topology.parent_by_entity.remove(&20);

        assert!(!topology.is_current_for_entity_count(2));
        assert!(topology.needs_source_rebuild(2));
    }
}
