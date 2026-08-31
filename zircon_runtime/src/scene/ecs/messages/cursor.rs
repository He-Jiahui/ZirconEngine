use std::marker::PhantomData;

use crate::scene::ecs::messages::id::{Message, MessageId};
use crate::scene::ecs::messages::queue::{MessageInstance, Messages};

pub struct MessageCursor<T>
where
    T: Message,
{
    next_id: usize,
    generation: u64,
    dropped_count: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for MessageCursor<T>
where
    T: Message,
{
    fn default() -> Self {
        Self {
            next_id: 0,
            generation: 0,
            dropped_count: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> MessageCursor<T>
where
    T: Message,
{
    pub fn read<'a>(&'a mut self, messages: Option<&'a Messages<T>>) -> MessageReadIter<'a, T> {
        let Some(messages) = messages else {
            self.next_id = 0;
            self.generation = 0;
            return MessageReadIter::empty();
        };
        let (start, dropped) = if self.generation == messages.generation() {
            messages.read_window_start(self.next_id)
        } else {
            (0, 0)
        };
        self.dropped_count = self.dropped_count.saturating_add(dropped as u64);
        // Commit only retention loss here. Delivery acknowledgement remains
        // iterator-driven so a partially consumed read leaves its tail unread.
        self.next_id = messages
            .messages
            .get(start)
            .map(|message| message.id.id())
            .unwrap_or_else(|| messages.next_id());
        self.generation = messages.generation();
        MessageReadIter::tracked(messages.messages.iter(), start, self)
    }

    pub fn unread_count(&self, messages: Option<&Messages<T>>) -> usize {
        let Some(messages) = messages else {
            return 0;
        };
        if self.generation == messages.generation() {
            let (start, _) = messages.read_window_start(self.next_id);
            messages.messages.len().saturating_sub(start)
        } else {
            messages.messages.len()
        }
    }

    pub fn clear(&mut self, messages: Option<&Messages<T>>) {
        if let Some(messages) = messages {
            self.next_id = messages.next_id();
            self.generation = messages.generation();
        } else {
            self.next_id = 0;
            self.generation = 0;
        }
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped_count
    }
}

pub struct MessageReadIter<'a, T>
where
    T: Message,
{
    inner: Option<std::collections::vec_deque::Iter<'a, MessageInstance<T>>>,
    cursor: Option<&'a mut MessageCursor<T>>,
}

impl<'a, T> MessageReadIter<'a, T>
where
    T: Message,
{
    pub(crate) fn tracked(
        mut inner: std::collections::vec_deque::Iter<'a, MessageInstance<T>>,
        skip: usize,
        cursor: &'a mut MessageCursor<T>,
    ) -> Self {
        for _ in 0..skip {
            let _ = inner.next();
        }
        Self {
            inner: Some(inner),
            cursor: Some(cursor),
        }
    }

    pub(crate) fn untracked(
        inner: std::collections::vec_deque::Iter<'a, MessageInstance<T>>,
    ) -> Self {
        Self {
            inner: Some(inner),
            cursor: None,
        }
    }

    pub(crate) fn empty() -> Self {
        Self {
            inner: None,
            cursor: None,
        }
    }
}

impl<'a, T> Iterator for MessageReadIter<'a, T>
where
    T: Message,
{
    type Item = (MessageId<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.as_mut()?.next()?;
        if let Some(cursor) = self.cursor.as_deref_mut() {
            cursor.next_id = next.id.id().saturating_add(1);
        }
        Some((next.id, &next.message))
    }
}
