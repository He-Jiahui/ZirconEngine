use crate::scene::ecs::{Component, QueryAccess, QueryAccessError};
use crate::scene::{EntityId, World};

pub trait QueryDataAccess {
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError>;
    fn matches_data(world: &World, entity: EntityId) -> bool;
}

pub trait QueryData: QueryDataAccess {
    type Item<'world>;

    fn fetch<'world>(world: &'world World, entity: EntityId) -> Option<Self::Item<'world>>;
}

pub trait QueryMutData: QueryDataAccess {
    type Item<'world>;

    fn fetch_mut<'world>(world: &'world mut World, entity: EntityId) -> Option<Self::Item<'world>>;
}

impl<'query, T> QueryDataAccess for &'query T
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_read(component_id)
    }

    fn matches_data(world: &World, entity: EntityId) -> bool {
        world.get::<T>(entity).is_some()
    }
}

impl<'query, T> QueryData for &'query T
where
    T: Component,
{
    type Item<'world> = &'world T;

    fn fetch<'world>(world: &'world World, entity: EntityId) -> Option<Self::Item<'world>> {
        world.get::<T>(entity)
    }
}

impl<'query, T> QueryDataAccess for &'query mut T
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_write(component_id)
    }

    fn matches_data(world: &World, entity: EntityId) -> bool {
        world.get::<T>(entity).is_some()
    }
}

impl<'query, T> QueryMutData for &'query mut T
where
    T: Component,
{
    type Item<'world> = &'world mut T;

    fn fetch_mut<'world>(world: &'world mut World, entity: EntityId) -> Option<Self::Item<'world>> {
        world.get_mut::<T>(entity)
    }
}

impl<'query, T> QueryDataAccess for Option<&'query T>
where
    T: Component,
{
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError> {
        let component_id = world.component_id::<T>();
        access.add_read(component_id)
    }

    fn matches_data(_world: &World, _entity: EntityId) -> bool {
        true
    }
}

impl<'query, T> QueryData for Option<&'query T>
where
    T: Component,
{
    type Item<'world> = Option<&'world T>;

    fn fetch<'world>(world: &'world World, entity: EntityId) -> Option<Self::Item<'world>> {
        Some(world.get::<T>(entity))
    }
}

impl QueryDataAccess for EntityId {
    fn update_access(
        _world: &mut World,
        _access: &mut QueryAccess,
    ) -> Result<(), QueryAccessError> {
        Ok(())
    }

    fn matches_data(_world: &World, _entity: EntityId) -> bool {
        true
    }
}

impl QueryData for EntityId {
    type Item<'world> = EntityId;

    fn fetch<'world>(_world: &'world World, entity: EntityId) -> Option<Self::Item<'world>> {
        Some(entity)
    }
}

impl QueryDataAccess for () {
    fn update_access(
        _world: &mut World,
        _access: &mut QueryAccess,
    ) -> Result<(), QueryAccessError> {
        Ok(())
    }

    fn matches_data(_world: &World, _entity: EntityId) -> bool {
        true
    }
}

impl QueryData for () {
    type Item<'world> = ();

    fn fetch<'world>(_world: &'world World, _entity: EntityId) -> Option<Self::Item<'world>> {
        Some(())
    }
}

macro_rules! tuple_query_data {
    ($($name:ident),*) => {
        impl<$($name),*> QueryDataAccess for ($($name,)*)
        where
            $($name: QueryDataAccess,)*
        {
            fn update_access(
                world: &mut World,
                access: &mut QueryAccess,
            ) -> Result<(), QueryAccessError> {
                $($name::update_access(world, access)?;)*
                Ok(())
            }

            fn matches_data(world: &World, entity: EntityId) -> bool {
                true $(&& $name::matches_data(world, entity))*
            }
        }

        impl<$($name),*> QueryData for ($($name,)*)
        where
            $($name: QueryData,)*
        {
            type Item<'world> = ($($name::Item<'world>,)*);

            #[allow(non_snake_case)]
            fn fetch<'world>(world: &'world World, entity: EntityId) -> Option<Self::Item<'world>> {
                Some(($($name::fetch(world, entity)?,)*))
            }
        }
    };
}

tuple_query_data!(A);
tuple_query_data!(A, B);
tuple_query_data!(A, B, C);
tuple_query_data!(A, B, C, D);
