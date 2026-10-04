use std::{array, marker::PhantomData};

use crate::scene::ecs::{
    single_from_iter, CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter,
    ChangeTickWindow, QueryCombinationIter, QueryCombinationMutIter, QueryData, QueryEntityError,
    QueryEntityItem, QueryFilter, QueryIter, QueryManyCachedIter, QueryManyIter, QueryManyMutIter,
    QueryManyUniqueMutIter, QueryMutData, QueryMutIter, QuerySingleError, QueryState,
    UniqueEntityArray,
};
use crate::scene::{EntityId, World};

pub struct Query<'world, D, F = ()> {
    world: *mut World,
    // SystemState owns the persistent QueryState for this run. Any entry point
    // that refreshes or lends its cache requires `&mut self`, preventing a
    // second refresh from invalidating plans held by a live iterator.
    state: *mut QueryState<D, F>,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, &'world mut QueryState<D, F>)>,
}

impl<'world, D, F> Query<'world, D, F> {
    /// # Safety
    /// The caller must hold the original SystemState World/state grant for
    /// `'world`, with truthful mutually compatible Params, fixed storage and a
    /// serialized observation window. No other Param/item may retain conflicting
    /// World/ordinary metadata parents; this Query must not outlive that run.
    pub(crate) unsafe fn new(
        world: *mut World,
        state: &'world mut QueryState<D, F>,
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            state,
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<D, F> Query<'_, D, F>
where
    D: QueryData,
    F: QueryFilter,
{
    pub fn iter(&mut self) -> QueryIter<'_, '_, D, F> {
        // The exclusive receiver keeps the cache borrow valid for the
        // iterator's full lifetime.
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn iter_many<EntityList>(
        &self,
        entities: EntityList,
    ) -> QueryManyIter<'_, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let state = unsafe { &*self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_many_with_ticks(self.world.cast_const(), entities, self.ticks) }
    }

    pub fn iter_many_unique<const N: usize>(
        &self,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyIter<'_, D, F, array::IntoIter<EntityId, N>> {
        let state = unsafe { &*self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_many_unique_with_ticks(self.world.cast_const(), entities, self.ticks) }
    }

    pub fn iter_cached(&mut self) -> QueryIter<'_, '_, D, F> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn iter_many_cached<EntityList>(
        &mut self,
        entities: EntityList,
    ) -> QueryManyCachedIter<'_, '_, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_many_cached_with_ticks(self.world.cast_const(), entities, self.ticks) }
    }

    pub fn iter_many_unique_cached<const N: usize>(
        &mut self,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyCachedIter<'_, '_, D, F, array::IntoIter<EntityId, N>> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe {
            state.iter_many_unique_cached_with_ticks(self.world.cast_const(), entities, self.ticks)
        }
    }

    pub fn iter_combinations<const K: usize>(&mut self) -> QueryCombinationIter<'_, '_, D, F, K> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_combinations_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn iter_combinations_cached<const K: usize>(
        &mut self,
    ) -> QueryCombinationIter<'_, '_, D, F, K> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_combinations_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn single(&mut self) -> Result<D::Item<'_>, QuerySingleError> {
        single_from_iter(self.iter())
    }

    pub fn single_cached(&mut self) -> Result<D::Item<'_>, QuerySingleError> {
        single_from_iter(self.iter_cached())
    }

    pub fn get(&self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let state = unsafe { &*self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.get_with_ticks(self.world.cast_const(), entity, self.ticks) }
    }

    pub fn get_many<const N: usize>(
        &self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &*self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe { state.get_many_with_ticks(self.world.cast_const(), entities, self.ticks) }
    }

    pub fn get_many_unique<const N: usize>(
        &self,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &*self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe { state.get_many_unique_with_ticks(self.world.cast_const(), entities, self.ticks) }
    }

    pub fn get_cached(&mut self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.get_cached_with_ticks(self.world.cast_const(), entity, self.ticks) }
    }

    pub fn get_many_cached<const N: usize>(
        &mut self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe { state.get_many_cached_with_ticks(self.world.cast_const(), entities, self.ticks) }
    }

    pub fn get_many_unique_cached<const N: usize>(
        &mut self,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe {
            state.get_many_unique_cached_with_ticks(self.world.cast_const(), entities, self.ticks)
        }
    }

    pub fn is_empty(&mut self) -> bool {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.is_empty_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn count(&mut self) -> usize {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.count_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn contains(&self, entity: EntityId) -> bool {
        let state = unsafe { &*self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.contains_with_ticks(self.world.cast_const(), entity, self.ticks) }
    }

    pub fn is_empty_cached(&mut self) -> bool {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.is_empty_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn count_cached(&mut self) -> usize {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.count_cached_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn contains_cached(&mut self, entity: EntityId) -> bool {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.contains_cached_with_ticks(self.world.cast_const(), entity, self.ticks) }
    }
}

impl<D, F> Query<'_, D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    pub fn iter_cached_direct(&mut self) -> CachedQueryIter<'_, '_, D, F> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_cached_direct_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn single_cached_direct(&mut self) -> Result<D::Item<'_>, QuerySingleError> {
        single_from_iter(self.iter_cached_direct())
    }

    pub fn iter_many_cached_direct<EntityList>(
        &mut self,
        entities: EntityList,
    ) -> CachedQueryManyIter<'_, '_, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe {
            state.iter_many_cached_direct_with_ticks(self.world.cast_const(), entities, self.ticks)
        }
    }

    pub fn iter_many_unique_cached_direct<const N: usize>(
        &mut self,
        entities: UniqueEntityArray<N>,
    ) -> CachedQueryManyIter<'_, '_, D, F, array::IntoIter<EntityId, N>> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe {
            state.iter_many_unique_cached_direct_with_ticks(
                self.world.cast_const(),
                entities,
                self.ticks,
            )
        }
    }

    pub fn get_cached_direct(&mut self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.get_cached_direct_with_ticks(self.world.cast_const(), entity, self.ticks) }
    }

    pub fn get_many_cached_direct<const N: usize>(
        &mut self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe {
            state.get_many_cached_direct_with_ticks(self.world.cast_const(), entities, self.ticks)
        }
    }

    pub fn get_many_unique_cached_direct<const N: usize>(
        &mut self,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe {
            state.get_many_unique_cached_direct_with_ticks(
                self.world.cast_const(),
                entities,
                self.ticks,
            )
        }
    }

    pub fn is_empty_cached_direct(&mut self) -> bool {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.is_empty_cached_direct_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn count_cached_direct(&mut self) -> usize {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.count_cached_direct_with_ticks(self.world.cast_const(), self.ticks) }
    }

    pub fn contains_cached_direct(&mut self, entity: EntityId) -> bool {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe {
            state.contains_cached_direct_with_ticks(self.world.cast_const(), entity, self.ticks)
        }
    }
}

impl<D, F> Query<'_, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub fn get_mut(&mut self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.get_mut_with_ticks(self.world, entity, self.ticks) }
    }

    pub fn single_mut(&mut self) -> Result<D::Item<'_>, QuerySingleError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.single_mut_with_ticks(self.world, self.ticks) }
    }

    pub fn iter_mut(&mut self) -> QueryMutIter<'_, '_, D, F> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_mut_with_ticks(self.world, self.ticks) }
    }

    pub fn get_many_mut<const N: usize>(
        &mut self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe { state.get_many_mut_with_ticks(self.world, entities, self.ticks) }
    }

    pub fn get_many_unique_mut<const N: usize>(
        &mut self,
        entities: UniqueEntityArray<N>,
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let state = unsafe { &mut *self.state };
        // SAFETY: the original run grant/access/window and receiver bound this call.
        unsafe { state.get_many_unique_mut_with_ticks(self.world, entities, self.ticks) }
    }

    pub fn iter_many_mut<EntityList>(
        &mut self,
        entities: EntityList,
    ) -> QueryManyMutIter<'_, '_, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_many_mut_with_ticks(self.world, entities, self.ticks) }
    }

    pub fn iter_many_unique_mut<const N: usize>(
        &mut self,
        entities: UniqueEntityArray<N>,
    ) -> QueryManyUniqueMutIter<'_, '_, D, F, array::IntoIter<EntityId, N>> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_many_unique_mut_with_ticks(self.world, entities, self.ticks) }
    }

    pub fn iter_combinations_mut<const K: usize>(
        &mut self,
    ) -> QueryCombinationMutIter<'_, '_, D, F, K> {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe { state.iter_combinations_mut_with_ticks(self.world, self.ticks) }
    }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(D::Item<'_>)) {
        let state = unsafe { &mut *self.state };
        // SAFETY: construction bound this state/access/window to the original
        // SystemState grant; the receiver bounds items and freezes its cache.
        unsafe {
            state.for_each_mut_with_ticks(self.world, self.ticks, |item| f(item));
        }
    }
}
