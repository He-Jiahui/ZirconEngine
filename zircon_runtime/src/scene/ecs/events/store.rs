use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::scene::ecs::events::{
    Event, EventCapacityMetrics, EventPayloadProfile, EventTypeId, Events,
};

trait ErasedEventQueue: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn update_erased(&mut self);
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

    fn capacity_metrics_erased(&self) -> EventCapacityMetrics {
        self.capacity_metrics()
    }
}

struct EventChannel {
    type_id: TypeId,
    type_name: &'static str,
    payload_profile: EventPayloadProfile,
    events: Box<dyn ErasedEventQueue>,
    reader_count: u32,
}

impl EventChannel {
    fn is_active(&self) -> bool {
        self.reader_count > 0
    }
}

#[derive(Default)]
pub struct EventStore {
    channels: Vec<EventChannel>,
    type_ids: HashMap<TypeId, EventTypeId>,
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
            reader_count: 0,
        });
        self.type_ids.insert(type_id, event_type_id);
        event_type_id
    }

    pub fn register_reader<T: Event>(&mut self) -> EventTypeId {
        let event_type_id = self.register::<T>();
        self.connect_reader(event_type_id);
        event_type_id
    }

    pub fn connect_reader(&mut self, event_type_id: EventTypeId) -> bool {
        let Some(channel) = self.channel_mut(event_type_id) else {
            return false;
        };
        channel.reader_count = channel.reader_count.saturating_add(1);
        true
    }

    pub fn disconnect_reader(&mut self, event_type_id: EventTypeId) -> bool {
        let Some(channel) = self.channel_mut(event_type_id) else {
            return false;
        };
        if channel.reader_count == 0 {
            return false;
        }
        channel.reader_count -= 1;
        true
    }

    pub fn event_type_id<T: Event>(&self) -> Option<EventTypeId> {
        self.type_ids.get(&TypeId::of::<T>()).copied()
    }

    pub fn event_type_count(&self) -> usize {
        self.channels.len()
    }

    pub fn reader_count(&self, event_type_id: EventTypeId) -> Option<u32> {
        self.channel(event_type_id)
            .map(|channel| channel.reader_count)
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
        self.events_mut_by_id::<T>(event_type_id).send(event);
        true
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
        self.events_mut_by_id::<T>(event_type_id).send_batch(events)
    }

    pub fn update<T: Event>(&mut self) {
        self.events_mut::<T>().update();
    }

    pub fn update_by_id<T: Event>(&mut self, event_type_id: EventTypeId) {
        self.events_mut_by_id::<T>(event_type_id).update();
    }

    pub fn update_all(&mut self) {
        for channel in &mut self.channels {
            channel.events.update_erased();
        }
    }

    pub fn drain<T: Event>(&mut self) -> Vec<T> {
        self.events_mut::<T>().drain()
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
