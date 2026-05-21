use std::marker::PhantomData;

use crate::scene::ecs::{ChangeTickWindow, Component, QueryAccess, QueryAccessError};
use crate::scene::{EntityId, World};

pub trait QueryFilter: 'static + Send + Sync {
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError>;
    fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool;
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
}

pub struct Added<T>(PhantomData<T>);

impl<T> QueryFilter for Added<T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_read(component_id)?;
        access.add_with(component_id);
        Ok(())
    }

    fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool {
        world
            .component_change_ticks::<T>(entity)
            .is_some_and(|component_ticks| component_ticks.is_added(ticks))
    }
}

pub struct Changed<T>(PhantomData<T>);

impl<T> QueryFilter for Changed<T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_read(component_id)?;
        access.add_with(component_id);
        Ok(())
    }

    fn matches(world: &World, entity: EntityId, ticks: ChangeTickWindow) -> bool {
        world
            .component_change_ticks::<T>(entity)
            .is_some_and(|component_ticks| component_ticks.is_changed(ticks))
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
