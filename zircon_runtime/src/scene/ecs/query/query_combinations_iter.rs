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
    pub(crate) fn new<EntityList>(
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<Item = EntityId>,
    {
        assert!(K != 0, "query combinations require K greater than zero");
        let entities = entities
            .into_iter()
            .filter(|entity| D::matches_data(world, *entity) && F::matches(world, *entity, ticks))
            .collect::<Vec<_>>();
        let candidates = QueryCombinationCandidates::Owned(entities);
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

    pub(crate) fn new_from_cached_entities(
        world: &'world World,
        entities: &'state [EntityId],
        stable_locations: &'state [StableEntityLocation],
        component_locations: &'state [ComponentStorageLocation],
        component_location_offsets: &'state [usize],
        ticks: ChangeTickWindow,
    ) -> Self {
        assert!(K != 0, "query combinations require K greater than zero");
        let mut cache_indices = Vec::with_capacity(entities.len());
        for (index, entity) in entities.iter().copied().enumerate() {
            if stable_locations.get(index).is_none() {
                continue;
            }
            let Some(entity_component_locations) = cached_query_component_locations(
                component_locations,
                component_location_offsets,
                index,
            ) else {
                continue;
            };
            if F::matches_component_locations(world, entity, entity_component_locations, ticks) {
                cache_indices.push(index);
            }
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

pub(crate) fn combination_count(entity_count: usize, group_size: usize) -> usize {
    if group_size > entity_count {
        return 0;
    }
    let group_size = group_size.min(entity_count - group_size);
    let numerator = (entity_count - group_size + 1..=entity_count).rev();
    (1..=group_size)
        .zip(numerator)
        .try_fold(1_usize, |accumulator, (denominator, numerator)| {
            Some(accumulator.checked_mul(numerator)? / denominator)
        })
        .unwrap_or(usize::MAX)
}
