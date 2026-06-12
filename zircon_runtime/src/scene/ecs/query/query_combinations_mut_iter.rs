use std::{array, marker::PhantomData};

use crate::scene::ecs::{ChangeTickWindow, ComponentStorageLocation, QueryFilter, QueryMutData};
use crate::scene::{EntityId, World};

use super::cached_query_iter::cached_query_component_locations;
use super::query_combinations_iter::combination_count;

/// Mutable K-combination cursor. Items are produced only through `fetch_next`.
pub struct QueryCombinationMutIter<'world, 'state, D, F = (), const K: usize = 2>
where
    D: QueryMutData,
    F: QueryFilter,
{
    world: *mut World,
    candidates: QueryCombinationMutCandidates<'state>,
    // Lexicographic entity-list positions for the next combination to fetch.
    indices: [usize; K],
    remaining: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, &'state [EntityId], fn() -> (D, F))>,
}

struct QueryCombinationMutCandidates<'state> {
    entities: &'state [EntityId],
    cache_indices: Vec<usize>,
}

impl QueryCombinationMutCandidates<'_> {
    fn len(&self) -> usize {
        self.cache_indices.len()
    }

    fn entity(&self, candidate_index: usize) -> EntityId {
        self.entities[self.cache_indices[candidate_index]]
    }
}

impl<'world, 'state, D, F, const K: usize> QueryCombinationMutIter<'world, 'state, D, F, K>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub(crate) fn new_from_cached_entities(
        world: &'world mut World,
        entities: &'state [EntityId],
        component_locations: &'state [ComponentStorageLocation],
        component_location_offsets: &'state [usize],
        ticks: ChangeTickWindow,
    ) -> Self {
        assert!(K != 0, "query combinations require K greater than zero");
        if K > entities.len() {
            return Self::empty(world, ticks);
        }
        let cache_indices = {
            let world = &*world;
            let mut cache_indices = Vec::with_capacity(entities.len());
            let mut index = 0_usize;
            while index < entities.len() {
                let entity = entities[index];
                if let Some(component_locations) = cached_query_component_locations(
                    component_locations,
                    component_location_offsets,
                    index,
                ) {
                    if F::matches_component_locations(world, entity, component_locations, ticks) {
                        cache_indices.push(index);
                    }
                }
                index += 1;
            }
            cache_indices
        };
        if cache_indices.len() < K {
            return Self::empty(world, ticks);
        }
        let candidates = QueryCombinationMutCandidates {
            entities,
            cache_indices,
        };
        let remaining = combination_count(candidates.len(), K);
        Self {
            world,
            candidates,
            indices: array::from_fn(|index| index),
            remaining,
            ticks,
            _marker: PhantomData,
        }
    }

    fn empty(world: &'world mut World, ticks: ChangeTickWindow) -> Self {
        Self {
            world,
            candidates: QueryCombinationMutCandidates {
                entities: &[],
                cache_indices: Vec::new(),
            },
            indices: array::from_fn(|index| index),
            remaining: 0,
            ticks,
            _marker: PhantomData,
        }
    }

    pub fn fetch_next(&mut self) -> Option<[D::Item<'_>; K]> {
        if self.remaining == 0 {
            return None;
        }

        let entities = self.current_entities();
        self.remaining -= 1;
        if self.remaining > 0 {
            self.advance_indices();
        }

        Some(array::from_fn(|index| {
            let entity = entities[index];
            // The stored combination indices are distinct, so every mutable
            // item in this array is fetched from a different stable entity.
            unsafe { fetch_combination_mut_unchecked::<D>(self.world, entity, self.ticks) }
                .expect("combination entity should still match mutable query data")
        }))
    }

    pub fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn current_entities(&self) -> [EntityId; K] {
        array::from_fn(|index| self.candidates.entity(self.indices[index]))
    }

    fn advance_indices(&mut self) {
        let entity_count = self.candidates.len();
        for index in (0..K).rev() {
            let max = entity_count - K + index;
            if self.indices[index] < max {
                self.indices[index] += 1;
                for next in (index + 1)..K {
                    self.indices[next] = self.indices[next - 1] + 1;
                }
                return;
            }
        }
    }
}

unsafe fn fetch_combination_mut_unchecked<'world, D>(
    world: *mut World,
    entity: EntityId,
    ticks: ChangeTickWindow,
) -> Option<D::Item<'world>>
where
    D: QueryMutData,
{
    D::fetch_mut_with_ticks(unsafe { &mut *world }, entity, ticks)
}
