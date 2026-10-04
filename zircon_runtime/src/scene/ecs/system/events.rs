use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, EventCursor, EventReadIter, EventReaderLease, EventTypeId, EventWriterGrant,
    Events, SystemParam, SystemParamAccess, SystemParamError,
};
use crate::scene::World;

/// 由系统参数状态持有 reader lease，并在 retire_state 中交还给所属 World。
pub struct EventReaderParam<T>(PhantomData<fn() -> T>);

pub struct EventWriterParam<T>(PhantomData<fn() -> T>);

pub struct EventReader<'world, T> {
    cursor: &'world mut EventCursor<T>,
    events: Option<&'world Events<T>>,
}

pub struct EventWriter<'world, T> {
    channel: EventWriterGrant<'world, T>,
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
        self.channel.send(event)
    }

    pub fn send_batch<I>(&mut self, events: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        self.channel.send_batch(events)
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
        let reader_lease = world.event_store_mut().register_reader::<T>().ok_or(
            SystemParamError::EventReaderLeaseExhausted {
                type_name: std::any::type_name::<T>(),
            },
        )?;
        Ok(EventReaderState {
            cursor: EventCursor::default(),
            event_type_id: reader_lease.event_type_id(),
            reader_lease: Some(reader_lease),
        })
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        EventReader {
            cursor: &mut state.cursor,
            events: unsafe { World::event_reader_grant::<T>(world, state.event_type_id) },
        }
    }

    fn retire_state(world: &mut World, state: &mut Self::State) {
        let Some(mut reader_lease) = state.reader_lease.take() else {
            return;
        };
        let disconnected = world.event_store_mut().disconnect_reader(&mut reader_lease);
        debug_assert!(
            disconnected,
            "event reader lease must belong to its active world"
        );
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
        EventWriter {
            channel: unsafe { World::event_writer_grant::<T>(world, state.event_type_id) },
        }
    }
}

pub struct EventReaderState<T> {
    cursor: EventCursor<T>,
    event_type_id: EventTypeId,
    reader_lease: Option<EventReaderLease>,
}

pub struct EventWriterState {
    event_type_id: EventTypeId,
}
