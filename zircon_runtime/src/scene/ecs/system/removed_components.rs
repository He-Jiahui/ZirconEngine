use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, Component, RemovedComponentEvents, RemovedComponentReader, SystemParam,
    SystemParamAccess, SystemParamError,
};
use crate::scene::{EntityId, World};

pub struct RemovedComponentsParam<T>(PhantomData<fn() -> T>);

pub struct RemovedComponents<'world, T>
where
    T: Component,
{
    events: &'world RemovedComponentEvents,
    reader: &'world mut RemovedComponentReader<T>,
}

impl<T> RemovedComponents<'_, T>
where
    T: Component,
{
    pub fn read(&mut self) -> impl Iterator<Item = EntityId> {
        self.reader.read(self.events).into_iter()
    }

    pub fn len(&self) -> usize {
        self.reader.len(self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.reader.is_empty(self.events)
    }

    pub fn clear(&mut self) {
        self.reader.clear(self.events);
    }
}

impl<T> SystemParam for RemovedComponentsParam<T>
where
    T: Component,
{
    type State = RemovedComponentReader<T>;
    type Item<'world> = RemovedComponents<'world, T>;

    fn init_state(
        _world: &mut World,
        _access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        Ok(RemovedComponentReader::default())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &*world;
        RemovedComponents {
            events: world.removed_component_events(),
            reader: state,
        }
    }
}
