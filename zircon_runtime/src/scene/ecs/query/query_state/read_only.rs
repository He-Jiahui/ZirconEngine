use std::array;

use crate::scene::ecs::{
    ChangeTickWindow, QueryCombinationIter, QueryData, QueryEntityError, QueryEntityItem,
    QueryFilter, QueryIter, QueryManyIter, QuerySingleError, UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::super::single_from_iter;
use super::many_item_array::collect_many_query_items;
use super::QueryState;

impl<D, F> QueryState<D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    // 非缓存入口直接扫描世界稳定实体序列，适合一次性读取而不保留 archetype 计划。
    pub fn iter<'world>(&self, world: &'world World) -> QueryIter<'world, 'world, D, F> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            QueryIter::new(world as *const World, ticks)
        }
    }

    pub fn single<'world>(&self, world: &'world World) -> Result<D::Item<'world>, QuerySingleError>
    where
        D: 'world,
    {
        single_from_iter(self.iter(world))
    }

    pub fn iter_many<'world, EntityList>(
        &self,
        world: &'world World,
        entities: EntityList,
    ) -> QueryManyIter<'world, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_many_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn iter_many_unique<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyIter<'world, D, F, array::IntoIter<EntityId, N>> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_many_unique_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn iter_combinations<'world, 'state, const K: usize>(
        &'state self,
        world: &'world World,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_combinations_with_ticks(world as *const World, ticks)
        }
    }

    pub fn get<'world>(
        &self,
        world: &'world World,
        entity: EntityId,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_with_ticks(world as *const World, entity, ticks)
        }
    }

    pub fn get_many<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_many_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn get_many_unique<'world, const N: usize>(
        &self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_many_unique_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn is_empty(&self, world: &World) -> bool {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.is_empty_with_ticks(world as *const World, ticks)
        }
    }

    pub fn count(&self, world: &World) -> usize {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.count_with_ticks(world as *const World, ticks)
        }
    }

    pub fn contains(&self, world: &World, entity: EntityId) -> bool {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.contains_with_ticks(world as *const World, entity, ticks)
        }
    }

    pub(crate) unsafe fn contains_with_ticks(
        &self,
        world: *const World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> bool {
        unsafe {
            (&*world).contains_entity(entity)
                && D::matches_data(&*world, entity)
                && F::matches(&*world, entity, ticks)
        }
    }

    pub(crate) unsafe fn is_empty_with_ticks(
        &self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> bool {
        unsafe {
            for entity in World::query_entity_ids(world) {
                if !F::matches(&*world, entity, ticks) || !D::matches_data(&*world, entity) {
                    continue;
                }
                if D::fetch_with_ticks(world, entity, ticks).is_some() {
                    return false;
                }
            }
            true
        }
    }

    pub(crate) unsafe fn count_with_ticks(
        &self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> usize {
        unsafe {
            let mut count = 0_usize;
            for entity in World::query_entity_ids(world) {
                if !F::matches(&*world, entity, ticks) || !D::matches_data(&*world, entity) {
                    continue;
                }
                if D::fetch_with_ticks(world, entity, ticks).is_some() {
                    count += 1;
                }
            }
            count
        }
    }

    pub(crate) unsafe fn get_with_ticks<'world>(
        &self,
        world: *const World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        unsafe {
            if !(&*world).contains_entity(entity) {
                return Err(QueryEntityError::NotSpawned(entity));
            }
            if !D::matches_data(&*world, entity) || !F::matches(&*world, entity, ticks) {
                return Err(QueryEntityError::QueryDoesNotMatch(entity));
            }
            let Some(item) = D::fetch_with_ticks(world, entity, ticks) else {
                return Err(QueryEntityError::QueryDoesNotMatch(entity));
            };
            Ok(item)
        }
    }

    pub(crate) unsafe fn get_many_with_ticks<'world, const N: usize>(
        &self,
        world: *const World,
        entities: [EntityId; N],
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        unsafe {
            collect_many_query_items(entities, |entity| self.get_with_ticks(world, entity, ticks))
        }
    }

    pub(crate) unsafe fn get_many_unique_with_ticks<'world, const N: usize>(
        &self,
        world: *const World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        unsafe { self.get_many_with_ticks(world, entities.into_inner(), ticks) }
    }

    pub(crate) unsafe fn iter_many_unique_with_ticks<'world, const N: usize>(
        &self,
        world: *const World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyIter<'world, D, F, array::IntoIter<EntityId, N>> {
        unsafe { self.iter_many_with_ticks(world, entities, ticks) }
    }

    pub(crate) unsafe fn iter_many_with_ticks<'world, EntityList>(
        &self,
        world: *const World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> QueryManyIter<'world, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        unsafe { QueryManyIter::new(world, entities, ticks) }
    }

    pub(crate) unsafe fn iter_combinations_with_ticks<'world, 'state, const K: usize>(
        &'state self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        unsafe { QueryCombinationIter::new(world, ticks) }
    }
}
