use crate::scene::ecs::{
    Message, MessageId, MessageRetention, MessageRetentionMetrics, MessageStore, Messages,
};

use super::World;

impl World {
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
}
