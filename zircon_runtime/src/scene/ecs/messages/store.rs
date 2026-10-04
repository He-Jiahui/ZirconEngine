use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Mutex;

use crate::scene::ecs::channel::OwnedChannel;
use crate::scene::ecs::messages::id::{Message, MessageId};
use crate::scene::ecs::messages::queue::{MessageRetention, MessageRetentionMetrics, Messages};

mod writer_grant;
use writer_grant::MessageActivity;
pub(in crate::scene) use writer_grant::MessageWriterGrant;

/// 按消息 `TypeId` 分区的世界内消息队列注册表。
///
/// `active_channels` 只包含仍需在下一帧推进的类型，因此 `advance_frame` 不必扫描
/// 已注册但当前为空的所有队列。
#[derive(Default)]
pub struct MessageStore {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    advance_operations: HashMap<TypeId, fn(&mut (dyn Any + Send + Sync), u64) -> bool>,
    activity: Mutex<MessageActivity>,
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
        store
            .downcast_ref::<OwnedChannel<Option<Messages<T>>>>()?
            .get()
            .as_ref()
    }

    pub fn messages_mut<T>(&mut self) -> &mut Messages<T>
    where
        T: Message,
    {
        // 首次取得可变队列时同时登记类型名、推进回调和活跃集合。
        self.prepare_writer::<T>();
        self.activity
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .publish::<T>();
        self.stores
            .get_mut(&TypeId::of::<T>())
            .expect("prepared message type must resolve to a slot")
            .downcast_mut::<OwnedChannel<Option<Messages<T>>>>()
            .expect("message store type id must match message slot type")
            .get_mut()
            .get_or_insert_with(Messages::default)
    }

    /// Reserve a stable typed slot under the original exclusive World loan. This is private
    /// preparation: it publishes no queue/name and adds no active maintenance work.
    pub(in crate::scene) fn prepare_writer<T: Message>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.advance_operations
            .entry(type_id)
            .or_insert(advance_message_queue::<T>);
        self.stores
            .entry(type_id)
            .or_insert_with(|| Box::new(OwnedChannel::new(None::<Messages<T>>)));
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
        self.activity
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .active_channels
            .remove(&type_id);
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
        // 双集合交换把本帧新写入的 channel 与上一帧待推进的 channel 分开。
        self.frame = self.frame.saturating_add(1);
        let activity = self
            .activity
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::swap(
            &mut activity.active_channels,
            &mut self.active_channel_spare,
        );
        activity.active_channels.clear();
        self.last_advance_channel_visits = self.active_channel_spare.len();
        for type_id in self.active_channel_spare.drain() {
            let Some(advance) = self.advance_operations.get(&type_id) else {
                continue;
            };
            let Some(store) = self.stores.get_mut(&type_id) else {
                continue;
            };
            if advance(store.as_mut(), self.frame) {
                activity.active_channels.insert(type_id);
            }
        }
    }

    pub fn last_advance_channel_visits(&self) -> usize {
        self.last_advance_channel_visits
    }

    pub fn registered_type_names(&self) -> Vec<&'static str> {
        let activity = self
            .activity
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut names = Vec::with_capacity(activity.type_names.len());
        for name in activity.type_names.values() {
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
    let slot = store
        .downcast_mut::<OwnedChannel<Option<Messages<T>>>>()
        .expect("message store type id must match message slot type")
        .get_mut();
    let Some(messages) = slot.as_mut() else {
        return false;
    };
    messages.advance_frame(frame);
    !messages.is_empty()
}

impl fmt::Debug for MessageStore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MessageStore")
            .field("registered_type_names", &self.registered_type_names())
            .field(
                "active_channel_count",
                &self
                    .activity
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .active_channels
                    .len(),
            )
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
#[path = "store/tests/hash_active_channel_tests.rs"]
mod hash_active_channel_tests;
