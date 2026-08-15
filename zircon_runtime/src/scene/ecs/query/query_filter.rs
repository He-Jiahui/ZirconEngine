use std::{any::TypeId, marker::PhantomData};

use crate::scene::ecs::{
    ChangeDetectionScanStats, ChangeTickWindow, Component, ComponentStorageLocation,
    ComponentTicks, QueryAccess, QueryAccessError,
};
use crate::scene::{EntityId, World};

pub trait QueryFilter: 'static + Send + Sync {
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError>;
    fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool;
    fn matches_component_locations(
        world: &World,
        entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
    ) -> bool;

    fn matches_component_locations_with_stats(
        world: &World,
        entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
        stats: &mut ChangeDetectionScanStats,
    ) -> bool;
}

pub struct With<T>(PhantomData<T>);

impl<T> QueryFilter for With<T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_with(component_id);
        Ok(())
    }

    fn matches(world: &World, entity: EntityId, _ticks: ChangeTickWindow) -> bool {
        world.get::<T>(entity).is_some()
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> bool {
        true
    }

    fn matches_component_locations_with_stats(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
        _stats: &mut ChangeDetectionScanStats,
    ) -> bool {
        true
    }
}

pub struct Without<T>(PhantomData<T>);

impl<T> QueryFilter for Without<T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_without(component_id);
        Ok(())
    }

    fn matches(world: &World, entity: EntityId, _ticks: ChangeTickWindow) -> bool {
        world.get::<T>(entity).is_none()
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> bool {
        true
    }

    fn matches_component_locations_with_stats(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
        _stats: &mut ChangeDetectionScanStats,
    ) -> bool {
        true
    }
}

pub struct Added<T>(PhantomData<T>);

impl<T> QueryFilter for Added<T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_filter_read(component_id);
        access.add_with(component_id);
        Ok(())
    }

    fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool {
        let Some(component_ticks) = world.component_change_ticks::<T>(entity) else {
            return false;
        };
        component_ticks.is_added(ticks)
    }

    fn matches_component_locations(
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

    fn matches_component_locations_with_stats(
        world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
        stats: &mut ChangeDetectionScanStats,
    ) -> bool {
        let Some(component_ticks) = component_ticks_at_location::<T>(world, component_locations)
        else {
            return false;
        };
        stats.scan_added(component_ticks, ticks)
    }
}

pub struct Changed<T>(PhantomData<T>);

impl<T> QueryFilter for Changed<T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_filter_read(component_id);
        access.add_with(component_id);
        Ok(())
    }

    fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool {
        let Some(component_ticks) = world.component_change_ticks::<T>(entity) else {
            return false;
        };
        component_ticks.is_changed(ticks)
    }

    fn matches_component_locations(
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

    fn matches_component_locations_with_stats(
        world: &World,
        _entity: EntityId,
        component_locations: &[ComponentStorageLocation],
        ticks: ChangeTickWindow,
        stats: &mut ChangeDetectionScanStats,
    ) -> bool {
        let Some(component_ticks) = component_ticks_at_location::<T>(world, component_locations)
        else {
            return false;
        };
        stats.scan_changed(component_ticks, ticks)
    }
}

impl QueryFilter for () {
    fn update_access(
        _world: &mut World,
        _access: &mut QueryAccess,
    ) -> Result<(), QueryAccessError> {
        Ok(())
    }

    fn matches(_world: &World, _entity: EntityId, _ticks: ChangeTickWindow) -> bool {
        true
    }

    fn matches_component_locations(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
    ) -> bool {
        true
    }

    fn matches_component_locations_with_stats(
        _world: &World,
        _entity: EntityId,
        _component_locations: &[ComponentStorageLocation],
        _ticks: ChangeTickWindow,
        _stats: &mut ChangeDetectionScanStats,
    ) -> bool {
        true
    }
}

macro_rules! tuple_query_filter {
    ($($name:ident),*) => {
        impl<$($name),*> QueryFilter for ($($name,)*)
        where
            $($name: QueryFilter,)*
        {
            fn update_access(
                world: &mut World,
                access: &mut QueryAccess,
            ) -> Result<(), QueryAccessError> {
                $($name::update_access(world, access)?;)*
                Ok(())
            }

            fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool {
                true $(&& $name::matches(world, entity, ticks))*
            }

            #[allow(non_snake_case)]
            fn matches_component_locations(
                world: &World,
                entity: EntityId,
                component_locations: &[ComponentStorageLocation],
                ticks: ChangeTickWindow,
            ) -> bool {
                true $(&& $name::matches_component_locations(world, entity, component_locations, ticks))*
            }

            #[allow(non_snake_case)]
            fn matches_component_locations_with_stats(
                world: &World,
                entity: EntityId,
                component_locations: &[ComponentStorageLocation],
                ticks: ChangeTickWindow,
                stats: &mut ChangeDetectionScanStats,
            ) -> bool {
                true $(&& $name::matches_component_locations_with_stats(world, entity, component_locations, ticks, stats))*
            }
        }
    };
}

tuple_query_filter!(A);
tuple_query_filter!(A, B);
tuple_query_filter!(A, B, C);
tuple_query_filter!(A, B, C, D);
tuple_query_filter!(A, B, C, D, E);
tuple_query_filter!(A, B, C, D, E, F);
tuple_query_filter!(A, B, C, D, E, F, G);
tuple_query_filter!(A, B, C, D, E, F, G, H);

fn component_ticks_at_location<T>(
    world: &World,
    component_locations: &[ComponentStorageLocation],
) -> Option<ComponentTicks>
where
    T: Component,
{
    let rust_type_id = TypeId::of::<T>();
    let location = component_locations
        .iter()
        .find(|location| location.rust_type_id == Some(rust_type_id))?;
    let (_, ticks) = world.component_ref_with_ticks_at_location::<T>(*location)?;
    Some(ticks)
}
