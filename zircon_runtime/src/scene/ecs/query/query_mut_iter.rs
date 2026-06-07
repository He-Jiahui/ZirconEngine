use std::marker::PhantomData;

use super::cached_query_iter::cached_query_component_locations;
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryFilter, QueryMutData, QueryState,
};
use crate::scene::{EntityId, World};

/// Mutable full-query iterator over a cached, unique structural candidate list.
pub struct QueryMutIter<'world, 'state, D, F = ()>
where
    D: QueryMutData,
    F: QueryFilter,
{
    world: *mut World,
    entities: &'state [EntityId],
    component_locations: &'state [ComponentStorageLocation],
    component_location_offsets: &'state [usize],
    index: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, 'state, D, F> QueryMutIter<'world, 'state, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub(crate) fn new(
        world: &'world mut World,
        entities: &'state [EntityId],
        component_locations: &'state [ComponentStorageLocation],
        component_location_offsets: &'state [usize],
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            entities,
            component_locations,
            component_location_offsets,
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }

    fn matches_entity(&self, entity: EntityId, index: usize) -> bool {
        let world = unsafe { &*self.world };
        let Some(component_locations) = cached_query_component_locations(
            self.component_locations,
            self.component_location_offsets,
            index,
        ) else {
            return false;
        };
        F::matches_component_locations(world, entity, component_locations, self.ticks)
    }
}

impl<'world, 'state, D, F> Iterator for QueryMutIter<'world, 'state, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.get(self.index).copied() {
            let index = self.index;
            self.index += 1;
            if self.matches_entity(entity, index) {
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
    pub fn iter_mut<'world, 'state>(
        &'state mut self,
        world: &'world mut World,
    ) -> QueryMutIter<'world, 'state, D, F> {
        self.iter_mut_with_ticks(world, ChangeTickWindow::all(world.read_change_tick()))
    }

    pub(crate) fn iter_mut_with_ticks<'world, 'state>(
        &'state mut self,
        world: &'world mut World,
        ticks: ChangeTickWindow,
    ) -> QueryMutIter<'world, 'state, D, F> {
        self.update_cache(world);
        QueryMutIter::new(
            world,
            self.cached_entities(),
            self.cached_component_locations(),
            self.cached_component_location_offsets(),
            ticks,
        )
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
