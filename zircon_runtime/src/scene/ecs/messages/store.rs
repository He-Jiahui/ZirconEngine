use std::any::{Any, TypeId, type_name};
use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::scene::ecs::messages::id::{Message, MessageId};
use crate::scene::ecs::messages::queue::{MessageRetention, MessageRetentionMetrics, Messages};

#[derive(Default)]
pub struct MessageStore {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    type_names: HashMap<TypeId, &'static str>,
    advance_operations: HashMap<TypeId, fn(&mut (dyn Any + Send + Sync), u64) -> bool>,
    // RUNTIME130_MESSAGE_STORE_HASH_ACTIVE_CHANNELS_BENCH_V1
    active_channels: HashSet<TypeId>,
    active_channel_spare: HashSet<TypeId>,
    last_advance_channel_visits: usize,
    frame: u64,
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
        self.advance_operations
            .entry(type_id)
            .or_insert(advance_message_queue::<T>);
        self.active_channels.insert(type_id);
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
        let frame = self.frame;
        self.messages_mut::<T>().write_at_frame(message, frame)
    }

    pub fn write_batch<T, I>(&mut self, messages: I) -> Vec<MessageId<T>>
    where
        T: Message,
        I: IntoIterator<Item = T>,
    {
        let frame = self.frame;
        self.messages_mut::<T>()
            .write_batch_at_frame(messages, frame)
    }

    pub fn clear<T>(&mut self)
    where
        T: Message,
    {
        let type_id = TypeId::of::<T>();
        self.messages_mut::<T>().clear();
        self.active_channels.remove(&type_id);
    }

    pub fn configure_retention<T>(&mut self, retention: MessageRetention)
    where
        T: Message,
    {
        self.messages_mut::<T>().set_retention(retention);
    }

    pub fn retention_metrics<T>(&self) -> Option<MessageRetentionMetrics>
    where
        T: Message,
    {
        self.messages::<T>().map(Messages::retention_metrics)
    }

    pub fn advance_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
        std::mem::swap(&mut self.active_channels, &mut self.active_channel_spare);
        self.active_channels.clear();
        self.last_advance_channel_visits = self.active_channel_spare.len();
        for type_id in self.active_channel_spare.drain() {
            let Some(advance) = self.advance_operations.get(&type_id) else {
                continue;
            };
            let Some(store) = self.stores.get_mut(&type_id) else {
                continue;
            };
            if advance(store.as_mut(), self.frame) {
                self.active_channels.insert(type_id);
            }
        }
    }

    pub fn last_advance_channel_visits(&self) -> usize {
        self.last_advance_channel_visits
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

fn advance_message_queue<T>(store: &mut (dyn Any + Send + Sync), frame: u64) -> bool
where
    T: Message,
{
    let messages = store
        .downcast_mut::<Messages<T>>()
        .expect("message store type id must match message queue type");
    messages.advance_frame(frame);
    !messages.is_empty()
}

impl fmt::Debug for MessageStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MessageStore")
            .field("registered_type_names", &self.registered_type_names())
            .field("active_channel_count", &self.active_channels.len())
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

#[cfg(test)]
#[path = "store/hash_active_channel_tests.rs"]
mod hash_active_channel_tests;
