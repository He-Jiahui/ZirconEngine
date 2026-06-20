use std::marker::PhantomData;

use crate::scene::ecs::events::Events;

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

pub struct EventReadIter<'events, T> {
    inner: Option<std::slice::Iter<'events, T>>,
}

impl<'events, T> EventReadIter<'events, T> {
    pub(crate) fn new(inner: std::slice::Iter<'events, T>) -> Self {
        Self { inner: Some(inner) }
    }

    pub(crate) fn empty() -> Self {
        Self { inner: None }
    }
}

impl<'events, T> Iterator for EventReadIter<'events, T> {
    type Item = &'events T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next()
    }
}
