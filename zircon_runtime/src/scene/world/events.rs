use crate::scene::ecs::{
    Event, EventCapacityMetrics, EventObserverHandle, EventPayloadProfile, EventReadIter,
    EventStore, EventSubscription, EventTypeId, EventWriterGrant, Events,
};

use super::World;

impl World {
    pub fn register_event<T>(&mut self)
    where
        T: Event,
    {
        self.events.register::<T>();
    }

    /// 向 World 的分帧事件通道发布一次事件；需要跨帧保留的调用者应显式配置订阅与读取时序。
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

    pub(crate) fn clear_all_events(&mut self) {
        self.events.clear_all();
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

    pub(crate) fn observe_event_delivery<T, F>(
        &mut self,
        callback: F,
    ) -> Option<EventObserverHandle>
    where
        T: Event,
        F: Fn(&T) -> bool + Send + Sync + 'static,
    {
        self.events.observe::<T, F>(callback)
    }

    pub(crate) fn unobserve_event_delivery(&mut self, handle: EventObserverHandle) -> bool {
        self.events.unobserve(handle)
    }

    pub fn read_event_subscription<'events, T>(
        &'events self,
        subscription: &'events mut EventSubscription<T>,
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

    /// # Safety
    /// The original World grant and its fixed event registry must outlive the Item. Access
    /// admission permits only this channel's writer and no readers for the same payload type.
    pub(in crate::scene) unsafe fn event_writer_grant<'world, T: Event>(
        world: *mut Self,
        event_type_id: EventTypeId,
    ) -> EventWriterGrant<'world, T> {
        unsafe { EventStore::writer_grant(std::ptr::addr_of_mut!((*world).events), event_type_id) }
            .expect("event writer channel must be registered before the system runs")
    }

    /// # Safety
    /// The original World grant admits shared reads for this channel, no same-type writes, and
    /// no event registry mutation while any channel Item exists.
    pub(in crate::scene) unsafe fn event_reader_grant<'world, T: Event>(
        world: *const Self,
        event_type_id: EventTypeId,
    ) -> Option<&'world Events<T>> {
        unsafe { EventStore::reader_grant(std::ptr::addr_of!((*world).events), event_type_id) }
    }
}
