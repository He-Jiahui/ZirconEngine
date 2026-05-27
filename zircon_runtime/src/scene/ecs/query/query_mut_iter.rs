use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, QueryFilter, QueryMutData, QueryState};
use crate::scene::{EntityId, World};

/// Mutable full-query iterator over a cached, unique structural candidate list.
pub struct QueryMutIter<'world, D, F = ()>
where
    D: QueryMutData,
    F: QueryFilter,
{
    world: *mut World,
    entities: std::vec::IntoIter<EntityId>,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, D, F> QueryMutIter<'world, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub(crate) fn new(
        world: &'world mut World,
        entities: Vec<EntityId>,
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            entities: entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }

    fn matches_entity(&self, entity: EntityId) -> bool {
        let world = unsafe { &*self.world };
        world.contains_entity(entity)
            && D::matches_data(world, entity)
            && F::matches(world, entity, self.ticks)
    }
}

impl<'world, D, F> Iterator for QueryMutIter<'world, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.next() {
            if self.matches_entity(entity) {
                // QueryState cache candidates are unique entity ids, so yielded
                // mutable items cannot alias each other across iterator steps.
                return unsafe { fetch_mut_unchecked::<D>(self.world, entity, self.ticks) };
            }
        }
        None
    }
}

impl<D, F> QueryState<D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub fn iter_mut<'world>(&mut self, world: &'world mut World) -> QueryMutIter<'world, D, F> {
        self.iter_mut_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub(crate) fn iter_mut_with_ticks<'world>(
        &mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> QueryMutIter<'world, D, F> {
        self.update_cache(world);
        let entities = self
            .cached_locations()
            .iter()
            .map(|location| location.stable_id)
            .collect::<Vec<_>>();
        QueryMutIter::new(world, entities, ticks)
    }
}

unsafe fn fetch_mut_unchecked<'world, D>(
    world: *mut World,
    entity: EntityId,
    ticks: ChangeTickWindow,
) -> Option<D::Item<'world>>
where
    D: QueryMutData,
{
    D::fetch_mut_with_ticks(unsafe { &mut *world }, entity, ticks)
}
