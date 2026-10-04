use std::{any::TypeId, marker::PhantomData};

use super::query_filter::{Added, Changed, QueryFilter, With, Without};
use super::query_state::{find_cached_archetype_plan, CachedArchetypePlan};
use crate::scene::ecs::{
    ChangeTickWindow, Component, ComponentStorageLocation, Mut, QueryDataAccess, QueryEntityItem,
    Ref, StableEntityLocation,
};
use crate::scene::{EntityId, World};

/// 按本行已编译位置提取查询项；内置的 `&mut T` 与 `Mut<T>` 描述在此分别投影为 `&T` 与 `Ref<T>`。
pub trait CachedQueryData: QueryDataAccess {
    type Item<'world>;

    /// # Safety
    /// Keep the raw origin valid for `'world` with coherent current typed rows,
    /// truthful declared access and compatibility with every live item/Param.
    /// Returned items retain only row leaves/copies; no World or eager metadata
    /// parent may escape. A scoped parent read also needs actual compatibility.
    unsafe fn fetch_cached<'world>(
        // The API constructor binds this raw origin to the genuine loan/run lifetime.
        world: *const World,
        entity: EntityId,
        stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>>;
}

/// 缓存过滤入口；内置 With/Without 依赖原型计划已完成的存在性筛选，Added/Changed 再检查当前实体时钟。
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
    world: *const World,
    plans: &'state [CachedArchetypePlan],
    component_locations: Vec<ComponentStorageLocation>,
    plan_index: usize,
    row: usize,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world World, fn() -> (D, F))>,
}

pub struct CachedQueryManyIter<'world, 'state, D, F = (), I = std::vec::IntoIter<EntityId>>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    world: *const World,
    plans: &'state [CachedArchetypePlan],
    component_locations: Vec<ComponentStorageLocation>,
    requested_entities: I,
    ticks: ChangeTickWindow,
    _marker: PhantomData<(&'world World, fn() -> (D, F))>,
}

impl<'world, 'state, D, F> CachedQueryIter<'world, 'state, D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    pub(crate) unsafe fn new(
        world: *const World,
        plans: &'state [CachedArchetypePlan],
        ticks: ChangeTickWindow,
    ) -> Self {
        unsafe {
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
}

impl<'world, 'state, D, F, I> CachedQueryManyIter<'world, 'state, D, F, I>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
    I: Iterator,
    I::Item: QueryEntityItem,
{
    pub(crate) unsafe fn new<EntityList>(
        world: *const World,
        plans: &'state [CachedArchetypePlan],
        requested_entities: EntityList,
        ticks: ChangeTickWindow,
    ) -> Self
    where
        EntityList: IntoIterator<IntoIter = I>,
        EntityList::Item: QueryEntityItem,
    {
        unsafe {
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
}

impl<'world, 'state, D, F> Iterator for CachedQueryIter<'world, 'state, D, F>
where
    D: CachedQueryData,
    F: CachedQueryFilter,
{
    type Item = D::Item<'world>;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: the cursor's original grant keeps storage/structure and
        // declared compatible leaves valid; no World parent is retained.
        unsafe {
            loop {
                // TODO: [CR-R02-runtime_ecs_query_change_windows-0002] 待确认直接缓存查询是否须保持普通查询的稳定顺序；此处按原型物理行扫描，普通迭代按逻辑顺序合并；缺少删除/迁移后的顺序对照，下一步补用例并核对调用方约束。
                let plan = self.plans.get(self.plan_index)?;
                let Some(stable_location) =
                    (&*self.world).query_stable_location_at(plan.archetype_id(), self.row)
                else {
                    self.plan_index += 1;
                    self.row = 0;
                    continue;
                };
                self.row += 1;
                if !plan.write_component_locations(
                    &*self.world,
                    stable_location,
                    &mut self.component_locations,
                ) {
                    continue;
                }
                let entity = stable_location.stable_id;
                if F::matches_cached(&*self.world, entity, &self.component_locations, self.ticks) {
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
        // SAFETY: the cursor's original grant keeps storage/structure and
        // declared compatible leaves valid; no World parent is retained.
        unsafe {
            for entity_item in self.requested_entities.by_ref() {
                let entity = entity_item.entity_id();
                let Some(stable_location) = (&*self.world).internal_entity_location(entity) else {
                    continue;
                };
                let Some(plan) =
                    find_cached_archetype_plan(self.plans, stable_location.location.archetype_id)
                else {
                    continue;
                };
                if !plan.write_component_locations(
                    &*self.world,
                    stable_location,
                    &mut self.component_locations,
                ) {
                    continue;
                }
                if F::matches_cached(&*self.world, entity, &self.component_locations, self.ticks) {
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

    unsafe fn fetch_cached<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, _) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(value)
        }
    }
}

impl<'query, T> CachedQueryData for &'query mut T
where
    T: Component,
{
    type Item<'world> = &'world T;

    unsafe fn fetch_cached<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, _) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(value)
        }
    }
}

impl<'query, T> CachedQueryData for Ref<'query, T>
where
    T: Component,
{
    type Item<'world> = Ref<'world, T>;

    unsafe fn fetch_cached<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, component_ticks) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(Ref::new(value, component_ticks, ticks))
        }
    }
}

impl<'query, T> CachedQueryData for Mut<'query, T>
where
    T: Component,
{
    type Item<'world> = Ref<'world, T>;

    unsafe fn fetch_cached<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let location = component_location::<T>(component_locations)?;
            let (value, component_ticks) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)?;
            Some(Ref::new(value, component_ticks, ticks))
        }
    }
}

impl<'query, T> CachedQueryData for Option<&'query T>
where
    T: Component,
{
    type Item<'world> = Option<&'world T>;

    unsafe fn fetch_cached<'world>(
        world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe {
            let Some(location) = component_location::<T>(component_locations) else {
                return Some(None);
            };
            let Some((value, _)) =
                World::query_component_ref_with_ticks_at_location::<T>(world, *location)
            else {
                return Some(None);
            };
            Some(Some(value))
        }
    }
}

impl CachedQueryData for EntityId {
    type Item<'world> = EntityId;

    unsafe fn fetch_cached<'world>(
        _world: *const World,
        entity: EntityId,
        _stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe { Some(entity) }
    }
}

impl CachedQueryData for StableEntityLocation {
    type Item<'world> = StableEntityLocation;

    unsafe fn fetch_cached<'world>(
        _world: *const World,
        _entity: EntityId,
        stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe { Some(stable_location) }
    }
}

impl CachedQueryData for () {
    type Item<'world> = ();

    unsafe fn fetch_cached<'world>(
        _world: *const World,
        _entity: EntityId,
        _stable_location: StableEntityLocation,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        unsafe { Some(()) }
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
            unsafe fn fetch_cached<'world>(
                world: *const World,
                entity: EntityId,
                stable_location: StableEntityLocation,
                component_locations: &[ComponentStorageLocation],
                ticks: ChangeTickWindow,
            ) -> Option<Self::Item<'world>> {
                unsafe {
                    Some(($($name::fetch_cached(world, entity, stable_location, component_locations, ticks)?,)*))

                }
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
    world.component_ticks_at_location::<T>(*location)
}
