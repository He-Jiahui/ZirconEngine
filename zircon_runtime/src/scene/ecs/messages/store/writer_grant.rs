use std::any::{type_name, TypeId};
use std::collections::{HashMap, HashSet};
use std::ptr::addr_of;
use std::sync::Mutex;

use crate::scene::ecs::channel::OwnedChannel;

use super::{Message, MessageId, MessageStore, Messages};

/// The existing publication metadata and active worklist have one synchronized owner.
#[derive(Default)]
pub(super) struct MessageActivity {
    pub(super) type_names: HashMap<TypeId, &'static str>,
    // RUNTIME130_MESSAGE_STORE_HASH_ACTIVE_CHANNELS_BENCH_V1
    pub(super) active_channels: HashSet<TypeId>,
}

impl MessageActivity {
    pub(super) fn publish<T: Message>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.type_names.entry(type_id).or_insert(type_name::<T>());
        self.active_channels.insert(type_id);
    }
}

/// A stable typed pending slot, copied frame, and narrow publication marker only.
pub(in crate::scene) struct MessageWriterGrant<'world, T: Message> {
    slot: &'world mut Option<Messages<T>>,
    activity: &'world Mutex<MessageActivity>,
    frame: u64,
}

impl<T: Message> MessageWriterGrant<'_, T> {
    pub(in crate::scene) fn write(&mut self, message: T) -> MessageId<T> {
        let frame = self.frame;
        self.messages_mut().write_at_frame(message, frame)
    }

    pub(in crate::scene) fn write_batch<I>(&mut self, messages: I) -> Vec<MessageId<T>>
    where
        I: IntoIterator<Item = T>,
    {
        let frame = self.frame;
        self.messages_mut().write_batch_at_frame(messages, frame)
    }

    fn messages_mut(&mut self) -> &mut Messages<T> {
        // Publish before queue creation/iteration, including an empty batch. No payload callback
        // runs under the metadata lock, and merely constructing a grant never publishes it.
        self.activity
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .publish::<T>();
        self.slot.get_or_insert_with(Messages::default)
    }
}

impl MessageStore {
    /// # Safety
    /// `store` is the original live system grant for `'world`; `prepare_writer` has run for T.
    /// Its registry, allocation owners and frame stay fixed until all Items drop. This TypeId is
    /// admitted for exclusive writes with no same-channel readers; other typed grants may coexist.
    /// No safe shared/exclusive Store operation may reach this payload while the grant is live.
    pub(in crate::scene) unsafe fn writer_grant<'world, T: Message>(
        store: *mut Self,
    ) -> MessageWriterGrant<'world, T> {
        let pointer = unsafe { Self::slot_from_grant::<T>(store) }
            .expect("message writer slot must be prepared before the system runs");
        MessageWriterGrant {
            slot: unsafe { &mut *pointer },
            activity: unsafe { &*addr_of!((*store).activity) },
            frame: unsafe { addr_of!((*store).frame).read() },
        }
    }

    /// # Safety
    /// The same stable registry rules apply, with shared read permission for T and no same-channel
    /// mutable grant. Private pending slots still return None until their first write operation.
    pub(in crate::scene) unsafe fn reader_grant<'world, T: Message>(
        store: *const Self,
    ) -> Option<&'world Messages<T>> {
        let pointer = unsafe { Self::slot_from_grant::<T>(store)? };
        unsafe { &*pointer }.as_ref()
    }

    unsafe fn slot_from_grant<T: Message>(store: *const Self) -> Option<*mut Option<Messages<T>>> {
        // Shared HashMap/Box/Any lookup borrows only the owner metadata, not Option or Messages.
        let stores = unsafe { &*addr_of!((*store).stores) };
        let owner = stores
            .get(&TypeId::of::<T>())?
            .downcast_ref::<OwnedChannel<Option<Messages<T>>>>()?;
        // This was produced by Box::into_raw, not by casting a shared payload reference.
        Some(owner.as_ptr())
    }
}
