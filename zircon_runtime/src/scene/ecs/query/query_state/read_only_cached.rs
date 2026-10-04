use std::array;

use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTickWindow, ComponentStorageLocation, QueryCombinationIter,
    QueryData, QueryEntityError, QueryEntityItem, QueryFilter, QueryIter, QueryManyCachedIter,
    QuerySingleError, UniqueEntityArray,
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
    // 缓存入口会把命中的列位置统计回 QueryState，供后续性能诊断与计划复用。
    pub fn iter_many_cached<'world, 'state, EntityList>(
        &'state mut self,
        world: &'world World,
        entities: EntityList,
    ) -> QueryManyCachedIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_many_cached_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn iter_many_unique_cached<'world, 'state, const N: usize>(
        &'state mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyCachedIter<'world, 'state, D, F, array::IntoIter<EntityId, N>> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_many_unique_cached_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn iter_combinations_cached<'world, 'state, const K: usize>(
        &'state mut self,
        world: &'world World,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_combinations_cached_with_ticks(world as *const World, ticks)
        }
    }

    pub fn iter_cached<'world, 'state>(
        &'state mut self,
        world: &'world World,
    ) -> QueryIter<'world, 'state, D, F> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.iter_cached_with_ticks(world as *const World, ticks)
        }
    }

    pub(crate) unsafe fn iter_cached_with_ticks<'world, 'state>(
        &'state mut self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> QueryIter<'world, 'state, D, F> {
        unsafe {
            self.update_cache(&*world);
            QueryIter::new_cached_plans(world, &self.cached_archetype_plans, ticks, self)
        }
    }

    pub fn single_cached<'world>(
        &mut self,
        world: &'world World,
    ) -> Result<D::Item<'world>, QuerySingleError> {
        single_from_iter(self.iter_cached(world))
    }

    pub fn get_cached<'world>(
        &mut self,
        world: &'world World,
        entity: EntityId,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_cached_with_ticks(world as *const World, entity, ticks)
        }
    }

    pub fn get_many_cached<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_many_cached_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn get_many_unique_cached<'world, const N: usize>(
        &mut self,
        world: &'world World,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.get_many_unique_cached_with_ticks(world as *const World, entities, ticks)
        }
    }

    pub fn is_empty_cached(&mut self, world: &World) -> bool {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.is_empty_cached_with_ticks(world as *const World, ticks)
        }
    }

    pub fn count_cached(&mut self, world: &World) -> usize {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.count_cached_with_ticks(world as *const World, ticks)
        }
    }

    pub fn contains_cached(&mut self, world: &World, entity: EntityId) -> bool {
        // SAFETY: the public World loan bounds every returned item/cursor; the
        // access plan, storage and observation window remain fixed for that loan.
        unsafe {
            let ticks = ChangeTickWindow::all(world.read_change_tick());
            self.contains_cached_with_ticks(world as *const World, entity, ticks)
        }
    }

    pub(crate) unsafe fn iter_many_unique_cached_with_ticks<'world, 'state, const N: usize>(
        &'state mut self,
        world: *const World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> QueryManyCachedIter<'world, 'state, D, F, array::IntoIter<EntityId, N>> {
        unsafe { self.iter_many_cached_with_ticks(world, entities, ticks) }
    }

    pub(crate) unsafe fn iter_combinations_cached_with_ticks<'world, 'state, const K: usize>(
        &'state mut self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> QueryCombinationIter<'world, 'state, D, F, K> {
        unsafe {
            self.update_cache(&*world);
            QueryCombinationIter::new_from_cached_plans(world, &self.cached_archetype_plans, ticks)
        }
    }

    pub(crate) unsafe fn iter_many_cached_with_ticks<'world, 'state, EntityList>(
        &'state mut self,
        world: *const World,
        entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> QueryManyCachedIter<'world, 'state, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        unsafe {
            self.update_cache(&*world);
            QueryManyCachedIter::new(world, &self.cached_archetype_plans, entities, ticks, self)
        }
    }

    pub(crate) unsafe fn is_empty_cached_with_ticks(
        &mut self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> bool {
        unsafe { self.iter_cached_with_ticks(world, ticks).next().is_none() }
    }

    pub(crate) unsafe fn count_cached_with_ticks(
        &mut self,
        world: *const World,
        ticks: ChangeTickWindow,
    ) -> usize {
        unsafe { self.iter_cached_with_ticks(world, ticks).count() }
    }

    pub(crate) unsafe fn contains_cached_with_ticks(
        &mut self,
        world: *const World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> bool {
        unsafe {
            self.update_cache(&*world);
            let mut component_locations = Vec::with_capacity(self.access.reads().len());
            let Some(_) = self.project_entity(&*world, entity, &mut component_locations) else {
                return false;
            };
            let mut change_detection_stats = ChangeDetectionScanStats::default();
            let matches = F::matches_component_locations_with_stats(
                &*world,
                entity,
                &component_locations,
                ticks,
                &mut change_detection_stats,
            );
            self.record_change_detection_stats(change_detection_stats);
            matches
        }
    }

    pub(crate) unsafe fn get_cached_with_ticks<'world>(
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
            self.get_cached_after_update_with_ticks(world, entity, ticks)
        }
    }

    unsafe fn get_cached_after_update_with_ticks<'world>(
        &mut self,
        world: *const World,
        entity: EntityId,
        ticks: ChangeTickWindow,
    ) -> Result<D::Item<'world>, QueryEntityError> {
        unsafe {
            if !(&*world).contains_entity(entity) {
                return Err(QueryEntityError::NotSpawned(entity));
            }
            let mut component_locations =
                Vec::<ComponentStorageLocation>::with_capacity(self.access.reads().len());
            let Some(stable_location) =
                self.project_entity(&*world, entity, &mut component_locations)
            else {
                return Err(QueryEntityError::QueryDoesNotMatch(entity));
            };
            let mut change_detection_stats = ChangeDetectionScanStats::default();
            if !F::matches_component_locations_with_stats(
                &*world,
                entity,
                &component_locations,
                ticks,
                &mut change_detection_stats,
            ) {
                self.record_change_detection_stats(change_detection_stats);
                return Err(QueryEntityError::QueryDoesNotMatch(entity));
            }
            let Some(item) = D::fetch_with_component_locations(
                world,
                entity,
                stable_location,
                &component_locations,
                ticks,
            ) else {
                self.record_change_detection_stats(change_detection_stats);
                return Err(QueryEntityError::QueryDoesNotMatch(entity));
            };
            self.record_change_detection_stats(change_detection_stats);
            Ok(item)
        }
    }

    pub(crate) unsafe fn get_many_cached_with_ticks<'world, const N: usize>(
        &mut self,
        world: *const World,
        entities: [EntityId; N],
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        unsafe {
            self.update_cache(&*world);
            collect_many_query_items(entities, |entity| {
                self.get_cached_after_update_with_ticks(world, entity, ticks)
            })
        }
    }

    pub(crate) unsafe fn get_many_unique_cached_with_ticks<'world, const N: usize>(
        &mut self,
        world: *const World,
        entities: UniqueEntityArray<N>,
        ticks: ChangeTickWindow,
    ) -> Result<[D::Item<'world>; N], QueryEntityError> {
        unsafe { self.get_many_cached_with_ticks(world, entities.into_inner(), ticks) }
    }
}
