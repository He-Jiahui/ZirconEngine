use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};
use std::ptr::addr_of;
use std::sync::Mutex;

use crate::scene::ecs::channel::OwnedChannel;

use super::super::observer::ErasedEventObserver;
use super::{Event, EventChannel, EventObserverId, EventStore, EventTypeId, Events};

/// One selected payload and its immutable observer list; registry ownership stays in the Store.
pub(in crate::scene) struct EventWriterGrant<'world, T> {
    events: &'world mut Events<T>,
    observers: &'world BTreeMap<EventObserverId, Box<dyn ErasedEventObserver>>,
    active_channels: &'world Mutex<BTreeSet<EventTypeId>>,
    event_type_id: EventTypeId,
}

impl<T: Event> EventWriterGrant<'_, T> {
    pub(in crate::scene) fn send(&mut self, event: T) -> bool {
        let mut accepted = true;
        for observer in self.observers.values() {
            accepted &= observer.notify(&event);
        }
        // Publication follows all synchronous observers, including rejecting observers.
        self.activate();
        self.events.send(event);
        accepted
    }

    pub(in crate::scene) fn send_batch<I>(&mut self, events: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let observers = self.observers;
        let written = self.events.send_batch(events.into_iter().inspect(|event| {
            for observer in observers.values() {
                let _ = observer.notify(event);
            }
        }));
        if written > 0 {
            self.activate();
        }
        written
    }

    fn activate(&self) {
        self.active_channels
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(self.event_type_id);
    }
}

impl EventStore {
    /// Projects one channel from the original system grant without borrowing its owning parents.
    ///
    /// # Safety
    /// `store` is the original live grant for `'world`. Its registry, observer collections and
    /// allocation owners stay fixed until every Item drops. The caller admits exclusive writes
    /// for this TypeId and no same-channel readers; different typed payload grants may coexist.
    /// No safe shared/exclusive Store operation may reach this payload while the grant is live.
    pub(in crate::scene) unsafe fn writer_grant<'world, T: Event>(
        store: *mut Self,
        event_type_id: EventTypeId,
    ) -> Option<EventWriterGrant<'world, T>> {
        let channel = unsafe { Self::channel_from_grant(store, event_type_id)? };
        assert_eq!(
            unsafe { addr_of!((*channel).type_id).read() },
            TypeId::of::<T>(),
            "event type id must match event queue type"
        );
        // Shared Box/Any lookup reaches only the allocation owner, never the Events payload.
        let owner = unsafe { &*addr_of!((*channel).events) }
            .as_any()
            .downcast_ref::<OwnedChannel<Events<T>>>()
            .expect("event store type id must match event queue type");
        // The original Box::into_raw pointer supplies permission under the admitted TypeId grant.
        let events = unsafe { &mut *owner.as_ptr() };
        Some(EventWriterGrant {
            events,
            observers: unsafe { &*addr_of!((*channel).observers) },
            active_channels: unsafe { &*addr_of!((*store).active_channels) },
            event_type_id,
        })
    }

    /// # Safety
    /// The same stable-registry grant rules apply. This TypeId has shared read permission and no
    /// mutable queue grant; other channels may have active writers.
    pub(in crate::scene) unsafe fn reader_grant<'world, T: Event>(
        store: *const Self,
        event_type_id: EventTypeId,
    ) -> Option<&'world Events<T>> {
        let channel = unsafe { Self::channel_from_grant(store, event_type_id)? };
        if unsafe { addr_of!((*channel).type_id).read() } != TypeId::of::<T>() {
            return None;
        }
        let owner = unsafe { &*addr_of!((*channel).events) }
            .as_any()
            .downcast_ref::<OwnedChannel<Events<T>>>()?;
        Some(unsafe { &*owner.as_ptr() })
    }

    unsafe fn channel_from_grant(
        store: *const Self,
        event_type_id: EventTypeId,
    ) -> Option<*const EventChannel> {
        // Only Vec metadata is read. Do not create an &[EventChannel] or borrow a channel here.
        let channels = unsafe { &*addr_of!((*store).channels) };
        (event_type_id.index() < channels.len())
            .then(|| unsafe { channels.as_ptr().add(event_type_id.index()) })
    }
}
