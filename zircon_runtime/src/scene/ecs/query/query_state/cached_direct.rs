use crate::scene::ecs::{
    CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter, ChangeTickWindow,
    ComponentStorageLocation, QueryEntityError, QueryEntityItem, QuerySingleError,
    UniqueEntityArray,
};
use crate::scene::EntityId;
use crate::scene::World;

use super::super::single_from_iter;
use super::many_item_array::collect_many_query_items;
use super::QueryState;

impl<D, F> QueryState<D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    // 这些入口先刷新 archetype 计划，再把稳定位置借给直接查询迭代器。
    pub fn iter_cached_direct<'world, 'state>(
        &'state mut self,
        world: &'world World,
    ) -> CachedQueryIter<'world, 'state, D, F> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_cached_direct_with_ticks(world as *const World, ticks)
        }
    }

    pub fn single_cached_direct<'world>(
        &mut self,
        world: &'world World,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        single_from_iter(self.iter_cached_direct(world))
    }

    pub fn iter_many_cached_direct<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world World,
        entities: EntityList,
    ) -> CachedQueryManyIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_many_cached_direct_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn iter_many_unique_cached_direct<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> CachedQueryManyIter<'world, 'state, D, F, std::array::IntoIter<EntityId, N>> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_many_unique_cached_direct_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn get_cached_direct<'world>(
        &mut self,
        world: &'world World,
        entity: EntityId,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_cached_direct_with_ticks(world as *const World, entity, ticks)
        }
    }

    pub fn get_many_cached_direct<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_many_cached_direct_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn get_many_unique_cached_direct<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_many_unique_cached_direct_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn is_empty_cached_direct(&mut self, world: &World) -> bool {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.is_empty_cached_direct_with_ticks(world as *const World, ticks)
        }
    }

    pub fn count_cached_direct(&mut self, world: &World) -> usize {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.count_cached_direct_with_ticks(world as *const World, ticks)
        }
    }

    pub fn contains_cached_direct(&mut self, world: &World, entity: EntityId) -> bool {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.contains_cached_direct_with_ticks(world as *const World, entity, ticks)
        }
    }

    pub(crate) unsafe fn iter_cached_direct_with_ticks<'world, 'state>(
        &'state mut self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> CachedQueryIter<'world, 'state, D, F> {
        unsafe {
            self.update_cache(&*world);
            CachedQueryIter::new(world, &self.cached_archetype_plans, ticks)
        }
    }

    pub(crate) unsafe fn iter_many_cached_direct_with_ticks<'world, 'state, EntityList>(
        &'state mut self,
        world: *const World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> CachedQueryManyIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        unsafe {
            self.update_cache(&*world);
            CachedQueryManyIter::new(world, &self.cached_archetype_plans, entities, ticks)
        }
    }

    pub(crate) unsafe fn iter_many_unique_cached_direct_with_ticks<
        'world,
        'state,
        const N: usize,
    >(
        &'state mut self,
        world: *const World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> CachedQueryManyIter<'world, 'state, D, F, std::array::IntoIter<EntityId, N>> {
        unsafe { self.iter_many_cached_direct_with_ticks(world, entities, ticks) }
    }

    pub(crate) unsafe fn is_empty_cached_direct_with_ticks(
        &mut self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> bool {
        unsafe {
            self.iter_cached_direct_with_ticks(world, ticks)
                .next()
                .is_none()
        }
    }

    pub(crate) unsafe fn count_cached_direct_with_ticks(
        &mut self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> usize {
        unsafe { self.iter_cached_direct_with_ticks(world, ticks).count() }
    }

    pub(crate) unsafe fn contains_cached_direct_with_ticks(
        &mut self,
        world: *const World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> bool {
        unsafe {
            self.update_cache(&*world);
            let mut component_locations = Vec::with_capacity(self.access.reads().len());
            let Some(stable_location) =
                self.project_entity(&*world, entity, &mut component_locations)
            else {
                return false;
            };
            F::matches_cached(&*world, entity, &component_locations, ticks)
                && D::fetch_cached(world, entity, stable_location, &component_locations, ticks)
                    .is_some()
        }
    }

    pub(crate) unsafe fn get_cached_direct_with_ticks<'world>(
        &mut self,
        world: *const World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        unsafe {
            if !(&*world).contains_entity(entity) {
                return Err(QueryEntityError::NotSpawned(entity));
            }
            self.update_cache(&*world);
            self.get_cached_direct_after_update(world, entity, ticks)
        }
    }

    unsafe fn get_cached_direct_after_update<'world>(
        &self,
        world: *const World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        unsafe {
            let mut component_locations =
                Vec::<ComponentStorageLocation>::with_capacity(self.access.reads().len());
            let Some(stable_location) =
                self.project_entity(&*world, entity, &mut component_locations)
            else {
                return Err(QueryEntityError::QueryDoesNotMatch(entity));
            };
            if !F::matches_cached(&*world, entity, &component_locations, ticks) {
                return Err(QueryEntityError::QueryDoesNotMatch(entity));
            }
            D::fetch_cached(world, entity, stable_location, &component_locations, ticks)
                .ok_or(QueryEntityError::QueryDoesNotMatch(entity))
        }
    }

    pub(crate) unsafe fn get_many_cached_direct_with_ticks<'world, const N: usize>(
        &mut self,
        world: *const World,
        entities: [EntityId; N],
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        unsafe {
            self.update_cache(&*world);
            collect_many_query_items(entities, |entity| {
                if !(&*world).contains_entity(entity) {
                    return Err(QueryEntityError::NotSpawned(entity));
                }
                self.get_cached_direct_after_update(world, entity, ticks)
            })
        }
    }

    pub(crate) unsafe fn get_many_unique_cached_direct_with_ticks<'world, const N: usize>(
        &mut self,
        world: *const World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        unsafe { self.get_many_cached_direct_with_ticks(world, entities.into_inner(), ticks) }
    }
}
