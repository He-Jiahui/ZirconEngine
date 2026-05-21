use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, QueryData, QueryFilter};
use crate::scene::{EntityId, World};

pub trait QueryEntityItem {
    fn entity_id(self) -> EntityId;
}

impl QueryEntityItem for EntityId {
    fn entity_id(self) -> EntityId {
        self
    }
}

impl QueryEntityItem for &EntityId {
    fn entity_id(self) -> EntityId {
        *self
    }
}

pub struct QueryManyIter<'world, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: &'world World,
    entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, D, F, I> QueryManyIter<'world, D, F, I>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) fn new<EntityList>(
        world: &'world World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        Self {
            world,
            entities: entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<'world, D, F, I> Iterator for QueryManyIter<'world, D, F, I>
where
    D: QueryData,
    F: QueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        for entity_item in self.entities.by_ref() {
            let entity = entity_item.entity_id();
            if world_entity_matches::<D, F>(self.world, entity, self.ticks) {
                if let Some(item) = D::fetch_with_ticks(self.world, entity, self.ticks) {
                    return Some(item);
                }
            }
        }
        None
    }
}

fn world_entity_matches<D, F>(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool
where
    D: QueryData,
    F: QueryFilter,
{
    world.contains_entity(entity)
        && F::matches(world, entity, ticks)
        && D::matches_data(world, entity)
}
