use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, EventCursor, EventReadIter, EventStore, Events, SystemParam,
    SystemParamAccess, SystemParamError,
};
use crate::scene::World;

pub struct EventReaderParam<T>(PhantomData<fn() -> T>);

pub struct EventWriterParam<T>(PhantomData<fn() -> T>);

pub struct EventReader<'world, T> {
    cursor: &'world mut EventCursor<T>,
    events: Option<&'world Events<T>>,
}

pub struct EventWriter<'world, T> {
    store: &'world mut EventStore,
    _marker: PhantomData<fn() -> T>,
}

impl<'world, T> EventReader<'world, T> {
    pub fn iter(&mut self) -> EventReadIter<'world, T> {
        self.cursor.read(self.events)
    }

    pub fn len(&self) -> usize {
        self.unread_count()
    }

    pub fn unread_count(&self) -> usize {
        self.cursor.unread_count(self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.cursor.clear(self.events);
    }
}

impl<T> EventWriter<'_, T>
where
    T: 'static + Send + Sync,
{
    pub fn send(&mut self, event: T) {
        self.store.send(event);
    }

    pub fn send_batch<I>(&mut self, events: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        self.store.send_batch::<T, I>(events)
    }
}

impl<T> SystemParam for EventReaderParam<T>
where
    T: 'static + Send + Sync,
{
    type State = EventCursor<T>;
    type Item<'world> = EventReader<'world, T>;

    fn init_state(
        _world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_event_read::<T>()?;
        Ok(EventCursor::default())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &*world;
        EventReader {
            cursor: state,
            events: world.events::<T>(),
        }
    }
}

impl<T> SystemParam for EventWriterParam<T>
where
    T: 'static + Send + Sync,
{
    type State = ();
    type Item<'world> = EventWriter<'world, T>;

    fn init_state(
        _world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_event_write::<T>()?;
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &mut *world;
        EventWriter {
            store: world.event_store_mut(),
            _marker: PhantomData,
        }
    }
}
