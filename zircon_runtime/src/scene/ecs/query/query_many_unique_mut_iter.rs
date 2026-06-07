use std::marker::PhantomData;

use super::cached_query_iter::{cached_query_component_locations, cached_query_entity_index};
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryEntityItem, QueryFilter, QueryMutData,
};
use crate::scene::{EntityId, World};

pub struct QueryManyUniqueMutIter<'world, 'state, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: *mut World,
    cached_entity_indices: &'state [(EntityId, usize)],
    cached_component_locations: &'state [ComponentStorageLocation],
    cached_component_location_offsets: &'state [usize],
    entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, 'state, D, F, I> QueryManyUniqueMutIter<'world, 'state, D, F, I>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) fn new<EntityList>(
        world: &'world mut World,
        cached_entity_indices: &'state [(EntityId, usize)],
        cached_component_locations: &'state [ComponentStorageLocation],
        cached_component_location_offsets: &'state [usize],
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        Self {
            world,
            cached_entity_indices,
            cached_component_locations,
            cached_component_location_offsets,
            entities: entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }

    fn matches_entity(&self, entity: EntityId) -> bool {
        let world = unsafe { &*self.world };
        let Some(index) = cached_query_entity_index(self.cached_entity_indices, entity) else {
            return false;
        };
        let Some(component_locations) = cached_query_component_locations(
            self.cached_component_locations,
            self.cached_component_location_offsets,
            index,
        ) else {
            return false;
        };
        F::matches_component_locations(world, entity, component_locations, self.ticks)
    }
}

impl<'world, 'state, D, F, I> Iterator for QueryManyUniqueMutIter<'world, 'state, D, F, I>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity_item) = self.entities.next() {
            let entity = entity_item.entity_id();
            if self.matches_entity(entity) {
                // Unique entity input prevents mutable aliases across yielded items.
                return unsafe { fetch_unique_mut_unchecked::<D>(self.world, entity, self.ticks) };
            }
        }
        None
    }
}

unsafe fn fetch_unique_mut_unchecked<'world, D>(
    world: *mut World,
    entity: EntityId,
    ticks: ChangeTickWindow,
) -> Option<D::Item<'world>>
where
    D: QueryMutData,
{
    D::fetch_mut_with_ticks(unsafe { &mut *world }, entity, ticks)
}
