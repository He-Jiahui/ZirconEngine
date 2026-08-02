use std::collections::VecDeque;

use crate::scene::ecs::messages::cursor::MessageReadIter;
use crate::scene::ecs::messages::id::{Message, MessageId};

pub(super) struct MessageInstance<T>
where
    T: Message,
{
    pub(super) id: MessageId<T>,
    pub(super) message: T,
    byte_size: usize,
    written_frame: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageRetention {
    pub max_entries: usize,
    pub max_bytes: usize,
    pub max_age_frames: u64,
}

impl MessageRetention {
    pub const fn new(max_entries: usize, max_bytes: usize, max_age_frames: u64) -> Self {
        Self {
            max_entries,
            max_bytes,
            max_age_frames,
        }
    }
}

impl Default for MessageRetention {
    fn default() -> Self {
        Self::new(1_024, 256 * 1_024, 600)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MessageRetentionMetrics {
    pub retained_entries: usize,
    pub retained_bytes: usize,
    pub budget_dropped_entries: u64,
    pub budget_dropped_bytes: u64,
    pub age_dropped_entries: u64,
    pub age_dropped_bytes: u64,
}

pub struct Messages<T>
where
    T: Message,
{
    pub(super) messages: VecDeque<MessageInstance<T>>,
    next_id: usize,
    // Cursor reset marker for explicit retention boundaries such as clear_messages.
    generation: u64,
    retention: MessageRetention,
    retained_bytes: usize,
    metrics: MessageRetentionMetrics,
}

impl<T> Default for Messages<T>
where
    T: Message,
{
    fn default() -> Self {
        Self {
            messages: VecDeque::new(),
            next_id: 0,
            generation: 0,
            retention: MessageRetention::default(),
            retained_bytes: 0,
            metrics: MessageRetentionMetrics::default(),
        }
    }
}

impl<T> Messages<T>
where
    T: Message,
{
    pub fn write(&mut self, message: T) -> MessageId<T> {
        self.write_at_frame(message, 0)
    }

    pub(super) fn write_at_frame(&mut self, message: T, frame: u64) -> MessageId<T> {
        let id = MessageId::new(self.next_id);
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("message id space exhausted");
        let byte_size = message.retained_byte_size();
        self.retained_bytes = self.retained_bytes.saturating_add(byte_size);
        self.messages.push_back(MessageInstance {
            id,
            message,
            byte_size,
            written_frame: frame,
        });
        self.enforce_budget();
        self.refresh_retention_metrics();
        id
    }

    pub fn write_batch<I>(&mut self, messages: I) -> Vec<MessageId<T>>
    where
        I: IntoIterator<Item = T>,
    {
        self.write_batch_at_frame(messages, 0)
    }

    pub(super) fn write_batch_at_frame<I>(&mut self, messages: I, frame: u64) -> Vec<MessageId<T>>
    where
        I: IntoIterator<Item = T>,
    {
        let messages = messages.into_iter();
        let (lower_bound, _) = messages.size_hint();
        self.messages.reserve(lower_bound);

        let mut ids = Vec::with_capacity(lower_bound);
        for message in messages {
            ids.push(self.write_at_frame(message, frame));
        }
        ids
    }

    pub fn iter(&self) -> MessageReadIter<'_, T> {
        MessageReadIter::new(self.messages.iter(), 0)
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.retained_bytes = 0;
        self.generation = self.generation.saturating_add(1);
        self.refresh_retention_metrics();
    }

    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub fn retention(&self) -> MessageRetention {
        self.retention
    }

    pub fn retention_metrics(&self) -> MessageRetentionMetrics {
        self.metrics
    }

    pub fn set_retention(&mut self, retention: MessageRetention) {
        self.retention = retention;
        self.enforce_budget();
        self.refresh_retention_metrics();
    }

    pub(super) fn advance_frame(&mut self, frame: u64) {
        self.retire_expired(frame);
        self.refresh_retention_metrics();
    }

    pub(super) fn next_id(&self) -> usize {
        self.next_id
    }

    pub(super) fn read_window_start(&self, cursor: usize) -> (usize, usize) {
        let Some(first) = self.messages.front() else {
            return (0, self.next_id.saturating_sub(cursor));
        };
        let first_id = first.id.id();
        if cursor < first_id {
            return (0, first_id - cursor);
        }
        (cursor.saturating_sub(first_id).min(self.messages.len()), 0)
    }

    fn enforce_budget(&mut self) {
        while self.messages.len() > self.retention.max_entries
            || self.retained_bytes > self.retention.max_bytes
        {
            self.drop_oldest_budget();
        }
    }

    fn retire_expired(&mut self, frame: u64) {
        while self.messages.front().is_some_and(|message| {
            frame.saturating_sub(message.written_frame) > self.retention.max_age_frames
        }) {
            let message = self
                .messages
                .pop_front()
                .expect("front message must exist while retiring");
            self.retained_bytes = self.retained_bytes.saturating_sub(message.byte_size);
            self.metrics.age_dropped_entries = self.metrics.age_dropped_entries.saturating_add(1);
            self.metrics.age_dropped_bytes = self
                .metrics
                .age_dropped_bytes
                .saturating_add(message.byte_size as u64);
        }
    }

    fn drop_oldest_budget(&mut self) {
        let message = self
            .messages
            .pop_front()
            .expect("budget overflow requires a queued message");
        self.retained_bytes = self.retained_bytes.saturating_sub(message.byte_size);
        self.metrics.budget_dropped_entries = self.metrics.budget_dropped_entries.saturating_add(1);
        self.metrics.budget_dropped_bytes = self
            .metrics
            .budget_dropped_bytes
            .saturating_add(message.byte_size as u64);
    }

    fn refresh_retention_metrics(&mut self) {
        self.metrics.retained_entries = self.messages.len();
        self.metrics.retained_bytes = self.retained_bytes;
    }
}
