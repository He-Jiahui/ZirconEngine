use std::{array, marker::PhantomData};

use crate::scene::World;
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryFilter, QueryMutData, StableEntityLocation,
};

use super::query_combinations_iter::combination_count;
use super::query_state::{CachedArchetypePlan, find_cached_archetype_plan};

/// Mutable K-combination cursor. Items are produced only through `fetch_next`.
pub struct QueryCombinationMutIter<'world, 'state, D, F = (), const K: usize = 2>
where
    D: QueryMutData,
    F: QueryFilter,
{
    world: *mut World,
    plans: &'state [CachedArchetypePlan],
    candidates: Vec<StableEntityLocation>,
    indices: [usize; K],
    remaining: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, 'state, D, F, const K: usize> QueryCombinationMutIter<'world, 'state, D, F, K>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub(crate) fn new_from_cached_plans(
        world: &'world mut World,
        plans: &'state [CachedArchetypePlan],
        ticks: ChangeTickWindow,
    ) -> Self {
        assert!(K != 0, "query combinations require K greater than zero");
        let mut candidates = Vec::new();
        let mut component_locations = Vec::new();
        for stable_location in
            world.stable_query_location_iter(plans.iter().map(CachedArchetypePlan::archetype_id))
        {
            let Some(plan) =
                find_cached_archetype_plan(plans, stable_location.location.archetype_id)
            else {
                continue;
            };
            if plan.write_component_locations(world, stable_location, &mut component_locations)
                && F::matches_component_locations(
                    world,
                    stable_location.stable_id,
                    &component_locations,
                    ticks,
                )
            {
                candidates.push(stable_location);
            }
        }
        let remaining = combination_count(candidates.len(), K);
        Self {
            world,
            plans,
            candidates,
            indices: array::from_fn(|index| index),
            remaining,
            ticks,
            _marker: PhantomData,
        }
    }

    pub fn fetch_next(&mut self) -> Option<[D::Item<'_>; K]> {
        if self.remaining == 0 {
            return None;
        }

        let locations: [StableEntityLocation; K] =
            array::from_fn(|index| self.candidates[self.indices[index]]);
        self.remaining -= 1;
        if self.remaining > 0 {
            self.advance_indices();
        }

        let items: [D::Item<'_>; K] = array::from_fn(|index| {
            let stable_location = locations[index];
            let plan =
                find_cached_archetype_plan(self.plans, stable_location.location.archetype_id)
                    .expect("mutable combination location must retain an archetype plan");
            let mut component_locations = Vec::<ComponentStorageLocation>::new();
            assert!(plan.write_component_locations(
                unsafe { &*self.world },
                stable_location,
                &mut component_locations,
            ));
            // Combination indices are distinct, so every mutable item belongs
            // to a different stable entity.
            unsafe {
                D::fetch_mut_with_component_locations(
                    &mut *self.world,
                    stable_location.stable_id,
                    &component_locations,
                    self.ticks,
                )
            }
            .expect("combination entity should still match mutable query data")
        });
        Some(items)
    }

    pub fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
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
