use std::{array, marker::PhantomData};

use crate::scene::ecs::{ChangeTickWindow, QueryFilter, QueryMutData};
use crate::scene::{EntityId, World};

use super::query_combinations_iter::combination_count;

/// Mutable K-combination cursor. Items are produced only through `fetch_next`.
pub struct QueryCombinationMutIter<'world, D, F = (), const K: usize = 2>
where
    D: QueryMutData,
    F: QueryFilter,
{
    world: *mut World,
    entities: Vec<EntityId>,
    // Lexicographic entity-list positions for the next combination to fetch.
    indices: [usize; K],
    remaining: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, D, F, const K: usize> QueryCombinationMutIter<'world, D, F, K>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub(crate) fn new<EntityList>(
        world: &'world mut World,
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
        let remaining = combination_count(entities.len(), K);
        Self {
            world,
            entities,
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
        array::from_fn(|index| self.entities[self.indices[index]])
    }

    fn advance_indices(&mut self) {
        let entity_count = self.entities.len();
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
