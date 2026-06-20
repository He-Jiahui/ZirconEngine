use std::marker::PhantomData;

use crate::scene::ecs::messages::id::{Message, MessageId};
use crate::scene::ecs::messages::queue::{MessageInstance, Messages};

pub struct MessageCursor<T>
where
    T: Message,
{
    cursor: usize,
    generation: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for MessageCursor<T>
where
    T: Message,
{
    fn default() -> Self {
        Self {
            cursor: 0,
            generation: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> MessageCursor<T>
where
    T: Message,
{
    pub fn read<'a>(&mut self, messages: Option<&'a Messages<T>>) -> MessageReadIter<'a, T> {
        let Some(messages) = messages else {
            self.cursor = 0;
            self.generation = 0;
            return MessageReadIter::empty();
        };
        let start = if self.generation == messages.generation() {
            self.cursor.min(messages.messages.len())
        } else {
            0
        };
        self.cursor = messages.messages.len();
        self.generation = messages.generation();
        MessageReadIter::new(messages.messages[start..].iter())
    }

    pub fn unread_count(&self, messages: Option<&Messages<T>>) -> usize {
        let Some(messages) = messages else {
            return 0;
        };
        if self.generation == messages.generation() {
            messages
                .messages
                .len()
                .saturating_sub(self.cursor.min(messages.messages.len()))
        } else {
            messages.messages.len()
        }
    }

    pub fn clear(&mut self, messages: Option<&Messages<T>>) {
        if let Some(messages) = messages {
            self.cursor = messages.len();
            self.generation = messages.generation();
        } else {
            self.cursor = 0;
            self.generation = 0;
        }
    }
}

pub struct MessageReadIter<'a, T>
where
    T: Message,
{
    inner: Option<std::slice::Iter<'a, MessageInstance<T>>>,
}

impl<'a, T> MessageReadIter<'a, T>
where
    T: Message,
{
    pub(crate) fn new(inner: std::slice::Iter<'a, MessageInstance<T>>) -> Self {
        Self { inner: Some(inner) }
    }

    pub(crate) fn empty() -> Self {
        Self { inner: None }
    }
}

impl<'a, T> Iterator for MessageReadIter<'a, T>
where
    T: Message,
{
    type Item = (MessageId<T>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.as_mut()?.next()?;
        Some((next.id, &next.message))
    }
}
