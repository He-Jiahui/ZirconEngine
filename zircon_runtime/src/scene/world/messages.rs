use crate::scene::ecs::{
    Message, MessageId, MessageRetention, MessageRetentionMetrics, MessageStore,
    MessageWriterGrant, Messages,
};

use super::World;

impl World {
    /// 向 World 的消息通道追加一条可追踪消息；与事件不同，消费保留期由消息存储策略管理。
    pub fn send_message<T>(&mut self, message: T) -> MessageId<T>
    where
        T: Message,
    {
        self.messages.write(message)
    }

    pub fn messages<T>(&self) -> Option<&Messages<T>>
    where
        T: Message,
    {
        self.messages.messages::<T>()
    }

    pub fn clear_messages<T>(&mut self)
    where
        T: Message,
    {
        self.messages.clear::<T>();
    }

    pub fn configure_message_retention<T>(&mut self, retention: MessageRetention)
    where
        T: Message,
    {
        self.messages.configure_retention::<T>(retention);
    }

    pub fn message_retention_metrics<T>(&self) -> Option<MessageRetentionMetrics>
    where
        T: Message,
    {
        self.messages.retention_metrics::<T>()
    }

    pub fn last_message_advance_channel_visits(&self) -> usize {
        self.messages.last_advance_channel_visits()
    }

    pub(crate) fn advance_messages(&mut self) {
        self.messages.advance_frame();
    }

    pub(crate) fn message_store_mut(&mut self) -> &mut MessageStore {
        &mut self.messages
    }

    /// # Safety
    /// The original World grant keeps the prepared message slots/registry and frame fixed for
    /// the Item lifetime; access admission permits exclusive writes for T and no same-type reads.
    pub(in crate::scene) unsafe fn message_writer_grant<'world, T: Message>(
        world: *mut Self,
    ) -> MessageWriterGrant<'world, T> {
        unsafe { MessageStore::writer_grant(std::ptr::addr_of_mut!((*world).messages)) }
    }

    /// # Safety
    /// The original World grant admits shared T reads, no same-channel writes, and no mutation of
    /// the registry/Box owners while Items exist. Other typed payload grants may remain live.
    pub(in crate::scene) unsafe fn message_reader_grant<'world, T: Message>(
        world: *const Self,
    ) -> Option<&'world Messages<T>> {
        unsafe { MessageStore::reader_grant(std::ptr::addr_of!((*world).messages)) }
    }
}
