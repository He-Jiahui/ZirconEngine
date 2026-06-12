use std::marker::PhantomData;

use super::query_filter::{Added, Changed, QueryFilter, With, Without};
use crate::scene::ecs::{
    ChangeTickWindow, Component, ComponentId, ComponentStorageLocation, Mut, QueryDataAccess,
    QueryEntityItem, Ref, StableEntityLocation,
};
use crate::scene::{EntityId, World};

pub trait CachedQueryData: QueryDataAccess {
    type Item<'world>;

    fn fetch_cached<'world>(
        world: &'world World,
        entity: EntityId,
        stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>>;
}

pub trait CachedQueryFilter: QueryFilter {
    fn matches_cached(
        world: &World,
        entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> bool;
}

pub struct CachedQueryIter<'world, 'state, D, F = ()>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    world: &'world World,
    entities: &'state [EntityId],
    locations: &'state [StableEntityLocation],
    component_locations: &'state [ComponentStorageLocation],
    component_location_offsets: &'state [usize],
    index: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

pub struct CachedQueryManyIter<'world, 'state, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: &'world World,
    cached_entities: &'state [EntityId],
    locations: &'state [StableEntityLocation],
    component_locations: &'state [ComponentStorageLocation],
    component_location_offsets: &'state [usize],
    cached_entity_indices: &'state [(EntityId, usize)],
    requested_entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<fn() -> (D, F)>,
}

impl<'world, 'state, D, F> CachedQueryIter<'world, 'state, D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    pub(crate) fn new(
        world: &'world World,
        entities: &'state [EntityId],
        locations: &'state [StableEntityLocation],
        component_locations: &'state [ComponentStorageLocation],
        component_location_offsets: &'state [usize],
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            entities,
            locations,
            component_locations,
            component_location_offsets,
            index: 0,
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<'world, 'state, D, F, I> CachedQueryManyIter<'world, 'state, D, F, I>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) fn new<EntityList>(
        world: &'world World,
        cached_entities: &'state [EntityId],
        locations: &'state [StableEntityLocation],
        component_locations: &'state [ComponentStorageLocation],
        component_location_offsets: &'state [usize],
        cached_entity_indices: &'state [(EntityId, usize)],
        requested_entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        Self {
            world,
            cached_entities,
            locations,
            component_locations,
            component_location_offsets,
            cached_entity_indices,
            requested_entities: requested_entities.into_iter(),
            ticks,
            _marker: PhantomData,
        }
    }
}

impl<'world, 'state, D, F> Iterator for CachedQueryIter<'world, 'state, D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entity) = self.entities.get(self.index).copied() {
            let stable_location = self.locations.get(self.index).copied()?;
            let component_locations = cached_query_component_locations(
                self.component_locations,
                self.component_location_offsets,
                self.index,
            )?;
            self.index += 1;

            if F::matches_cached(self.world, entity, component_locations, self.ticks) {
                if let Some(item) = D::fetch_cached(
                    self.world,
                    entity,
                    stable_location,
                    component_locations,
                    self.ticks,
                ) {
                    return Some(item);
                }
            }
        }
        None
    }
}

impl<'world, 'state, D, F, I> Iterator for CachedQueryManyIter<'world, 'state, D, F, I>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        for entity_item in self.requested_entities.by_ref() {
            let entity = entity_item.entity_id();
            let Some(index) = cached_query_entity_index(self.cached_entity_indices, entity) else {
                continue;
            };
            let entity = *self.cached_entities.get(index)?;
            let stable_location = *self.locations.get(index)?;
            let component_locations = cached_query_component_locations(
                self.component_locations,
                self.component_location_offsets,
                index,
            )?;

            if F::matches_cached(self.world, entity, component_locations, self.ticks) {
                if let Some(item) = D::fetch_cached(
                    self.world,
                    entity,
                    stable_location,
                    component_locations,
                    self.ticks,
                ) {
                    return Some(item);
                }
            }
        }
        None
    }
}

pub(crate) fn cached_query_entity_index(
    cached_entity_indices: &[(EntityId, usize)],
    entity: EntityId,
) -> Option<usize> {
    let entry = cached_entity_indices
        .binary_search_by_key(&entity, |(candidate, _)| *candidate)
        .ok()?;
    Some(cached_entity_indices[entry].1)
}

pub(crate) fn cached_query_component_locations<'locations>(
    component_locations: &'locations [ComponentStorageLocation],
    component_location_offsets: &[usize],
    index: usize,
) -> Option<&'locations [ComponentStorageLocation]> {
    let start = *component_location_offsets.get(index)?;
    let end = *component_location_offsets.get(index + 1)?;
    component_locations.get(start..end)
}

impl<T> CachedQueryFilter for With<T>
where
    T: Component,
{
    fn matches_cached(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> bool {
        // The query cache is already built from the access descriptor's
        // required component set, so structural filters do not need another
        // entity-map lookup on the direct iteration path.
        true
    }
}

impl<T> CachedQueryFilter for Without<T>
where
    T: Component,
{
    fn matches_cached(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> bool {
        // The query cache excludes archetypes that contain this component.
        true
    }
}

impl<T> CachedQueryFilter for Added<T>
where
    T: Component,
{
    fn matches_cached(
        world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> bool {
        let Some(component_ticks) = component_ticks_at_location::<T>(world, component_locations)
        else {
            return false;
        };
        component_ticks.is_added(ticks)
    }
}

impl<T> CachedQueryFilter for Changed<T>
where
    T: Component,
{
    fn matches_cached(
        world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> bool {
        let Some(component_ticks) = component_ticks_at_location::<T>(world, component_locations)
        else {
            return false;
        };
        component_ticks.is_changed(ticks)
    }
}

impl CachedQueryFilter for () {
    fn matches_cached(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> bool {
        true
    }
}

macro_rules! tuple_cached_query_filter {
    ($($name:ident),*) => {
        impl<$($name),*> CachedQueryFilter for ($($name,)*)
        where
            $($name: CachedQueryFilter,)*
        {
            #[allow(non_snake_case)]
            fn matches_cached(
                world: &World,
                entity: EntityId,
                component_locations: &[ComponentStorageLocation],
                ticks: ChangeTickWindow,
            ) -> bool {
                true $(&& $name::matches_cached(world, entity, component_locations, ticks))*
            }
        }
    };
}

tuple_cached_query_filter!(A);
tuple_cached_query_filter!(A, B);
tuple_cached_query_filter!(A, B, C);
tuple_cached_query_filter!(A, B, C, D);
tuple_cached_query_filter!(A, B, C, D, E);
tuple_cached_query_filter!(A, B, C, D, E, F);
tuple_cached_query_filter!(A, B, C, D, E, F, G);
tuple_cached_query_filter!(A, B, C, D, E, F, G, H);

impl<'query, T> CachedQueryData for &'query T
where
    T: Component,
{
    type Item<'world> = &'world T;

    fn fetch_cached<'world>(
        world: &'world World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let component_id = world.registered_component_id::<T>()?;
        let location = component_location(component_locations, component_id)?;
        let (value, _) = world.component_ref_with_ticks_at_location::<T>(*location)?;
        Some(value)
    }
}

impl<'query, T> CachedQueryData for &'query mut T
where
    T: Component,
{
    type Item<'world> = &'world T;

    fn fetch_cached<'world>(
        world: &'world World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let component_id = world.registered_component_id::<T>()?;
        let location = component_location(component_locations, component_id)?;
        let (value, _) = world.component_ref_with_ticks_at_location::<T>(*location)?;
        Some(value)
    }
}

impl<'query, T> CachedQueryData for Ref<'query, T>
where
    T: Component,
{
    type Item<'world> = Ref<'world, T>;

    fn fetch_cached<'world>(
        world: &'world World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let component_id = world.registered_component_id::<T>()?;
        let location = component_location(component_locations, component_id)?;
        let (value, component_ticks) =
            world.component_ref_with_ticks_at_location::<T>(*location)?;
        Some(Ref::new(value, component_ticks, ticks))
    }
}

impl<'query, T> CachedQueryData for Mut<'query, T>
where
    T: Component,
{
    type Item<'world> = Ref<'world, T>;

    fn fetch_cached<'world>(
        world: &'world World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let component_id = world.registered_component_id::<T>()?;
        let location = component_location(component_locations, component_id)?;
        let (value, component_ticks) =
            world.component_ref_with_ticks_at_location::<T>(*location)?;
        Some(Ref::new(value, component_ticks, ticks))
    }
}

impl<'query, T> CachedQueryData for Option<&'query T>
where
    T: Component,
{
    type Item<'world> = Option<&'world T>;

    fn fetch_cached<'world>(
        world: &'world World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let Some(component_id) = world.registered_component_id::<T>() else {
            return Some(None);
        };
        let Some(location) = component_location(component_locations, component_id) else {
            return Some(None);
        };
        let Some((value, _)) = world.component_ref_with_ticks_at_location::<T>(*location) else {
            return Some(None);
        };
        Some(Some(value))
    }
}

impl CachedQueryData for EntityId {
    type Item<'world> = EntityId;

    fn fetch_cached<'world>(
        _world: &'world World,
        entity: EntityId,
        _stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        Some(entity)
    }
}

impl CachedQueryData for StableEntityLocation {
    type Item<'world> = StableEntityLocation;

    fn fetch_cached<'world>(
        _world: &'world World,
        _entity: EntityId,
        stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        Some(stable_location)
    }
}

impl CachedQueryData for () {
    type Item<'world> = ();

    fn fetch_cached<'world>(
        _world: &'world World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        Some(())
    }
}

macro_rules! tuple_cached_query_data {
    ($($name:ident),*) => {
        impl<$($name),*> CachedQueryData for ($($name,)*)
        where
            $($name: CachedQueryData,)*
        {
            type Item<'world> = ($($name::Item<'world>,)*);

            #[allow(non_snake_case)]
            fn fetch_cached<'world>(
                world: &'world World,
                entity: EntityId,
                stable_location: StableEntityLocation,
                component_locations: &[ComponentStorageLocation],
                ticks: ChangeTickWindow,
            ) -> Option<Self::Item<'world>> {
                Some(($($name::fetch_cached(world, entity, stable_location, component_locations, ticks)?,)*))
            }
        }
    };
}

tuple_cached_query_data!(A);
tuple_cached_query_data!(A, B);
tuple_cached_query_data!(A, B, C);
tuple_cached_query_data!(A, B, C, D);
tuple_cached_query_data!(A, B, C, D, E);
tuple_cached_query_data!(A, B, C, D, E, F);
tuple_cached_query_data!(A, B, C, D, E, F, G);
tuple_cached_query_data!(A, B, C, D, E, F, G, H);

fn component_location(
    component_locations: &[ComponentStorageLocation],
    component_id: ComponentId,
) -> Option<&ComponentStorageLocation> {
    let index = component_locations
        .binary_search_by_key(&component_id, |location| location.component_id)
        .ok()?;
    component_locations.get(index)
}

fn component_ticks_at_location<T>(
    world: &World,
    component_locations: &[ComponentStorageLocation],
) -> Option<crate::scene::ecs::ComponentTicks>
where
    T: Component,
{
    let component_id = world.registered_component_id::<T>()?;
    let location = component_location(component_locations, component_id)?;
    let (_, ticks) = world.component_ref_with_ticks_at_location::<T>(*location)?;
    Some(ticks)
}
