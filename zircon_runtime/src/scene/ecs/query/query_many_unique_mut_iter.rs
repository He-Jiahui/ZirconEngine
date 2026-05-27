use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, QueryEntityItem, QueryFilter, QueryMutData};
use crate::scene::{EntityId, World};

pub struct QueryManyUniqueMutIter<'world, D, F = (), I = std::vec::IntoIter<EntityId>>
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

impl<'world, D, F, I> QueryManyUniqueMutIter<'world, D, F, I>
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

    fn matches_entity(&self, entity: EntityId) -> bool {
        let world = unsafe { &*self.world };
        world.contains_entity(entity)
            && self.cached_entities.contains(&entity)
            && D::matches_data(world, entity)
            && F::matches(world, entity, self.ticks)
    }
}

impl<'world, D, F, I> Iterator for QueryManyUniqueMutIter<'world, D, F, I>
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
