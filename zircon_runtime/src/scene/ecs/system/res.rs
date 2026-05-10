use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::scene::ecs::{
    ChangeTickWindow, ComponentTicks, Resource, SystemParam, SystemParamAccess, SystemParamError,
};
use crate::scene::World;

pub struct ResParam<T>(PhantomData<fn() -> T>);

pub struct ResMutParam<T>(PhantomData<fn() -> T>);

pub struct Res<'world, T> {
    value: &'world T,
    ticks: ComponentTicks,
    window: ChangeTickWindow,
}

pub struct ResMut<'world, T> {
    value: &'world mut T,
    ticks: ComponentTicks,
    window: ChangeTickWindow,
}

impl<T> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> Deref for ResMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for ResMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T> Res<'_, T> {
    pub fn is_added(&self) -> bool {
        self.ticks.is_added(self.window)
    }

    pub fn is_changed(&self) -> bool {
        self.ticks.is_changed(self.window)
    }

    pub fn last_changed(&self) -> crate::scene::ecs::ChangeTick {
        self.ticks.changed()
    }
}

impl<T> ResMut<'_, T> {
    pub fn is_added(&self) -> bool {
        self.ticks.is_added(self.window)
    }

    pub fn is_changed(&self) -> bool {
        self.ticks.is_changed(self.window)
    }

    pub fn last_changed(&self) -> crate::scene::ecs::ChangeTick {
        self.ticks.changed()
    }
}

impl<T> SystemParam for ResParam<T>
where
    T: Resource,
{
    type State = ();
    type Item<'world> = Res<'world, T>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        let resource_id = world.resource_id::<T>();
        access.add_resource_read(resource_id)?;
        if world.get_resource::<T>().is_none() {
            return Err(SystemParamError::MissingResource {
                type_name: std::any::type_name::<T>(),
            });
        }
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &*world;
        let value = world.resource::<T>();
        let resource_ticks = world
            .resource_change_ticks::<T>()
            .expect("resource param must have registered change ticks");
        Res {
            value,
            ticks: resource_ticks,
            window: ticks,
        }
    }
}

impl<T> SystemParam for ResMutParam<T>
where
    T: Resource,
{
    type State = ();
    type Item<'world> = ResMut<'world, T>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        let resource_id = world.resource_id::<T>();
        access.add_resource_write(resource_id)?;
        if world.get_resource::<T>().is_none() {
            return Err(SystemParamError::MissingResource {
                type_name: std::any::type_name::<T>(),
            });
        }
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &mut *world;
        let (value, resource_ticks) = world
            .resource_mut_with_ticks::<T>()
            .expect("resource mut param must reference an existing resource");
        ResMut {
            value,
            ticks: resource_ticks,
            window: ticks,
        }
    }
}

impl<T> SystemParam for Option<ResParam<T>>
where
    T: Resource,
{
    type State = ();
    type Item<'world> = Option<Res<'world, T>>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        let resource_id = world.resource_id::<T>();
        access.add_resource_read(resource_id)?;
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &*world;
        let value = world.get_resource::<T>()?;
        let resource_ticks = world.resource_change_ticks::<T>()?;
        Some(Res {
            value,
            ticks: resource_ticks,
            window: ticks,
        })
    }
}

impl<T> SystemParam for Option<ResMutParam<T>>
where
    T: Resource,
{
    type State = ();
    type Item<'world> = Option<ResMut<'world, T>>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        let resource_id = world.resource_id::<T>();
        access.add_resource_write(resource_id)?;
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &mut *world;
        let (value, resource_ticks) = world.resource_mut_with_ticks::<T>()?;
        Some(ResMut {
            value,
            ticks: resource_ticks,
            window: ticks,
        })
    }
}
