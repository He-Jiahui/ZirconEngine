use crate::scene::ecs::{
    Event, EventCapacityMetrics, EventPayloadProfile, EventReadIter, EventStore, EventSubscription,
    EventTypeId, Events,
};

use super::World;

impl World {
    pub fn register_event<T>(&mut self)
    where
        T: Event,
    {
        self.events.register::<T>();
    }

    pub fn send_event<T>(&mut self, event: T) -> bool
    where
        T: Event,
    {
        self.events.send(event)
    }

    pub fn update_events<T>(&mut self)
    where
        T: Event,
    {
        self.events.update::<T>();
    }

    pub fn update_all_events(&mut self) {
        self.events.update_all();
    }

    pub fn clear_events<T>(&mut self)
    where
        T: Event,
    {
        self.events.events_mut::<T>().clear();
    }

    pub fn events<T>(&self) -> Option<&Events<T>>
    where
        T: Event,
    {
        self.events.events::<T>()
    }

    pub fn event_type_id<T>(&self) -> Option<EventTypeId>
    where
        T: Event,
    {
        self.events.event_type_id::<T>()
    }

    pub fn event_reader_count(&self, event_type_id: EventTypeId) -> Option<u32> {
        self.events.reader_count(event_type_id)
    }

    pub fn event_payload_profile(&self, event_type_id: EventTypeId) -> Option<EventPayloadProfile> {
        self.events.payload_profile(event_type_id)
    }

    pub fn event_capacity_metrics(
        &self,
        event_type_id: EventTypeId,
    ) -> Option<EventCapacityMetrics> {
        self.events.capacity_metrics(event_type_id)
    }

    pub fn register_dormant_event_subscription<T>(&mut self) -> EventSubscription<T>
    where
        T: Event,
    {
        EventSubscription::new_dormant(&mut self.events)
    }

    pub fn connect_event_subscription<T>(&mut self, subscription: &mut EventSubscription<T>) -> bool
    where
        T: Event,
    {
        subscription.connect(&mut self.events)
    }

    pub fn disconnect_event_subscription<T>(
        &mut self,
        subscription: &mut EventSubscription<T>,
    ) -> bool
    where
        T: Event,
    {
        subscription.disconnect(&mut self.events)
    }

    pub fn read_event_subscription<'events, T>(
        &'events self,
        subscription: &mut EventSubscription<T>,
    ) -> EventReadIter<'events, T>
    where
        T: Event,
    {
        subscription.read(&self.events)
    }

    pub(crate) fn event_store(&self) -> &EventStore {
        &self.events
    }

    pub(crate) fn event_store_mut(&mut self) -> &mut EventStore {
        &mut self.events
    }
}
