use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::mem::size_of;

pub trait Event: 'static + Send + Sync {}

impl<T> Event for T where T: 'static + Send + Sync {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EventTypeId(u32);

impl EventTypeId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    fn index(self) -> usize {
        self.0 as usize
    }
}

pub const EVENT_INLINE_PAYLOAD_MAX_BYTES: usize = 128;
pub const EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES: u32 = 8;
const EVENT_CAPACITY_LOW_WATER_DIVISOR: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPayloadStorage {
    Inline,
    IndirectRecommended,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventPayloadProfile {
    size_bytes: usize,
    storage: EventPayloadStorage,
}

impl EventPayloadProfile {
    pub const fn for_size(size_bytes: usize) -> Self {
        let storage = if size_bytes > EVENT_INLINE_PAYLOAD_MAX_BYTES {
            EventPayloadStorage::IndirectRecommended
        } else {
            EventPayloadStorage::Inline
        };
        Self {
            size_bytes,
            storage,
        }
    }

    pub fn of<T>() -> Self {
        Self::for_size(size_of::<T>())
    }

    pub const fn size_bytes(self) -> usize {
        self.size_bytes
    }

    pub const fn storage(self) -> EventPayloadStorage {
        self.storage
    }

    pub const fn requires_indirection(self) -> bool {
        match self.storage {
            EventPayloadStorage::Inline => false,
            EventPayloadStorage::IndirectRecommended => true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventCapacityMetrics {
    pub current_len: usize,
    pub next_len: usize,
    pub current_capacity: usize,
    pub next_capacity: usize,
    pub high_water_len: usize,
    pub low_water_frames: u32,
    pub shrink_count: u64,
}

impl EventCapacityMetrics {
    pub const fn retained_capacity(self) -> usize {
        if self.current_capacity > self.next_capacity {
            self.current_capacity
        } else {
            self.next_capacity
        }
    }

    pub const fn queued_len(self) -> usize {
        self.current_len + self.next_len
    }
}

#[derive(Clone, Debug)]
pub struct Events<T> {
    current: Vec<T>,
    next: Vec<T>,
    generation: u64,
    high_water_len: usize,
    low_water_frames: u32,
    capacity_shrink_count: u64,
}

impl<T> PartialEq for Events<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.current == other.current && self.next == other.next
    }
}

impl<T> Eq for Events<T> where T: Eq {}

impl<T> Default for Events<T> {
    fn default() -> Self {
        Self {
            current: Vec::new(),
            next: Vec::new(),
            generation: 0,
            high_water_len: 0,
            low_water_frames: 0,
            capacity_shrink_count: 0,
        }
    }
}

impl<T> Events<T> {
    pub fn send(&mut self, event: T) {
        self.next.push(event);
        self.record_next_queue_len();
    }

    pub fn send_batch<I>(&mut self, events: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let events = events.into_iter();
        let (lower_bound, _) = events.size_hint();
        self.next.reserve(lower_bound);

        let mut written = 0;
        for event in events {
            self.next.push(event);
            written += 1;
        }
        if written > 0 {
            self.record_next_queue_len();
        }
        written
    }

    pub fn update(&mut self) {
        self.current.clear();
        std::mem::swap(&mut self.current, &mut self.next);
        self.generation = self.generation.saturating_add(1);
        self.update_capacity_policy();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.current.iter()
    }

    pub fn iter_from(&self, start: usize) -> std::slice::Iter<'_, T> {
        self.current[start.min(self.current.len())..].iter()
    }

    pub fn drain(&mut self) -> Vec<T> {
        std::mem::take(&mut self.current)
    }

    pub fn clear(&mut self) {
        self.current.clear();
        self.next.clear();
        self.generation = self.generation.saturating_add(1);
        self.high_water_len = 0;
        self.low_water_frames = 0;
    }

    pub fn len(&self) -> usize {
        self.current.len()
    }

    pub fn is_empty(&self) -> bool {
        self.current.is_empty()
    }

    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    pub fn capacity_metrics(&self) -> EventCapacityMetrics {
        EventCapacityMetrics {
            current_len: self.current.len(),
            next_len: self.next.len(),
            current_capacity: self.current.capacity(),
            next_capacity: self.next.capacity(),
            high_water_len: self.high_water_len,
            low_water_frames: self.low_water_frames,
            shrink_count: self.capacity_shrink_count,
        }
    }

    fn record_next_queue_len(&mut self) {
        self.high_water_len = self.high_water_len.max(self.next.len());
        self.low_water_frames = 0;
    }

    fn update_capacity_policy(&mut self) {
        let active_len = self.current.len().max(self.next.len());
        self.high_water_len = self.high_water_len.max(active_len);
        self.reserve_next_for_high_water();

        let retained_capacity = self.current.capacity().max(self.next.capacity());
        if retained_capacity == 0 {
            self.high_water_len = 0;
            self.low_water_frames = 0;
            return;
        }

        let low_water_threshold = (retained_capacity / EVENT_CAPACITY_LOW_WATER_DIVISOR).max(1);
        if active_len > low_water_threshold {
            self.low_water_frames = 0;
            return;
        }

        self.low_water_frames = self.low_water_frames.saturating_add(1);
        if self.low_water_frames < EVENT_CAPACITY_SHRINK_DEBOUNCE_FRAMES {
            return;
        }

        if self.shrink_buffers_to(active_len) {
            self.capacity_shrink_count = self.capacity_shrink_count.saturating_add(1);
        }
        self.high_water_len = active_len;
        self.low_water_frames = 0;
    }

    fn reserve_next_for_high_water(&mut self) {
        if self.high_water_len == 0 || self.next.capacity() >= self.high_water_len {
            return;
        }
        self.next
            .reserve_exact(self.high_water_len - self.next.capacity());
    }

    fn shrink_buffers_to(&mut self, target_capacity: usize) -> bool {
        let before = self.current.capacity().max(self.next.capacity());
        let current_target_capacity = target_capacity.max(self.current.len());
        let next_target_capacity = target_capacity.max(self.next.len());
        Self::shrink_vec_to(&mut self.current, current_target_capacity);
        Self::shrink_vec_to(&mut self.next, next_target_capacity);
        let after = self.current.capacity().max(self.next.capacity());
        after < before
    }

    fn shrink_vec_to(vec: &mut Vec<T>, target_capacity: usize) {
        if vec.capacity() <= target_capacity {
            return;
        }
        let mut replacement = Vec::with_capacity(target_capacity.max(vec.len()));
        replacement.extend(vec.drain(..));
        *vec = replacement;
    }
}

pub struct EventCursor<T> {
    cursor: usize,
    generation: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for EventCursor<T> {
    fn default() -> Self {
        Self {
            cursor: 0,
            generation: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> EventCursor<T> {
    pub fn read<'events>(
        &mut self,
        events: Option<&'events Events<T>>,
    ) -> EventReadIter<'events, T> {
        let Some(events) = events else {
            self.cursor = 0;
            self.generation = 0;
            return EventReadIter::empty();
        };
        let start = if self.generation == events.generation() {
            self.cursor.min(events.len())
        } else {
            0
        };
        self.cursor = events.len();
        self.generation = events.generation();
        EventReadIter::new(events.iter_from(start))
    }

    pub fn unread_count(&self, events: Option<&Events<T>>) -> usize {
        let Some(events) = events else {
            return 0;
        };
        if self.generation == events.generation() {
            events.len().saturating_sub(self.cursor.min(events.len()))
        } else {
            events.len()
        }
    }

    pub fn clear(&mut self, events: Option<&Events<T>>) {
        if let Some(events) = events {
            self.cursor = events.len();
            self.generation = events.generation();
        } else {
            self.cursor = 0;
            self.generation = 0;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventSubscriptionStatus {
    Dormant,
    Connected,
}

/// A manually controlled reader cursor for weak plugin/event dependencies.
///
/// Dormant subscriptions register the event type without activating its channel.
/// Connecting starts the cursor at the current queue position so late plugin
/// activation never replays historical events.
pub struct EventSubscription<T> {
    cursor: EventCursor<T>,
    event_type_id: EventTypeId,
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
        if self.is_connected() || !store.connect_reader(self.event_type_id) {
            return false;
        }
        self.cursor
            .clear(store.events_by_id::<T>(self.event_type_id));
        self.status = EventSubscriptionStatus::Connected;
        true
    }

    pub fn disconnect(&mut self, store: &mut EventStore) -> bool {
        if !self.is_connected() || !store.disconnect_reader(self.event_type_id) {
            return false;
        }
        self.cursor.clear(None);
        self.status = EventSubscriptionStatus::Dormant;
        true
    }

    pub fn read<'events>(&mut self, store: &'events EventStore) -> EventReadIter<'events, T> {
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

pub struct EventReadIter<'events, T> {
    inner: Option<std::slice::Iter<'events, T>>,
}

impl<'events, T> EventReadIter<'events, T> {
    fn new(inner: std::slice::Iter<'events, T>) -> Self {
        Self { inner: Some(inner) }
    }

    fn empty() -> Self {
        Self { inner: None }
    }
}

impl<'events, T> Iterator for EventReadIter<'events, T> {
    type Item = &'events T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next()
    }
}

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
        if !self.is_active(event_type_id) {
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
        if !self.is_active(event_type_id) {
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

#[cfg(test)]
mod tests {
    use super::Events;

    #[test]
    fn event_queue_equality_ignores_reader_generation_metadata() {
        let mut first = Events::<u32>::default();
        let mut second = Events::<u32>::default();

        first.update();
        first.update();

        assert_eq!(first, second);

        first.send(5);
        second.send(5);

        assert_eq!(first, second);
    }
}
