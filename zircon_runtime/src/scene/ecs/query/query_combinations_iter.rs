use std::{array, marker::PhantomData};

use crate::scene::ecs::{ChangeTickWindow, QueryData, QueryFilter};
use crate::scene::{EntityId, World};

/// Read-only K-combination iterator over a stable snapshot of matching scene entities.
pub struct QueryCombinationIter<'world, D, F = (), const K: usize = 2>
where
    D: QueryData,
    F: QueryFilter,
{
    world: &'world World,
    entities: Vec<EntityId>,
    // Lexicographic entity-list positions for the next combination to fetch.
    indices: [usize; K],
    remaining: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, D, F, const K: usize> QueryCombinationIter<'world, D, F, K>
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

    fn fetch_current(&self) -> [D::Item<'world>; K] {
        array::from_fn(|index| {
            let entity = self.entities[self.indices[index]];
            D::fetch_with_ticks(self.world, entity, self.ticks)
                .expect("combination entity should still match query data")
        })
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

impl<'world, D, F, const K: usize> Iterator for QueryCombinationIter<'world, D, F, K>
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

impl<'world, D, F, const K: usize> ExactSizeIterator for QueryCombinationIter<'world, D, F, K>
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
