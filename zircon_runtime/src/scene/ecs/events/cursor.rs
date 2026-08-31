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
    /// Reads unread events and commits the cursor only when the iterator yields an item.
    ///
    /// Dropping a partially consumed iterator leaves its tail unread, which lets bounded
    /// consumers page without silently acknowledging events they did not process.
    pub fn read<'events>(
        &'events mut self,
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
        self.cursor = start;
        self.generation = events.generation();
        EventReadIter::new(events.iter_from(start), self)
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
    state: EventReadState<'events, T>,
}

enum EventReadState<'events, T> {
    Empty,
    Events {
        inner: std::slice::Iter<'events, T>,
        cursor: &'events mut EventCursor<T>,
    },
}

impl<'events, T> EventReadIter<'events, T> {
    pub(crate) fn new(
        inner: std::slice::Iter<'events, T>,
        cursor: &'events mut EventCursor<T>,
    ) -> Self {
        Self {
            state: EventReadState::Events { inner, cursor },
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            state: EventReadState::Empty,
        }
    }
}

impl<'events, T> Iterator for EventReadIter<'events, T> {
    type Item = &'events T;

    fn next(&mut self) -> Option<Self::Item> {
        let EventReadState::Events { inner, cursor } = &mut self.state else {
            return None;
        };
        let event = inner.next()?;
        cursor.cursor += 1;
        Some(event)
    }
}

#[cfg(test)]
mod tests {
    use super::{EventCursor, EventReadIter};
    use crate::scene::ecs::events::Events;

    #[test]
    fn runtime60_batch_empty_event_read_iterator_stays_exhausted() {
        let mut read = EventReadIter::<u32>::empty();

        assert_eq!(read.next(), None);
        assert_eq!(read.next(), None);
    }

    #[test]
    fn runtime60_batch_partial_event_read_commits_each_yield_exactly_once() {
        let mut events = Events::default();
        events.send_batch([1_u32, 2, 3]);
        events.update();
        let mut cursor = EventCursor::default();

        let mut read = cursor.read(Some(&events));
        assert_eq!(read.next(), Some(&1));
        drop(read);

        assert_eq!(cursor.unread_count(Some(&events)), 2);
    }

    #[test]
    fn runtime60_batch_event_read_iterator_stays_exhausted_after_tail() {
        let mut events = Events::default();
        events.send(7_u32);
        events.update();
        let mut cursor = EventCursor::default();

        let mut read = cursor.read(Some(&events));
        assert_eq!(read.next(), Some(&7));
        assert_eq!(read.next(), None);
        assert_eq!(read.next(), None);
        drop(read);

        assert_eq!(cursor.unread_count(Some(&events)), 0);
    }

    #[test]
    fn bounded_read_only_commits_events_consumed_by_the_iterator() {
        let mut events = Events::default();
        events.send_batch([1_u32, 2, 3, 4]);
        events.update();
        let mut cursor = EventCursor::default();

        let first_page = cursor
            .read(Some(&events))
            .take(2)
            .copied()
            .collect::<Vec<_>>();

        assert_eq!(first_page, [1, 2]);
        assert_eq!(cursor.unread_count(Some(&events)), 2);
        assert_eq!(
            cursor.read(Some(&events)).copied().collect::<Vec<_>>(),
            [3, 4]
        );
        assert_eq!(cursor.unread_count(Some(&events)), 0);
    }
}
