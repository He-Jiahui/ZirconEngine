use std::marker::PhantomData;

use crate::scene::ecs::events::Events;

/// 单个 Events<T> 队列的读取进度；系统参数把它保存在 SystemState 中跨帧复用。
/// 不应把同一 cursor 转用于另一队列；相同 generation 不能识别不同队列。
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
#[path = "tests/cursor.rs"]
mod tests;
