use std::{any::TypeId, marker::PhantomData};

use super::query_filter::{Added, Changed, QueryFilter, With, Without};
use super::query_state::{find_cached_archetype_plan, CachedArchetypePlan};
use crate::scene::ecs::{
    ChangeTickWindow, Component, ComponentStorageLocation, Mut, QueryDataAccess, QueryEntityItem,
    Ref, StableEntityLocation,
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
    plans: &'state [CachedArchetypePlan],
    component_locations: Vec<ComponentStorageLocation>,
    plan_index: usize,
    row: usize,
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
    plans: &'state [CachedArchetypePlan],
    component_locations: Vec<ComponentStorageLocation>,
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
        plans: &'state [CachedArchetypePlan],
        ticks: ChangeTickWindow,
    ) -> Self {
        Self {
            world,
            plans,
            component_locations: Vec::new(),
            plan_index: 0,
            row: 0,
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
        plans: &'state [CachedArchetypePlan],
        requested_entities: EntityList,
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
        loop {
            let plan = self.plans.get(self.plan_index)?;
            let Some(stable_location) = self
                .world
                .query_stable_location_at(plan.archetype_id(), self.row)
            else {
                self.plan_index += 1;
                self.row = 0;
                continue;
            };
            self.row += 1;
            if !plan.write_component_locations(
                self.world,
                stable_location,
                &mut self.component_locations,
            ) {
                continue;
            }
            let entity = stable_location.stable_id;
            if F::matches_cached(self.world, entity, &self.component_locations, self.ticks) {
                if let Some(item) = D::fetch_cached(
                    self.world,
                    entity,
                    stable_location,
                    &self.component_locations,
                    self.ticks,
                ) {
                    return Some(item);
                }
            }
        }
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
            let Some(stable_location) = self.world.internal_entity_location(entity) else {
                continue;
            };
            let Some(plan) =
                find_cached_archetype_plan(self.plans, stable_location.location.archetype_id)
            else {
                continue;
            };
            if !plan.write_component_locations(
                self.world,
                stable_location,
                &mut self.component_locations,
            ) {
                continue;
            }
            if F::matches_cached(self.world, entity, &self.component_locations, self.ticks) {
                if let Some(item) = D::fetch_cached(
                    self.world,
                    entity,
                    stable_location,
                    &self.component_locations,
                    self.ticks,
                ) {
                    return Some(item);
                }
            }
        }
        None
    }
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
        let location = component_location::<T>(component_locations)?;
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
        let location = component_location::<T>(component_locations)?;
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
        let location = component_location::<T>(component_locations)?;
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
        let location = component_location::<T>(component_locations)?;
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
        let Some(location) = component_location::<T>(component_locations) else {
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

fn component_location<T>(
    component_locations: &[ComponentStorageLocation],
) -> Option<&ComponentStorageLocation>
where
    T: Component,
{
    let rust_type_id = TypeId::of::<T>();
    component_locations
        .iter()
        .find(|location| location.rust_type_id == Some(rust_type_id))
}

fn component_ticks_at_location<T>(
    world: &World,
    component_locations: &[ComponentStorageLocation],
) -> Option<crate::scene::ecs::ComponentTicks>
where
    T: Component,
{
    let location = component_location::<T>(component_locations)?;
    let (_, ticks) = world.component_ref_with_ticks_at_location::<T>(*location)?;
    Some(ticks)
}
