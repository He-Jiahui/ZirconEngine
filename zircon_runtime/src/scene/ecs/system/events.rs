use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, EventCursor, EventReadIter, EventStore, EventTypeId, Events, SystemParam,
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
    event_type_id: EventTypeId,
    _marker: PhantomData<fn() -> T>,
}

impl<'world, T> EventReader<'world, T> {
    pub fn iter(&mut self) -> EventReadIter<'_, T> {
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
    pub fn send(&mut self, event: T) -> bool {
        self.store.send_by_id(self.event_type_id, event)
    }

    pub fn send_batch<I>(&mut self, events: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        self.store
            .send_batch_by_id::<T, I>(self.event_type_id, events)
    }
}

impl<T> SystemParam for EventReaderParam<T>
where
    T: 'static + Send + Sync,
{
    type State = EventReaderState<T>;
    type Item<'world> = EventReader<'world, T>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_event_read::<T>()?;
        Ok(EventReaderState {
            cursor: EventCursor::default(),
            event_type_id: world.event_store_mut().register_reader::<T>(),
        })
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &*world;
        EventReader {
            cursor: &mut state.cursor,
            events: world.event_store().events_by_id::<T>(state.event_type_id),
        }
    }
}

impl<T> SystemParam for EventWriterParam<T>
where
    T: 'static + Send + Sync,
{
    type State = EventWriterState;
    type Item<'world> = EventWriter<'world, T>;

    fn init_state(
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_event_write::<T>()?;
        Ok(EventWriterState {
            event_type_id: world.event_store_mut().register::<T>(),
        })
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &mut *world;
        EventWriter {
            store: world.event_store_mut(),
            event_type_id: state.event_type_id,
            _marker: PhantomData,
        }
    }
}

pub struct EventReaderState<T> {
    cursor: EventCursor<T>,
    event_type_id: EventTypeId,
}

pub struct EventWriterState {
    event_type_id: EventTypeId,
}
