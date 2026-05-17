use crate::scene::ecs::{Message, MessageId, MessageStore, Messages};

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

    pub(crate) fn message_store_mut(&mut self) -> &mut MessageStore {
        &mut self.messages
    }
}
