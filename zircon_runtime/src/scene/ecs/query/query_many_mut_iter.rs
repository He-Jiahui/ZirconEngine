use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, QueryEntityItem, QueryFilter, QueryMutData};
use crate::scene::{EntityId, World};

pub struct QueryManyMutIter<'world, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: *mut World,
    cached_entities: Vec<EntityId>,
    entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, fn() -> (D, F))>,
}

impl<'world, D, F, I> QueryManyMutIter<'world, D, F, I>
where
    D: QueryMutData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) fn new<EntityList>(
        world: &'world mut World,
        cached_entities: Vec<EntityId>,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        Self {
            world,
            cached_entities,
            entities: entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }

    pub fn fetch_next(&mut self) -> Option<D::Item<'_>> {
        while let Some(entity_item) = self.entities.next() {
            let entity = entity_item.entity_id();
            if self.matches_entity(entity) {
                return unsafe { D::fetch_mut_with_ticks(&mut *self.world, entity, self.ticks) };
            }
        }
        None
    }

    fn matches_entity(&self, entity: EntityId) -> bool {
        let world = unsafe { &*self.world };
        world.contains_entity(entity)
            && self.cached_entities.contains(&entity)
            && D::matches_data(world, entity)
            && F::matches(world, entity, self.ticks)
    }
}
