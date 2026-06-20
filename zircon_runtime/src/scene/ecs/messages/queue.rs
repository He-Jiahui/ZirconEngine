use crate::scene::ecs::messages::cursor::MessageReadIter;
use crate::scene::ecs::messages::id::{Message, MessageId};

pub(super) struct MessageInstance<T>
where
    T: Message,
{
    pub(super) id: MessageId<T>,
    pub(super) message: T,
}

pub struct Messages<T>
where
    T: Message,
{
    pub(super) messages: Vec<MessageInstance<T>>,
    next_id: usize,
    // Cursor reset marker for explicit retention boundaries such as clear_messages.
    generation: u64,
}

impl<T> Default for Messages<T>
where
    T: Message,
{
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            next_id: 0,
            generation: 0,
        }
    }
}

impl<T> Messages<T>
where
    T: Message,
{
    pub fn write(&mut self, message: T) -> MessageId<T> {
        let id = MessageId::new(self.next_id);
        self.next_id += 1;
        self.messages.push(MessageInstance { id, message });
        id
    }

    pub fn write_batch<I>(&mut self, messages: I) -> Vec<MessageId<T>>
    where
        I: IntoIterator<Item = T>,
    {
        let messages = messages.into_iter();
        let (lower_bound, _) = messages.size_hint();
        self.messages.reserve(lower_bound);

        let mut ids = Vec::with_capacity(lower_bound);
        for message in messages {
            ids.push(self.write(message));
        }
        ids
    }

    pub fn iter(&self) -> MessageReadIter<'_, T> {
        MessageReadIter::new(self.messages.iter())
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.generation = self.generation.saturating_add(1);
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }
}
