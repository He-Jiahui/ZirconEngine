use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::messages::id::{Message, MessageId};
use crate::scene::ecs::messages::queue::Messages;

#[derive(Default)]
pub struct MessageStore {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    type_names: HashMap<TypeId, &'static str>,
}

impl MessageStore {
    pub fn messages<T>(&self) -> Option<&Messages<T>>
    where
        T: Message,
    {
        let store = self.stores.get(&TypeId::of::<T>())?;
        store.downcast_ref::<Messages<T>>()
    }

    pub fn messages_mut<T>(&mut self) -> &mut Messages<T>
    where
        T: Message,
    {
        let type_id = TypeId::of::<T>();
        self.type_names.entry(type_id).or_insert(type_name::<T>());
        self.stores
            .entry(type_id)
            .or_insert_with(|| Box::<Messages<T>>::default())
            .downcast_mut::<Messages<T>>()
            .expect("message store type id must match message queue type")
    }

    pub fn write<T>(&mut self, message: T) -> MessageId<T>
    where
        T: Message,
    {
        self.messages_mut::<T>().write(message)
    }

    pub fn write_batch<T, I>(&mut self, messages: I) -> Vec<MessageId<T>>
    where
        T: Message,
        I: IntoIterator<Item = T>,
    {
        self.messages_mut::<T>().write_batch(messages)
    }

    pub fn clear<T>(&mut self)
    where
        T: Message,
    {
        self.messages_mut::<T>().clear();
    }

    pub fn registered_type_names(&self) -> Vec<&'static str> {
        let mut names = Vec::with_capacity(self.type_names.len());
        for name in self.type_names.values() {
            names.push(*name);
        }
        names.sort_unstable();
        names
    }
}

impl fmt::Debug for MessageStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MessageStore")
            .field("registered_type_names", &self.registered_type_names())
            .finish()
    }
}

impl Clone for MessageStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for MessageStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
