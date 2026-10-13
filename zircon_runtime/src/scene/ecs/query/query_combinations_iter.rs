use std::{array, marker::PhantomData};

use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryData, QueryFilter, StableEntityLocation,
};
use crate::scene::{EntityId, World};

use super::query_state::{find_cached_archetype_plan, CachedArchetypePlan};

/// Read-only K-combination iterator over a stable snapshot of matching scene entities.
pub struct QueryCombinationIter<'world, 'state, D, F = (), const K: usize = 2>
where
    D: QueryData,
    F: QueryFilter,
{
    // The API constructor binds this raw origin to the genuine loan/run lifetime.
    world: *const World,
    candidates: QueryCombinationCandidates<'state>,
    // Lexicographic entity-list positions for the next combination to fetch.
    indices: [usize; K],
    remaining: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world World, fn() -> (D, F))>,
}

enum QueryCombinationCandidates<'state> {
    Owned(Vec<EntityId>),
    Cached {
        plans: &'state [CachedArchetypePlan],
        stable_locations: Vec<StableEntityLocation>,
    },
}

impl QueryCombinationCandidates<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Owned(entities) => entities.len(),
            Self::Cached {
                stable_locations, ..
            } => stable_locations.len(),
        }
    }
}

impl<'world, 'state, D, F, const K: usize> QueryCombinationIter<'world, 'state, D, F, K>
where
    D: QueryData,
    F: QueryFilter,
{
    pub(crate) unsafe fn new(world: *const World, ticks: ChangeTickWindow) -> Self {
        unsafe {
            assert!(K != 0, "query combinations require K greater than zero");
            let mut matched_entities = Vec::new();
            for entity in World::query_entity_ids(world) {
                if read_only_combination_candidate_matches::<D, F>(&*world, entity, ticks) {
                    matched_entities.push(entity);
                }
            }
            if matched_entities.len() < K {
                return Self::empty(world, ticks);
            }
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
    }

    unsafe fn empty(world: *const World, ticks: ChangeTickWindow) -> Self {
        unsafe {
            Self {
                world,
                candidates: QueryCombinationCandidates::Owned(Vec::new()),
                indices: array::from_fn(|index| index),
                remaining: 0,
                ticks,
                _marker: PhantomData,
            }
        }
    }

    pub(crate) unsafe fn new_from_cached_plans(
        world: *const World,
        plans: &'state [CachedArchetypePlan],
        ticks: ChangeTickWindow,
    ) -> Self {
        unsafe {
            assert!(K != 0, "query combinations require K greater than zero");
            let mut stable_locations = Vec::new();
            let mut component_locations = Vec::new();
            for stable_location in World::query_stable_location_iter(
                world,
                plans.iter().map(CachedArchetypePlan::archetype_id),
            ) {
                let Some(plan) =
                    find_cached_archetype_plan(plans, stable_location.location.archetype_id)
                else {
                    continue;
                };
                if plan.write_component_locations(
                    &*world,
                    stable_location,
                    &mut component_locations,
                ) && F::matches_component_locations(
                    &*world,
                    stable_location.stable_id,
                    &component_locations,
                    ticks,
                ) {
                    stable_locations.push(stable_location);
                }
            }
            if stable_locations.len() < K {
                return Self::empty(world, ticks);
            }
            let candidates = QueryCombinationCandidates::Cached {
                plans,
                stable_locations,
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
    }

    fn fetch_current(&self) -> [D::Item<'world>; K] {
        // SAFETY: the cursor grant and provider contracts preserve all returned leaves.
        unsafe {
            array::from_fn(|index| {
                let candidate_index = self.indices[index];
                match &self.candidates {
                    QueryCombinationCandidates::Owned(entities) => {
                        let entity = entities[candidate_index];
                        D::fetch_with_ticks(self.world, entity, self.ticks)
                            .expect("combination entity should still match query data")
                    }
                    QueryCombinationCandidates::Cached {
                        plans,
                        stable_locations,
                    } => {
                        let stable_location = stable_locations[candidate_index];
                        let entity = stable_location.stable_id;
                        let plan = find_cached_archetype_plan(
                            plans,
                            stable_location.location.archetype_id,
                        )
                        .expect("cached combination location must retain an archetype plan");
                        let mut component_locations = Vec::new();
                        assert!(plan.write_component_locations(
                            &*self.world,
                            stable_location,
                            &mut component_locations,
                        ));
                        D::fetch_with_component_locations(
                            self.world,
                            entity,
                            stable_location,
                            &component_locations,
                            self.ticks,
                        )
                        .expect("cached combination entity should still match query data")
                    }
                }
            })
        }
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
        // SAFETY: the cursor's original grant keeps storage/structure and
        // declared compatible leaves valid; no World parent is retained.
        unsafe {
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
        // The product may exceed usize even when the binomial coefficient fits.
        // Both factors fit usize, so u128 holds their product on supported targets.
        let next_count = (count as u128 * numerator as u128) / denominator as u128;
        let Ok(next_count) = usize::try_from(next_count) else {
            return usize::MAX;
        };
        count = next_count;
        denominator += 1;
    }
    count
}

#[cfg(test)]
mod count_tests {
    use super::combination_count;

    #[test]
    fn intermediate_product_does_not_saturate_representable_counts() {
        assert_eq!(combination_count(33, 15), 1_037_158_320);
        assert_eq!(combination_count(33, 18), 1_037_158_320);
        #[cfg(target_pointer_width = "64")]
        assert_eq!(combination_count(67, 30), 9_989_690_752_182_277_136);
    }

    #[test]
    fn combinations_cover_zero_impossible_and_overflowing_counts() {
        assert_eq!(combination_count(0, 0), 1);
        assert_eq!(combination_count(4, 0), 1);
        assert_eq!(combination_count(4, 4), 1);
        assert_eq!(combination_count(4, 5), 0);
        assert_eq!(combination_count(usize::MAX, 1), usize::MAX);
        assert_eq!(combination_count(usize::MAX, 2), usize::MAX);
    }
}
