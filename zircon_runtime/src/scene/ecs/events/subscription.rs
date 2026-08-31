use crate::scene::ecs::events::{
    Event, EventCursor, EventReadIter, EventReaderLease, EventStore, EventTypeId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventSubscriptionStatus {
    Dormant,
    Connected,
}

/// A manually controlled reader cursor for weak plugin/event dependencies.
///
/// Dormant subscriptions register the event type without adding an active reader.
/// Event producers may still queue into the registered channel; connecting starts
/// the cursor at the current queue position so late plugin activation never
/// replays historical events.
pub struct EventSubscription<T> {
    cursor: EventCursor<T>,
    event_type_id: EventTypeId,
    reader_lease: Option<EventReaderLease>,
    status: EventSubscriptionStatus,
}

impl<T> EventSubscription<T>
where
    T: Event,
{
    pub fn new_dormant(store: &mut EventStore) -> Self {
        Self {
            cursor: EventCursor::default(),
            event_type_id: store.register::<T>(),
            reader_lease: None,
            status: EventSubscriptionStatus::Dormant,
        }
    }

    pub fn event_type_id(&self) -> EventTypeId {
        self.event_type_id
    }

    pub fn status(&self) -> EventSubscriptionStatus {
        self.status
    }

    pub fn is_connected(&self) -> bool {
        self.status == EventSubscriptionStatus::Connected
    }

    pub fn connect(&mut self, store: &mut EventStore) -> bool {
        if self.is_connected() {
            return false;
        }
        let Some(reader_lease) = store.connect_reader(self.event_type_id) else {
            return false;
        };
        self.cursor
            .clear(store.events_by_id::<T>(self.event_type_id));
        self.reader_lease = Some(reader_lease);
        self.status = EventSubscriptionStatus::Connected;
        true
    }

    pub fn disconnect(&mut self, store: &mut EventStore) -> bool {
        if !self.is_connected() {
            return false;
        }
        let Some(mut reader_lease) = self.reader_lease.take() else {
            return false;
        };
        if !store.disconnect_reader(&mut reader_lease) {
            self.reader_lease = Some(reader_lease);
            return false;
        }
        self.cursor.clear(None);
        self.status = EventSubscriptionStatus::Dormant;
        true
    }

    pub fn read<'events>(
        &'events mut self,
        store: &'events EventStore,
    ) -> EventReadIter<'events, T> {
        if !self.is_connected() {
            self.cursor.clear(None);
            return EventReadIter::empty();
        }
        self.cursor
            .read(store.events_by_id::<T>(self.event_type_id))
    }

    pub fn unread_count(&self, store: &EventStore) -> usize {
        if !self.is_connected() {
            return 0;
        }
        self.cursor
            .unread_count(store.events_by_id::<T>(self.event_type_id))
    }
}
