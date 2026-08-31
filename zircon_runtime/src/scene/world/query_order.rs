use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap, HashMap, btree_map};

use crate::scene::EntityId;
use crate::scene::ecs::{ArchetypeId, EntityLocation, InternalEntity, StableEntityLocation};

/// Keeps the stable world order separately from swap-remove archetype rows.
///
/// This index intentionally stores only entity-to-archetype membership. Dense component values
/// and change ticks remain owned by each archetype table; sparse values remain in component
/// storage.
#[derive(Debug, Default)]
pub(super) struct StableQueryOrderIndex {
    next_order: usize,
    entries: HashMap<EntityId, StableQueryOrderEntry>,
    entities_by_order: BTreeMap<usize, EntityId>,
    entities_by_archetype: Vec<BTreeMap<usize, StableEntityLocation>>,
}

#[derive(Debug)]
struct StableQueryOrderEntry {
    order: usize,
    internal: InternalEntity,
    archetype: Option<ArchetypeId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct QueryOrderCandidate {
    order: usize,
    iterator_index: usize,
    location: StableEntityLocation,
}

pub(crate) struct StableQueryLocationIter<'world> {
    iterators: Vec<btree_map::Iter<'world, usize, StableEntityLocation>>,
    next_entities: BinaryHeap<QueryOrderCandidate>,
}

/// Iterates the stable logical world order without depending on the physical
/// swap-remove layout used by entity storage.
pub(crate) struct StableWorldEntityIter<'world> {
    entities: btree_map::Values<'world, usize, EntityId>,
}

impl Iterator for StableWorldEntityIter<'_> {
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next().copied()
    }
}

impl Iterator for StableQueryLocationIter<'_> {
    type Item = StableEntityLocation;

    fn next(&mut self) -> Option<Self::Item> {
        let candidate = self.next_entities.pop()?;
        if let Some((&order, &location)) = self.iterators[candidate.iterator_index].next() {
            self.next_entities.push(QueryOrderCandidate {
                order,
                iterator_index: candidate.iterator_index,
                location,
            });
        }
        Some(candidate.location)
    }
}

impl Ord for QueryOrderCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .order
            .cmp(&self.order)
            .then_with(|| other.location.stable_id.cmp(&self.location.stable_id))
            .then_with(|| other.iterator_index.cmp(&self.iterator_index))
    }
}

impl PartialOrd for QueryOrderCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for StableQueryOrderIndex {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl StableQueryOrderIndex {
    pub(super) fn register(&mut self, entity: EntityId, internal: InternalEntity) {
        let order = self.next_order;
        self.next_order = self
            .next_order
            .checked_add(1)
            .expect("stable query order exhausted");
        self.register_at_order(entity, internal, order);
    }

    pub(super) fn register_at_order(
        &mut self,
        entity: EntityId,
        internal: InternalEntity,
        order: usize,
    ) {
        let replaced = self.entries.insert(
            entity,
            StableQueryOrderEntry {
                order,
                internal,
                archetype: None,
            },
        );
        debug_assert!(replaced.is_none());
        let replaced = self.entities_by_order.insert(order, entity);
        debug_assert!(replaced.is_none());
        self.next_order = self
            .next_order
            .max(order.checked_add(1).expect("stable query order exhausted"));
    }

    pub(super) fn order_of(&self, entity: EntityId) -> Option<usize> {
        self.entries.get(&entity).map(|entry| entry.order)
    }

    pub(super) fn contains_order(&self, order: usize) -> bool {
        self.entities_by_order.contains_key(&order)
    }

    pub(super) fn remove(&mut self, entity: EntityId) {
        let entry = self
            .entries
            .remove(&entity)
            .expect("removed entity must have stable query order");
        let removed = self.entities_by_order.remove(&entry.order);
        debug_assert_eq!(removed, Some(entity));
        let Some(archetype) = entry.archetype else {
            return;
        };
        let removed = self
            .entities_by_archetype
            .get_mut(archetype.index())
            .and_then(|entities| entities.remove(&entry.order));
        debug_assert_eq!(removed.map(|location| location.stable_id), Some(entity));
    }

    pub(super) fn rebuild(
        &mut self,
        entities: impl IntoIterator<Item = (EntityId, InternalEntity)>,
    ) {
        self.next_order = 0;
        self.entries.clear();
        self.entities_by_order.clear();
        self.entities_by_archetype.clear();
        for (entity, internal) in entities {
            self.register(entity, internal);
        }
    }

    pub(super) fn clear_archetypes(&mut self) {
        for entities in &mut self.entities_by_archetype {
            entities.clear();
        }
        for entry in self.entries.values_mut() {
            entry.archetype = None;
        }
    }

    pub(super) fn entities(&self) -> StableWorldEntityIter<'_> {
        StableWorldEntityIter {
            entities: self.entities_by_order.values(),
        }
    }

    pub(super) fn move_to(&mut self, entity: EntityId, location: EntityLocation) {
        let archetype = location.archetype_id;
        let (order, internal, previous) = {
            let entry = self
                .entries
                .get_mut(&entity)
                .expect("moved entity must have stable query order");
            let previous = entry.archetype.replace(archetype);
            (entry.order, entry.internal, previous)
        };
        let location = StableEntityLocation {
            stable_id: entity,
            internal,
            location,
        };
        self.ensure_archetype(archetype);
        if previous == Some(archetype) {
            let replaced = self.entities_by_archetype[archetype.index()].insert(order, location);
            debug_assert!(replaced.is_some());
            return;
        }
        if let Some(previous) = previous {
            let removed = self
                .entities_by_archetype
                .get_mut(previous.index())
                .and_then(|entities| entities.remove(&order));
            debug_assert_eq!(removed.map(|location| location.stable_id), Some(entity));
        }
        let replaced = self.entities_by_archetype[archetype.index()].insert(order, location);
        debug_assert!(replaced.is_none());
    }

    pub(super) fn update_row(&mut self, entity: EntityId, row: usize) {
        let entry = self
            .entries
            .get(&entity)
            .expect("moved archetype row must have stable query order");
        let archetype = entry
            .archetype
            .expect("moved archetype row must belong to an archetype");
        let location = self.entities_by_archetype[archetype.index()]
            .get_mut(&entry.order)
            .expect("moved archetype row must have indexed stable location");
        debug_assert_eq!(location.stable_id, entity);
        location.location.table_row = row;
    }

    pub(super) fn visit_matching(
        &self,
        archetypes: &[ArchetypeId],
        mut visitor: impl FnMut(StableEntityLocation),
    ) {
        for location in self.iter_matching(archetypes.iter().copied()) {
            visitor(location);
        }
    }

    pub(super) fn iter_matching(
        &self,
        archetypes: impl IntoIterator<Item = ArchetypeId>,
    ) -> StableQueryLocationIter<'_> {
        let archetypes = archetypes.into_iter();
        let (lower_bound, _) = archetypes.size_hint();
        let mut iterators = Vec::with_capacity(lower_bound);
        let mut next_entities = BinaryHeap::with_capacity(lower_bound);
        for archetype in archetypes {
            let Some(entities) = self.entities_by_archetype.get(archetype.index()) else {
                continue;
            };
            let mut iterator = entities.iter();
            let Some((&order, &location)) = iterator.next() else {
                continue;
            };
            let iterator_index = iterators.len();
            iterators.push(iterator);
            next_entities.push(QueryOrderCandidate {
                order,
                iterator_index,
                location,
            });
        }

        StableQueryLocationIter {
            iterators,
            next_entities,
        }
    }

    fn ensure_archetype(&mut self, archetype: ArchetypeId) {
        if self.entities_by_archetype.len() <= archetype.index() {
            self.entities_by_archetype
                .resize_with(archetype.index() + 1, BTreeMap::new);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visits_matching_archetypes_in_stable_world_order_after_moves_and_removal() {
        let first = ArchetypeId::new(1);
        let second = ArchetypeId::new(2);
        let mut index = StableQueryOrderIndex::default();
        for entity in [10, 20, 30, 40] {
            index.register(entity, InternalEntity::new(entity as u32, 0));
        }
        index.move_to(10, EntityLocation::new(first, 0));
        index.move_to(20, EntityLocation::new(second, 0));
        index.move_to(30, EntityLocation::new(first, 1));
        index.move_to(40, EntityLocation::new(second, 1));
        index.move_to(10, EntityLocation::new(second, 2));
        index.update_row(30, 0);
        index.remove(30);

        let mut visited = Vec::new();
        index.visit_matching(&[first, second], |location| {
            visited.push((location.stable_id, location.location.table_row))
        });

        assert_eq!(visited, vec![(10, 2), (20, 0), (40, 1)]);
    }

    #[test]
    fn rebuild_discards_old_membership_and_restores_the_supplied_world_order() {
        let first = ArchetypeId::new(1);
        let second = ArchetypeId::new(2);
        let mut index = StableQueryOrderIndex::default();
        index.register(1, InternalEntity::new(1, 0));
        index.move_to(1, EntityLocation::new(first, 0));

        index.rebuild([
            (30, InternalEntity::new(30, 0)),
            (10, InternalEntity::new(10, 0)),
            (20, InternalEntity::new(20, 0)),
        ]);
        index.move_to(30, EntityLocation::new(second, 0));
        index.move_to(10, EntityLocation::new(first, 0));
        index.move_to(20, EntityLocation::new(second, 1));

        let mut visited = Vec::new();
        index.visit_matching(&[second, first], |location| {
            visited.push(location.stable_id)
        });

        assert_eq!(visited, vec![30, 10, 20]);
    }
}
