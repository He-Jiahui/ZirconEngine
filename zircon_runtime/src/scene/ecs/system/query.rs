use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, QueryData, QueryFilter, QueryIter, QueryMutData};
use crate::scene::World;

pub struct Query<'world, D, F = ()> {
    world: *mut World,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, D, F> Query<'world, D, F> {
    pub(crate) fn new(world: *mut World, ticks: ChangeTickWindow) -> Self {
        Self {
            world,
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<D, F> Query<'_, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub fn iter(&self) -> QueryIter<'_, D, F> {
        let world = unsafe { &*self.world };
        QueryIter::new(world, world.entity_ids_for_query(), self.ticks)
    }
}

impl<D, F> Query<'_, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub fn for_each_mut(&mut self, mut f: impl FnMut(D::Item<'_>)) {
        let world = unsafe { &mut *self.world };
        let entities = world.entity_ids_for_query().to_vec();
        for entity in entities {
            if F::matches(world, entity, self.ticks) && D::matches_data(world, entity) {
                if let Some(item) = D::fetch_mut(world, entity) {
                    f(item);
                }
            }
        }
    }
}
