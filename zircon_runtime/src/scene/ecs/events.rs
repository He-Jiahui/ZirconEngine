use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct Events<T> {
    current: Vec<T>,
    next: Vec<T>,
    generation: u64,
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
        }
    }
}

impl<T> Events<T> {
    pub fn send(&mut self, event: T) {
        self.next.push(event);
    }

    pub fn send_batch<I>(&mut self, events: I) -> usize
    where
        I: IntoIterator<Item = T>,
    {
        let mut written = 0;
        for event in events {
            self.send(event);
            written += 1;
        }
        written
    }

    pub fn update(&mut self) {
        self.current.clear();
        std::mem::swap(&mut self.current, &mut self.next);
        self.generation = self.generation.saturating_add(1);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.current.iter()
    }

    pub fn iter_from(&self, start: usize) -> std::slice::Iter<'_, T> {
        self.current[start.min(self.current.len())..].iter()
    }

    pub fn drain(&mut self) -> Vec<T> {
        self.current.drain(..).collect()
    }

    pub fn clear(&mut self) {
        self.current.clear();
        self.next.clear();
        self.generation = self.generation.saturating_add(1);
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
        events
            .map(|events| {
                if self.generation == events.generation() {
                    events.len().saturating_sub(self.cursor.min(events.len()))
                } else {
                    events.len()
                }
            })
            .unwrap_or_default()
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

#[derive(Default)]
pub struct EventStore {
    stores: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
    type_names: HashMap<TypeId, &'static str>,
}

impl EventStore {
    pub fn events<T: 'static + Send + Sync>(&self) -> Option<&Events<T>> {
        self.stores
            .get(&TypeId::of::<T>())
            .and_then(|store| store.downcast_ref::<Events<T>>())
    }

    pub fn events_mut<T: 'static + Send + Sync>(&mut self) -> &mut Events<T> {
        let type_id = TypeId::of::<T>();
        self.type_names.entry(type_id).or_insert(type_name::<T>());
        self.stores
            .entry(type_id)
            .or_insert_with(|| Box::<Events<T>>::default())
            .downcast_mut::<Events<T>>()
            .expect("event store type id must match event queue type")
    }

    pub fn send<T: 'static + Send + Sync>(&mut self, event: T) {
        self.events_mut::<T>().send(event);
    }

    pub fn send_batch<T, I>(&mut self, events: I) -> usize
    where
        T: 'static + Send + Sync,
        I: IntoIterator<Item = T>,
    {
        self.events_mut::<T>().send_batch(events)
    }

    pub fn update<T: 'static + Send + Sync>(&mut self) {
        self.events_mut::<T>().update();
    }

    pub fn drain<T: 'static + Send + Sync>(&mut self) -> Vec<T> {
        self.events_mut::<T>().drain()
    }

    pub fn registered_type_names(&self) -> Vec<&'static str> {
        let mut names = self.type_names.values().copied().collect::<Vec<_>>();
        names.sort_unstable();
        names
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
