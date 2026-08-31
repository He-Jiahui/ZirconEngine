use std::marker::PhantomData;

use super::query_state::{CachedArchetypePlan, project_entity_from_plans};
use crate::scene::ecs::{
    ChangeTickWindow, ComponentStorageLocation, QueryEntityItem, QueryFilter, QueryMutData,
};
use crate::scene::{EntityId, World};

pub struct QueryManyMutIter<'world, 'state, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: *mut World,
    plans: &'state [CachedArchetypePlan],
    component_locations: Vec<ComponentStorageLocation>,
    entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, 'state, D, F, I> QueryManyMutIter<'world, 'state, D, F, I>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) fn new<EntityList>(
        world: &'world mut World,
        plans: &'state [CachedArchetypePlan],
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        Self {
            world,
            plans,
            component_locations: Vec::new(),
            entities: entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }

    pub fn fetch_next(&mut self) -> Option<D::Item<'_>> {
        while let Some(entity_item) = self.entities.next() {
            let entity = entity_item.entity_id();
            let world = unsafe { &*self.world };
            if project_entity_from_plans(self.plans, world, entity, &mut self.component_locations)
                .is_none()
                || !F::matches_component_locations(
                    world,
                    entity,
                    &self.component_locations,
                    self.ticks,
                )
            {
                continue;
            }
            return unsafe {
                D::fetch_mut_with_component_locations(
                    &mut *self.world,
                    entity,
                    &self.component_locations,
                    self.ticks,
                )
            };
        }
        None
    }
}
