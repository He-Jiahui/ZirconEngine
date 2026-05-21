use std::marker::PhantomData;

use crate::scene::ecs::{
    single_from_iter, CachedQueryData, CachedQueryFilter, CachedQueryIter, CachedQueryManyIter,
    ChangeTickWindow, QueryData, QueryEntityError, QueryEntityItem, QueryFilter, QueryIter,
    QueryManyIter, QueryManyMutIter, QueryMutData, QuerySingleError, QueryState,
};
use crate::scene::{EntityId, World};

pub struct Query<'world, D, F = ()> {
    world: *mut World,
    // SystemState owns the persistent QueryState; this run item borrows it for
    // explicit cached iteration without changing the default scan iterator.
    state: *mut QueryState<D, F>,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world mut World, &'world mut QueryState<D, F>)>,
}

impl<'world, D, F> Query<'world, D, F> {
    pub(crate) fn new(
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
    pub fn iter(&self) -> QueryIter<'_, '_, D, F> {
        let world = unsafe { &*self.world };
        QueryIter::new(world, world.entity_ids_for_query(), self.ticks)
    }

    pub fn iter_many<EntityList>(
        &self,
        entities: EntityList,
    ) -> QueryManyIter<'_, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let world = unsafe { &*self.world };
        let state = unsafe { &*self.state };
        state.iter_many_with_ticks(world, entities, self.ticks)
    }

    pub fn iter_cached(&mut self) -> QueryIter<'_, '_, D, F> {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.iter_cached_with_ticks(world, self.ticks)
    }

    pub fn iter_many_cached<EntityList>(&mut self, entities: EntityList) -> QueryManyIter<'_, D, F>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.iter_many_cached_with_ticks(world, entities, self.ticks)
    }

    pub fn single(&self) -> Result<D::Item<'_>, QuerySingleError> {
        single_from_iter(self.iter())
    }

    pub fn single_cached(&mut self) -> Result<D::Item<'_>, QuerySingleError> {
        single_from_iter(self.iter_cached())
    }

    pub fn get(&self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let world = unsafe { &*self.world };
        let state = unsafe { &*self.state };
        state.get_with_ticks(world, entity, self.ticks)
    }

    pub fn get_many<const N: usize>(
        &self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let world = unsafe { &*self.world };
        let state = unsafe { &*self.state };
        state.get_many_with_ticks(world, entities, self.ticks)
    }

    pub fn get_cached(&mut self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.get_cached_with_ticks(world, entity, self.ticks)
    }

    pub fn get_many_cached<const N: usize>(
        &mut self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.get_many_cached_with_ticks(world, entities, self.ticks)
    }

    pub fn is_empty(&self) -> bool {
        self.iter().next().is_none()
    }

    pub fn count(&self) -> usize {
        self.iter().count()
    }

    pub fn contains(&self, entity: EntityId) -> bool {
        let world = unsafe { &*self.world };
        let state = unsafe { &*self.state };
        state.contains_with_ticks(world, entity, self.ticks)
    }

    pub fn is_empty_cached(&mut self) -> bool {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.is_empty_cached_with_ticks(world, self.ticks)
    }

    pub fn count_cached(&mut self) -> usize {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.count_cached_with_ticks(world, self.ticks)
    }

    pub fn contains_cached(&mut self, entity: EntityId) -> bool {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.contains_cached_with_ticks(world, entity, self.ticks)
    }
}

impl<D, F> Query<'_, D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    pub fn iter_cached_direct(&mut self) -> CachedQueryIter<'_, '_, D, F> {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.iter_cached_direct_with_ticks(world, self.ticks)
    }

    pub fn single_cached_direct(&mut self) -> Result<D::Item<'_>, QuerySingleError> {
        single_from_iter(self.iter_cached_direct())
    }

    pub fn iter_many_cached_direct<EntityList>(
        &mut self,
        entities: EntityList,
    ) -> CachedQueryManyIter<'_, '_, D, F>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.iter_many_cached_direct_with_ticks(world, entities, self.ticks)
    }

    pub fn get_cached_direct(&mut self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.get_cached_direct_with_ticks(world, entity, self.ticks)
    }

    pub fn get_many_cached_direct<const N: usize>(
        &mut self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.get_many_cached_direct_with_ticks(world, entities, self.ticks)
    }

    pub fn is_empty_cached_direct(&mut self) -> bool {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.is_empty_cached_direct_with_ticks(world, self.ticks)
    }

    pub fn count_cached_direct(&mut self) -> usize {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.count_cached_direct_with_ticks(world, self.ticks)
    }

    pub fn contains_cached_direct(&mut self, entity: EntityId) -> bool {
        let world = unsafe { &*self.world };
        let state = unsafe { &mut *self.state };
        state.contains_cached_direct_with_ticks(world, entity, self.ticks)
    }
}

impl<D, F> Query<'_, D, F>
where
    D: QueryMutData,
    F: QueryFilter,
{
    pub fn get_mut(&mut self, entity: EntityId) -> Result<D::Item<'_>, QueryEntityError> {
        let world = unsafe { &mut *self.world };
        let state = unsafe { &mut *self.state };
        state.get_mut_with_ticks(world, entity, self.ticks)
    }

    pub fn get_many_mut<const N: usize>(
        &mut self,
        entities: [EntityId; N],
    ) -> Result<[D::Item<'_>; N], QueryEntityError> {
        let world = unsafe { &mut *self.world };
        let state = unsafe { &mut *self.state };
        state.get_many_mut_with_ticks(world, entities, self.ticks)
    }

    pub fn iter_many_mut<EntityList>(
        &mut self,
        entities: EntityList,
    ) -> QueryManyMutIter<'_, D, F, EntityList::IntoIter>
    where
        EntityList: IntoIterator,
        EntityList::Item: QueryEntityItem,
    {
        let world = unsafe { &mut *self.world };
        let state = unsafe { &mut *self.state };
        state.iter_many_mut_with_ticks(world, entities, self.ticks)
    }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(D::Item<'_>)) {
        let world = unsafe { &mut *self.world };
        let state = unsafe { &mut *self.state };
        state.for_each_mut_with_ticks(world, self.ticks, |item| f(item));
    }
}
