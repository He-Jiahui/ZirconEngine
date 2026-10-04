use std::marker::PhantomData;

use crate::scene::ecs::{
    ChangeTickWindow, Message, MessageCursor, MessageId, MessageReadIter, MessageWriterGrant,
    SystemParam, SystemParamAccess, SystemParamError,
};
use crate::scene::World;

pub struct MessageReaderParam<T>(PhantomData<fn() -> T>);

/// 消息写入参数只借用当前 World 的类型通道，不保留跨帧队列所有权。
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
    channel: MessageWriterGrant<'world, T>,
}

impl<'world, T> MessageReader<'world, T>
where
    T: Message,
{
    pub fn read<'reader>(&'reader mut self) -> MessageReadIter<'reader, T> {
        self.cursor.read(self.messages)
    }

    pub fn unread_count(&self) -> usize {
        self.cursor.unread_count(self.messages)
    }

    pub fn len(&self) -> usize {
        self.unread_count()
    }

    pub fn is_empty(&self) -> bool {
        self.unread_count() == 0
    }

    pub fn clear(&mut self) {
        self.cursor.clear(self.messages);
    }

    pub fn dropped_count(&self) -> u64 {
        self.cursor.dropped_count()
    }
}

impl<T> MessageWriter<'_, T>
where
    T: Message,
{
    pub fn write(&mut self, message: T) -> MessageId<T> {
        self.channel.write(message)
    }

    pub fn write_batch<I>(&mut self, messages: I) -> Vec<MessageId<T>>
    where
        I: IntoIterator<Item = T>,
    {
        self.channel.write_batch(messages)
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
        MessageReader {
            cursor: state,
            messages: unsafe { World::message_reader_grant::<T>(world) },
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
        world: &mut World,
        access: &mut SystemParamAccess,
    ) -> Result<Self::State, SystemParamError> {
        access.add_message_write::<T>()?;
        world.message_store_mut().prepare_writer::<T>();
        Ok(())
    }

    unsafe fn get_param<'world>(
        world: *mut World,
        _state: &'world mut Self::State,
        _ticks: ChangeTickWindow,
    ) -> Self::Item<'world> {
        MessageWriter {
            channel: unsafe { World::message_writer_grant::<T>(world) },
        }
    }
}
