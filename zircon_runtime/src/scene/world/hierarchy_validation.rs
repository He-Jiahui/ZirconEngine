use std::collections::{HashMap, HashSet};

use super::World;
use crate::scene::EntityId;
use crate::scene::components::Hierarchy;

impl World {
    pub(super) fn rebuild_hierarchy_validity(&mut self) {
        // First repair direct invalid edges, then select one stable edge from each actual cycle.
        // Descendants that merely lead into a cycle remain attached to its repaired root.
        let mut parents = self.hierarchy_parent_snapshot();
        let mut hierarchy_updates = Vec::new();
        let mut parent_chain_steps: usize = 0;
        let hierarchy_index_was_current = self
            .hierarchy_mutation_index
            .is_current_for_entity_count(self.entities.len());

        let entities = self.stable_entity_ids().collect::<Vec<_>>();
        let stable_orders = entities
            .iter()
            .enumerate()
            .map(|(order, entity)| (*entity, order))
            .collect::<HashMap<_, _>>();
        for entity in entities.iter().copied() {
            let Some(hierarchy) = self.get::<Hierarchy>(entity) else {
                continue;
            };
            let previous_parent = hierarchy.parent;
            let current_parent =
                previous_parent.filter(|parent| *parent != entity && parents.contains_key(parent));
            if previous_parent != current_parent {
                parents.insert(entity, current_parent);
                hierarchy_updates.push((entity, previous_parent, current_parent));
            }
        }

        let mut completed = HashSet::with_capacity(entities.len());
        let mut path_positions = HashMap::with_capacity(entities.len());
        let mut path = Vec::new();
        for start in entities.iter().copied() {
            if completed.contains(&start) {
                continue;
            }

            path.clear();
            let mut cursor = Some(start);
            while let Some(entity) = cursor {
                if completed.contains(&entity) {
                    break;
                }
                if let Some(cycle_start) = path_positions.get(&entity).copied() {
                    let repaired_entity = path[cycle_start..]
                        .iter()
                        .copied()
                        .min_by_key(|candidate| {
                            stable_orders
                                .get(candidate)
                                .copied()
                                .expect("cycle entity must retain stable order")
                        })
                        .expect("cycle path must contain the repeated entity");
                    let previous_parent = parents
                        .insert(repaired_entity, None)
                        .flatten()
                        .expect("cycle edge must retain a parent before repair");
                    hierarchy_updates.push((repaired_entity, Some(previous_parent), None));
                    break;
                }

                path_positions.insert(entity, path.len());
                path.push(entity);
                cursor = parents.get(&entity).copied().flatten();
                if cursor.is_some() {
                    parent_chain_steps = parent_chain_steps.saturating_add(1);
                }
            }

            for entity in path.drain(..) {
                path_positions.remove(&entity);
                completed.insert(entity);
            }
        }
        for (entity, previous_parent, current_parent) in hierarchy_updates.iter().copied() {
            let updated = if let Some(hierarchy) = self.get_mut::<Hierarchy>(entity) {
                hierarchy.parent = current_parent;
                true
            } else {
                false
            };
            if updated && hierarchy_index_was_current {
                self.update_hierarchy_mutation_index(entity, previous_parent, current_parent);
            }
        }
        if hierarchy_index_was_current {
            self.hierarchy_mutation_index.mark_current();
        }
        if !hierarchy_updates.is_empty() {
            self.derived_state_dirty.mark_hierarchy_repaired();
            for (entity, _, _) in hierarchy_updates {
                self.inspection_artifact_cache.mark_fields_dirty(entity);
            }
        }
        self.record_derived_state_hierarchy_validity(
            parents.len(),
            self.entities.len(),
            parent_chain_steps,
        );
    }

    fn hierarchy_parent_snapshot(&self) -> HashMap<EntityId, Option<EntityId>> {
        let mut parents = HashMap::with_capacity(self.entities.len());
        for entity in self.stable_entity_ids() {
            let parent = match self.get::<Hierarchy>(entity) {
                Some(hierarchy) => hierarchy.parent,
                None => None,
            };
            parents.insert(entity, parent);
        }
        parents
    }
}
