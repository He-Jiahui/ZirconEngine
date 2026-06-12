use std::{array, marker::PhantomData};

use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryData, QueryFilter, StableEntityLocation,
};
use crate::scene::{EntityId, World};

use super::cached_query_iter::cached_query_component_locations;

/// Read-only K-combination iterator over a stable snapshot of matching scene entities.
pub struct QueryCombinationIter<'world, 'state, D, F = (), const K: usize = 2>
where
    D: QueryData,
    F: QueryFilter,
{
    world: &'world World,
    candidates: QueryCombinationCandidates<'state>,
    // Lexicographic entity-list positions for the next combination to fetch.
    indices: [usize; K],
    remaining: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

enum QueryCombinationCandidates<'state> {
    Owned(Vec<EntityId>),
    Cached {
        entities: &'state [EntityId],
        stable_locations: &'state [StableEntityLocation],
        component_locations: &'state [ComponentStorageLocation],
        component_location_offsets: &'state [usize],
        cache_indices: Vec<usize>,
    },
}

impl QueryCombinationCandidates<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Owned(entities) => entities.len(),
            Self::Cached { cache_indices, .. } => cache_indices.len(),
        }
    }
}

impl<'world, 'state, D, F, const K: usize> QueryCombinationIter<'world, 'state, D, F, K>
where
    D: QueryData,
    F: QueryFilter,
{
    pub(crate) fn new(
        world: &'world World,
        entities: &[EntityId],
        ticks: ChangeTickWindow,
    ) -> Self {
        assert!(K != 0, "query combinations require K greater than zero");
        if K > entities.len() {
            return Self::empty(world, ticks);
        }
        let candidate_count = read_only_combination_candidate_count::<D, F>(world, entities, ticks);
        if candidate_count < K {
            return Self::empty(world, ticks);
        }
        let mut matched_entities = Vec::with_capacity(candidate_count);
        for entity in entities.iter().copied() {
            if read_only_combination_candidate_matches::<D, F>(world, entity, ticks) {
                matched_entities.push(entity);
            }
        }
        debug_assert_eq!(matched_entities.len(), candidate_count);
        let candidates = QueryCombinationCandidates::Owned(matched_entities);
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

    fn empty(world: &'world World, ticks: ChangeTickWindow) -> Self {
        Self {
            world,
            candidates: QueryCombinationCandidates::Owned(Vec::new()),
            indices: array::from_fn(|index| index),
            remaining: 0,
            ticks,
            _marker: PhantomData,
        }
    }

    pub(crate) fn new_from_cached_entities(
        world: &'world World,
        entities: &'state [EntityId],
        stable_locations: &'state [StableEntityLocation],
        component_locations: &'state [ComponentStorageLocation],
        component_location_offsets: &'state [usize],
        ticks: ChangeTickWindow,
    ) -> Self {
        assert!(K != 0, "query combinations require K greater than zero");
        if K > entities.len() {
            return Self::empty(world, ticks);
        }
        let mut cache_indices = Vec::with_capacity(entities.len());
        let mut index = 0_usize;
        while index < entities.len() {
            let entity = entities[index];
            if stable_locations.get(index).is_some() {
                if let Some(entity_component_locations) = cached_query_component_locations(
                    component_locations,
                    component_location_offsets,
                    index,
                ) {
                    if F::matches_component_locations(
                        world,
                        entity,
                        entity_component_locations,
                        ticks,
                    ) {
                        cache_indices.push(index);
                    }
                }
            }
            index += 1;
        }
        if cache_indices.len() < K {
            return Self::empty(world, ticks);
        }
        let candidates = QueryCombinationCandidates::Cached {
            entities,
            stable_locations,
            component_locations,
            component_location_offsets,
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

    fn fetch_current(&self) -> [D::Item<'world>; K] {
        array::from_fn(|index| {
            let candidate_index = self.indices[index];
            match &self.candidates {
                QueryCombinationCandidates::Owned(entities) => {
                    let entity = entities[candidate_index];
                    D::fetch_with_ticks(self.world, entity, self.ticks)
                        .expect("combination entity should still match query data")
                }
                QueryCombinationCandidates::Cached {
                    entities,
                    stable_locations,
                    component_locations,
                    component_location_offsets,
                    cache_indices,
                } => {
                    let cache_index = cache_indices[candidate_index];
                    let entity = entities[cache_index];
                    let stable_location = stable_locations[cache_index];
                    let component_locations = cached_query_component_locations(
                        component_locations,
                        component_location_offsets,
                        cache_index,
                    )
                    .expect("cached combination component locations should stay index-aligned");
                    D::fetch_with_component_locations(
                        self.world,
                        entity,
                        stable_location,
                        component_locations,
                        self.ticks,
                    )
                    .expect("cached combination entity should still match query data")
                }
            }
        })
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

impl<'world, 'state, D, F, const K: usize> Iterator
    for QueryCombinationIter<'world, 'state, D, F, K>
where
    D: QueryData,
    F: QueryFilter,
{
    type Item = [D::Item<'world>; K];

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let items = self.fetch_current();
        self.remaining -= 1;
        if self.remaining > 0 {
            self.advance_indices();
        }
        Some(items)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<'world, 'state, D, F, const K: usize> ExactSizeIterator
    for QueryCombinationIter<'world, 'state, D, F, K>
where
    D: QueryData,
    F: QueryFilter,
{
}

fn read_only_combination_candidate_count<D, F>(
    world: &World,
    entities: &[EntityId],
    ticks: ChangeTickWindow,
) -> usize
where
    D: QueryData,
    F: QueryFilter,
{
    let mut count = 0_usize;
    for entity in entities.iter().copied() {
        if read_only_combination_candidate_matches::<D, F>(world, entity, ticks) {
            count += 1;
        }
    }
    count
}

fn read_only_combination_candidate_matches<D, F>(
    world: &World,
    entity: EntityId,
    ticks: ChangeTickWindow,
) -> bool
where
    D: QueryData,
    F: QueryFilter,
{
    D::matches_data(world, entity) && F::matches(world, entity, ticks)
}

pub(crate) fn combination_count(entity_count: usize, group_size: usize) -> usize {
    if group_size > entity_count {
        return 0;
    }
    let group_size = group_size.min(entity_count - group_size);
    let mut count = 1_usize;
    let mut denominator = 1;
    while denominator <= group_size {
        let numerator = entity_count - denominator + 1;
        let Some(next_count) = count.checked_mul(numerator) else {
            return usize::MAX;
        };
        count = next_count / denominator;
        denominator += 1;
    }
    count
}
