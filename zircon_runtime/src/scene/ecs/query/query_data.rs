use crate::scene::ecs::{Component, Mut, QueryAccess, QueryAccessError, Ref};
use crate::scene::{EntityId, World};

pub trait QueryDataAccess {
    fn update_access(world: &mut World, access: &mut QueryAccess) -> Result<(), QueryAccessError>;
    fn matches_data(world: &World, entity: EntityId) -> bool;
}

pub trait QueryData: QueryDataAccess {
    type Item<'world>;

    fn fetch<'world>(world: &'world World, entity: EntityId) -> Option<Self::Item<'world>>;
    fn fetch_with_ticks<'world>(
        world: &'world World,
        entity: EntityId,
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        Self::fetch(world, entity)
    }
}

pub trait QueryMutData: QueryDataAccess {
    type Item<'world>;

    fn fetch_mut<'world>(world: &'world mut World, entity: EntityId) -> Option<Self::Item<'world>>;
    fn fetch_mut_with_ticks<'world>(
        world: &'world mut World,
        entity: EntityId,
        _ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        Self::fetch_mut(world, entity)
    }
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

impl<'query, T> QueryDataAccess for Ref<'query, T>
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

impl<'query, T> QueryData for Ref<'query, T>
where
    T: Component,
{
    type Item<'world> = Ref<'world, T>;

    fn fetch<'world>(world: &'world World, entity: EntityId) -> Option<Self::Item<'world>> {
        Self::fetch_with_ticks(
            world,
            entity,
            crate::scene::ecs::ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    fn fetch_with_ticks<'world>(
        world: &'world World,
        entity: EntityId,
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let value = world.get::<T>(entity)?;
        let component_ticks = world.component_change_ticks::<T>(entity)?;
        Some(Ref::new(value, component_ticks, ticks))
    }
}

impl<'query, T> QueryDataAccess for Mut<'query, T>
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

impl<'query, T> QueryMutData for Mut<'query, T>
where
    T: Component,
{
    type Item<'world> = Mut<'world, T>;

    fn fetch_mut<'world>(world: &'world mut World, entity: EntityId) -> Option<Self::Item<'world>> {
        Self::fetch_mut_with_ticks(
            world,
            entity,
            crate::scene::ecs::ChangeTickWindow::all(world.read_change_tick()),
        )
    }

    fn fetch_mut_with_ticks<'world>(
        world: &'world mut World,
        entity: EntityId,
        ticks: crate::scene::ecs::ChangeTickWindow,
    ) -> Option<Self::Item<'world>> {
        let component_ticks = world.component_change_ticks::<T>(entity)?;
        let value = world.get_mut::<T>(entity)?;
        Some(Mut::new(value, component_ticks, ticks))
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

            #[allow(non_snake_case)]
            fn fetch_with_ticks<'world>(
                world: &'world World,
                entity: EntityId,
                ticks: crate::scene::ecs::ChangeTickWindow,
            ) -> Option<Self::Item<'world>> {
                Some(($($name::fetch_with_ticks(world, entity, ticks)?,)*))
            }
        }
    };
}

tuple_query_data!(A);
tuple_query_data!(A, B);
tuple_query_data!(A, B, C);
tuple_query_data!(A, B, C, D);
