use std::any::{Any, TypeId, type_name};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::sync::Arc;

use crate::scene::ecs::events::{
    Event, EventCapacityMetrics, EventObserverHandle, EventObserverId, EventPayloadProfile,
    EventReaderLease, EventTypeId, Events,
};

use super::lease::EventReaderLeaseRegistry;
use super::observer::{ErasedEventObserver, TypedEventObserver};

trait ErasedEventQueue: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update_erased(&mut self);
    fn clear_erased(&mut self);
    fn requires_maintenance_erased(&self) -> bool;
    fn capacity_metrics_erased(&self) -> EventCapacityMetrics;
}

impl<T> ErasedEventQueue for Events<T>
where
    T: Event,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn update_erased(&mut self) {
        self.update();
    }

    fn clear_erased(&mut self) {
        self.clear();
    }

    fn requires_maintenance_erased(&self) -> bool {
        self.requires_maintenance()
    }

    fn capacity_metrics_erased(&self) -> EventCapacityMetrics {
        self.capacity_metrics()
    }
}

struct EventChannel {
    type_id: TypeId,
    type_name: &'static str,
    payload_profile: EventPayloadProfile,
    events: Box<dyn ErasedEventQueue>,
    reader_leases: Arc<EventReaderLeaseRegistry>,
    observers: BTreeMap<EventObserverId, Box<dyn ErasedEventObserver>>,
}

impl EventChannel {
    fn is_active(&self) -> bool {
        self.reader_leases.reader_count() > 0
    }
}

#[derive(Default)]
pub struct EventStore {
    channels: Vec<EventChannel>,
    type_ids: HashMap<TypeId, EventTypeId>,
    active_channels: BTreeSet<EventTypeId>,
    last_update_channel_visits: usize,
    next_observer_id: u64,
}

impl EventStore {
    pub fn register<T: Event>(&mut self) -> EventTypeId {
        let type_id = TypeId::of::<T>();
        if let Some(event_type_id) = self.type_ids.get(&type_id).copied() {
            return event_type_id;
        }

        let event_type_id = EventTypeId::new(self.channels.len() as u32);
        self.channels.push(EventChannel {
            type_id,
            type_name: type_name::<T>(),
            payload_profile: EventPayloadProfile::of::<T>(),
            events: Box::<Events<T>>::default(),
            reader_leases: Arc::new(EventReaderLeaseRegistry::new()),
            observers: BTreeMap::new(),
        });
        self.type_ids.insert(type_id, event_type_id);
        event_type_id
    }

    pub fn register_reader<T: Event>(&mut self) -> Option<EventReaderLease> {
        let event_type_id = self.register::<T>();
        self.connect_reader(event_type_id)
    }

    pub fn connect_reader(&mut self, event_type_id: EventTypeId) -> Option<EventReaderLease> {
        self.channel(event_type_id)
            .and_then(|channel| channel.reader_leases.acquire(event_type_id))
    }

    pub fn disconnect_reader(&mut self, lease: &mut EventReaderLease) -> bool {
        let Some(channel) = self.channel(lease.event_type_id()) else {
            return false;
        };
        if !lease.belongs_to(&channel.reader_leases) {
            return false;
        }
        lease.disconnect()
    }

    pub(crate) fn observe<T, F>(&mut self, callback: F) -> Option<EventObserverHandle>
    where
        T: Event,
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        let event_type_id = self.register::<T>();
        let observer_raw = self.next_observer_id.max(1);
        let next_observer_id = observer_raw.checked_add(1)?;
        let channel = self.channel_mut(event_type_id)?;
        if !channel.reader_leases.connect_untracked() {
            return None;
        }
        let observer_id = EventObserverId::new(observer_raw);
        channel.observers.insert(
            observer_id,
            Box::new(TypedEventObserver::<T, F>::new(callback)),
        );
        self.next_observer_id = next_observer_id;
        Some(EventObserverHandle::new(event_type_id, observer_id))
    }

    pub(crate) fn unobserve(&mut self, handle: EventObserverHandle) -> bool {
        let Some(channel) = self.channel_mut(handle.event_type_id()) else {
            return false;
        };
        if !channel.observers.contains_key(&handle.observer_id()) {
            return false;
        }
        channel.observers.remove(&handle.observer_id());
        channel.reader_leases.disconnect_untracked()
    }

    pub fn event_type_id<T: Event>(&self) -> Option<EventTypeId> {
        self.type_ids.get(&TypeId::of::<T>()).copied()
    }

    pub fn event_type_count(&self) -> usize {
        self.channels.len()
    }

    pub fn active_channel_count(&self) -> usize {
        self.active_channels.len()
    }

    pub fn last_update_channel_visits(&self) -> usize {
        self.last_update_channel_visits
    }

    pub fn reader_count(&self, event_type_id: EventTypeId) -> Option<u32> {
        self.channel(event_type_id)
            .map(|channel| channel.reader_leases.reader_count())
    }

    pub fn is_active(&self, event_type_id: EventTypeId) -> bool {
        self.channel(event_type_id)
            .is_some_and(EventChannel::is_active)
    }

    pub fn payload_profile(&self, event_type_id: EventTypeId) -> Option<EventPayloadProfile> {
        self.channel(event_type_id)
            .map(|channel| channel.payload_profile)
    }

    pub fn capacity_metrics(&self, event_type_id: EventTypeId) -> Option<EventCapacityMetrics> {
        self.channel(event_type_id)
            .map(|channel| channel.events.capacity_metrics_erased())
    }

    pub fn events<T: Event>(&self) -> Option<&Events<T>> {
        let event_type_id = self.event_type_id::<T>()?;
        self.events_by_id(event_type_id)
    }

    pub fn events_by_id<T: Event>(&self, event_type_id: EventTypeId) -> Option<&Events<T>> {
        let channel = self.channel(event_type_id)?;
        if channel.type_id != TypeId::of::<T>() {
            return None;
        }
        channel.events.as_any().downcast_ref::<Events<T>>()
    }

    pub fn events_mut<T: Event>(&mut self) -> &mut Events<T> {
        let event_type_id = self.register::<T>();
        self.events_mut_by_id(event_type_id)
    }

    pub fn events_mut_by_id<T: Event>(&mut self, event_type_id: EventTypeId) -> &mut Events<T> {
        self.active_channels.insert(event_type_id);
        let channel = self
            .channel_mut(event_type_id)
            .expect("registered event type id must resolve to a channel");
        assert_eq!(
            channel.type_id,
            TypeId::of::<T>(),
            "event type id must match event queue type"
        );
        channel
            .events
            .as_any_mut()
            .downcast_mut::<Events<T>>()
            .expect("event store type id must match event queue type")
    }

    pub fn send<T: Event>(&mut self, event: T) -> bool {
        let event_type_id = self.register::<T>();
        self.send_by_id(event_type_id, event)
    }

    pub fn send_by_id<T: Event>(&mut self, event_type_id: EventTypeId, event: T) -> bool {
        if self.channel(event_type_id).is_none() {
            return false;
        }
        let observers_accepted = self.notify_event_observers(event_type_id, &event);
        self.events_mut_by_id::<T>(event_type_id).send(event);
        observers_accepted
    }

    pub fn send_batch<T, I>(&mut self, events: I) -> usize
    where
        T: Event,
        I: IntoIterator<Item = T>,
    {
        let event_type_id = self.register::<T>();
        self.send_batch_by_id(event_type_id, events)
    }

    pub fn send_batch_by_id<T, I>(&mut self, event_type_id: EventTypeId, events: I) -> usize
    where
        T: Event,
        I: IntoIterator<Item = T>,
    {
        if self.channel(event_type_id).is_none() {
            return 0;
        }
        let written = {
            let channel = self
                .channel_mut(event_type_id)
                .expect("registered event type id must resolve to a channel");
            assert_eq!(
                channel.type_id,
                TypeId::of::<T>(),
                "event type id must match event queue type"
            );
            let observers = &channel.observers;
            let event_queue = channel
                .events
                .as_any_mut()
                .downcast_mut::<Events<T>>()
                .expect("event store type id must match event queue type");
            event_queue.send_batch(events.into_iter().inspect(|event| {
                for observer in observers.values() {
                    let _ = observer.notify(event);
                }
            }))
        };
        if written > 0 {
            self.active_channels.insert(event_type_id);
        }
        written
    }

    pub fn update<T: Event>(&mut self) {
        let event_type_id = self.register::<T>();
        self.update_by_id::<T>(event_type_id);
    }

    pub fn update_by_id<T: Event>(&mut self, event_type_id: EventTypeId) {
        self.events_mut_by_id::<T>(event_type_id).update();
        self.refresh_active_channel(event_type_id);
    }

    pub fn update_all(&mut self) {
        let active_channels = std::mem::take(&mut self.active_channels);
        self.last_update_channel_visits = active_channels.len();
        for event_type_id in active_channels {
            let keep_active = self.channel_mut(event_type_id).is_some_and(|channel| {
                channel.events.update_erased();
                channel.events.requires_maintenance_erased()
            });
            if keep_active {
                self.active_channels.insert(event_type_id);
            }
        }
    }

    pub(crate) fn clear_all(&mut self) {
        self.active_channels.clear();
        for (index, channel) in self.channels.iter_mut().enumerate() {
            channel.events.clear_erased();
            if channel.events.requires_maintenance_erased() {
                self.active_channels.insert(EventTypeId::new(index as u32));
            }
        }
    }

    pub fn drain<T: Event>(&mut self) -> Vec<T> {
        let event_type_id = self.register::<T>();
        let drained = self.events_mut_by_id::<T>(event_type_id).drain();
        self.refresh_active_channel(event_type_id);
        drained
    }

    pub fn registered_type_names(&self) -> Vec<&'static str> {
        let mut names = Vec::with_capacity(self.channels.len());
        for channel in &self.channels {
            names.push(channel.type_name);
        }
        names.sort_unstable();
        names
    }

    fn channel(&self, event_type_id: EventTypeId) -> Option<&EventChannel> {
        self.channels.get(event_type_id.index())
    }

    fn channel_mut(&mut self, event_type_id: EventTypeId) -> Option<&mut EventChannel> {
        self.channels.get_mut(event_type_id.index())
    }

    fn refresh_active_channel(&mut self, event_type_id: EventTypeId) {
        let keep_active = self
            .channel(event_type_id)
            .is_some_and(|channel| channel.events.requires_maintenance_erased());
        if keep_active {
            self.active_channels.insert(event_type_id);
        } else {
            self.active_channels.remove(&event_type_id);
        }
    }

    fn notify_event_observers<T: Event>(&self, event_type_id: EventTypeId, event: &T) -> bool {
        let Some(channel) = self.channel(event_type_id) else {
            return false;
        };
        let mut accepted = true;
        for observer in channel.observers.values() {
            accepted &= observer.notify(event);
        }
        accepted
    }
}

impl fmt::Debug for EventStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventStore")
            .field("registered_type_names", &self.registered_type_names())
            .finish()
    }
}

impl Clone for EventStore {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl PartialEq for EventStore {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
