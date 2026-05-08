use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, QueryData, QueryFilter};
use crate::scene::{EntityId, World};

pub struct QueryIter<'world, D, F = ()>
where
    D: QueryData,
    F: QueryFilter,
{
    world: &'world World,
    entities: &'world [EntityId],
    index: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, D, F> QueryIter<'world, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub(crate) fn new(
        world: &'world World,
        entities: &'world [EntityId],
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            entities,
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<'world, D, F> Iterator for QueryIter<'world, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.get(self.index).copied() {
            self.index += 1;
            if F::matches(self.world, entity, self.ticks) && D::matches_data(self.world, entity) {
                if let Some(item) = D::fetch(self.world, entity) {
                    return Some(item);
                }
            }
        }
        None
    }
}
