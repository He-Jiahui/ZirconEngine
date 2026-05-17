use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, Message, MessageCursor, MessageId, MessageReadIter, MessageStore,
    SystemParam, SystemParamAccess, SystemParamError,
};
use crate::scene::World;

pub struct MessageReaderParam<T>(PhantomData<fn() -> T>);

pub struct MessageWriterParam<T>(PhantomData<fn() -> T>);

pub struct MessageReader<'world, T>
where
    T: Message,
{
    cursor: &'world mut MessageCursor<T>,
    messages: Option<&'world crate::scene::ecs::Messages<T>>,
}

pub struct MessageWriter<'world, T>
where
    T: Message,
{
    store: &'world mut MessageStore,
    _marker: PhantomData<fn() -> T>,
}

impl<'world, T> MessageReader<'world, T>
where
    T: Message,
{
    pub fn read(&mut self) -> MessageReadIter<'world, T> {
        self.cursor.read(self.messages)
    }

    pub fn unread_count(&self) -> usize {
        self.cursor.unread_count(self.messages)
    }

    pub fn is_empty(&self) -> bool {
        self.unread_count() == 0
    }
}

impl<T> MessageWriter<'_, T>
where
    T: Message,
{
    pub fn write(&mut self, message: T) -> MessageId<T> {
        self.store.write(message)
    }
}

impl<T> SystemParam for MessageReaderParam<T>
where
    T: Message,
{
    type State = MessageCursor<T>;
    type Item<'world> = MessageReader<'world, T>;

    fn init_state(
        _world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_message_read::<T>()?;
        Ok(MessageCursor::default())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &*world;
        MessageReader {
            cursor: state,
            messages: world.messages::<T>(),
        }
    }
}

impl<T> SystemParam for MessageWriterParam<T>
where
    T: Message,
{
    type State = ();
    type Item<'world> = MessageWriter<'world, T>;

    fn init_state(
        _world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_message_write::<T>()?;
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        let world = &mut *world;
        MessageWriter {
            store: world.message_store_mut(),
            _marker: PhantomData,
        }
    }
}
