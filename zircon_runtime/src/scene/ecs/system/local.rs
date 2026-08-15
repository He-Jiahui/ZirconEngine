use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use crate::scene::ecs::{
    ChangeTickWindow, SystemParam, SystemParamAccess, SystemParamError, WorldlessSystemParam,
};
use crate::scene::World;

pub struct LocalParam<T>(PhantomData<fn() -> T>);

pub struct Local<'world, T> {
    value: &'world mut T,
}

impl<T> Deref for Local<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

impl<T> DerefMut for Local<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value
    }
}

impl<T> SystemParam for LocalParam<T>
where
    T: Default + Send + 'static,
{
    type State = T;
    type Item<'world> = Local<'world, T>;

    fn init_state(
        _world: &mut World,
        _access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        Ok(T::default())
    }

    unsafe fn get_param<'world>(
        _world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        Local { value: state }
    }
}

impl<T> super::system_param::worldless_private::Sealed for LocalParam<T> where
    T: Default + Send + 'static
{
}

impl<T> WorldlessSystemParam for LocalParam<T>
where
    T: Default + Send + 'static,
{
    fn get_param_without_world<'world>(state: &'world mut Self::State) -> Self::Item<'world> {
        Local { value: state }
    }
}
