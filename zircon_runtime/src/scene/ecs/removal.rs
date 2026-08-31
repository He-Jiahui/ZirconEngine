use std::any::{TypeId, type_name};
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::marker::PhantomData;

use crate::scene::EntityId;

const DEFAULT_REMOVED_COMPONENT_MAX_ENTRIES: usize = 1_024;
const DEFAULT_REMOVED_COMPONENT_MAX_BYTES: usize = 256 * 1_024;
const DEFAULT_REMOVED_COMPONENT_MAX_AGE_FRAMES: u64 = 600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemovedComponentEvent {
    entity: EntityId,
}

impl RemovedComponentEvent {
    pub const fn new(entity: EntityId) -> Self {
        Self { entity }
    }

    pub const fn entity(self) -> EntityId {
        self.entity
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RemovedComponentRetention {
    pub max_entries: usize,
    pub max_bytes: usize,
    pub max_age_frames: u64,
}

impl RemovedComponentRetention {
    pub const fn new(max_entries: usize, max_bytes: usize, max_age_frames: u64) -> Self {
        Self {
            max_entries,
            max_bytes,
            max_age_frames,
        }
    }
}

impl Default for RemovedComponentRetention {
    fn default() -> Self {
        Self::new(
            DEFAULT_REMOVED_COMPONENT_MAX_ENTRIES,
            DEFAULT_REMOVED_COMPONENT_MAX_BYTES,
            DEFAULT_REMOVED_COMPONENT_MAX_AGE_FRAMES,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RemovedComponentRetentionMetrics {
    pub retained_entries: usize,
    pub retained_bytes: usize,
    pub retained_capacity: usize,
    pub budget_dropped_entries: u64,
    pub budget_dropped_bytes: u64,
    pub age_dropped_entries: u64,
    pub age_dropped_bytes: u64,
    pub rejected_writes: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RemovedComponentWriteReceipt {
    sequence: Option<u64>,
    retained: bool,
    dropped_entries: u64,
    dropped_bytes: u64,
}

impl RemovedComponentWriteReceipt {
    pub const fn sequence(self) -> Option<u64> {
        self.sequence
    }

    pub const fn is_retained(self) -> bool {
        self.retained
    }

    pub const fn dropped_entries(self) -> u64 {
        self.dropped_entries
    }

    pub const fn dropped_bytes(self) -> u64 {
        self.dropped_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RemovedComponentEntry {
    sequence: u64,
    event: RemovedComponentEvent,
    written_frame: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RemovedComponentChannel {
    entries: VecDeque<RemovedComponentEntry>,
    next_sequence: u64,
    generation: u64,
    retention: RemovedComponentRetention,
    retained_bytes: usize,
    metrics: RemovedComponentRetentionMetrics,
}

impl Default for RemovedComponentChannel {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            next_sequence: 0,
            generation: 0,
            retention: RemovedComponentRetention::default(),
            retained_bytes: 0,
            metrics: RemovedComponentRetentionMetrics::default(),
        }
    }
}

impl RemovedComponentChannel {
    fn push(&mut self, entity: EntityId, frame: u64) -> RemovedComponentWriteReceipt {
        let Some(next_sequence) = self.next_sequence.checked_add(1) else {
            self.metrics.rejected_writes = self.metrics.rejected_writes.saturating_add(1);
            return RemovedComponentWriteReceipt::default();
        };

        let sequence = self.next_sequence;
        self.next_sequence = next_sequence;
        self.retained_bytes = self
            .retained_bytes
            .saturating_add(REMOVED_COMPONENT_ENTRY_BYTES);
        self.entries.push_back(RemovedComponentEntry {
            sequence,
            event: RemovedComponentEvent::new(entity),
            written_frame: frame,
        });
        let (dropped_entries, dropped_bytes) = self.enforce_budget();
        self.refresh_metrics();
        RemovedComponentWriteReceipt {
            sequence: Some(sequence),
            retained: self
                .entries
                .back()
                .is_some_and(|entry| entry.sequence == sequence),
            dropped_entries,
            dropped_bytes,
        }
    }

    fn configure_retention(&mut self, retention: RemovedComponentRetention) {
        self.retention = retention;
        self.enforce_budget();
        self.shrink_capacity_to_budget();
        self.refresh_metrics();
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.entries.shrink_to(0);
        self.retained_bytes = 0;
        self.generation = self.generation.saturating_add(1);
        self.refresh_metrics();
    }

    fn advance_frame(&mut self, frame: u64) {
        while self.entries.front().is_some_and(|entry| {
            frame.saturating_sub(entry.written_frame) > self.retention.max_age_frames
        }) {
            let Some(_entry) = self.entries.pop_front() else {
                break;
            };
            self.retained_bytes = self
                .retained_bytes
                .saturating_sub(REMOVED_COMPONENT_ENTRY_BYTES);
            self.metrics.age_dropped_entries = self.metrics.age_dropped_entries.saturating_add(1);
            self.metrics.age_dropped_bytes = self
                .metrics
                .age_dropped_bytes
                .saturating_add(REMOVED_COMPONENT_ENTRY_BYTES as u64);
        }
        self.shrink_capacity_to_budget();
        self.refresh_metrics();
    }

    fn read_window_start(&self, next_sequence: u64, generation: u64) -> (usize, u64) {
        if generation != self.generation {
            return (0, 0);
        }
        let Some(first) = self.entries.front() else {
            return (0, self.next_sequence.saturating_sub(next_sequence));
        };
        if next_sequence < first.sequence {
            return (0, first.sequence - next_sequence);
        }
        (
            next_sequence
                .saturating_sub(first.sequence)
                .min(self.entries.len() as u64) as usize,
            0,
        )
    }

    fn unread_count(&self, next_sequence: u64, generation: u64) -> usize {
        let (start, _) = self.read_window_start(next_sequence, generation);
        self.entries.len().saturating_sub(start)
    }

    fn enforce_budget(&mut self) -> (u64, u64) {
        let mut dropped_entries = 0_u64;
        let mut dropped_bytes = 0_u64;
        while self.entries.len() > self.retention.max_entries
            || self.retained_bytes > self.retention.max_bytes
        {
            let Some(entry) = self.entries.pop_front() else {
                break;
            };
            self.retained_bytes = self
                .retained_bytes
                .saturating_sub(REMOVED_COMPONENT_ENTRY_BYTES);
            dropped_entries = dropped_entries.saturating_add(1);
            dropped_bytes = dropped_bytes.saturating_add(REMOVED_COMPONENT_ENTRY_BYTES as u64);
            debug_assert!(entry.sequence < self.next_sequence);
        }
        self.metrics.budget_dropped_entries = self
            .metrics
            .budget_dropped_entries
            .saturating_add(dropped_entries);
        self.metrics.budget_dropped_bytes = self
            .metrics
            .budget_dropped_bytes
            .saturating_add(dropped_bytes);
        (dropped_entries, dropped_bytes)
    }

    fn shrink_capacity_to_budget(&mut self) {
        let byte_capacity = self.retention.max_bytes / REMOVED_COMPONENT_ENTRY_BYTES;
        let target_capacity = self
            .retention
            .max_entries
            .min(byte_capacity)
            .max(self.entries.len());
        self.entries.shrink_to(target_capacity);
    }

    fn refresh_metrics(&mut self) {
        self.metrics.retained_entries = self.entries.len();
        self.metrics.retained_bytes = self.retained_bytes;
        self.metrics.retained_capacity = self.entries.capacity();
    }
}

const REMOVED_COMPONENT_ENTRY_BYTES: usize = std::mem::size_of::<RemovedComponentEntry>();

/// World-owned, bounded removal history partitioned by component type.
///
/// The runtime advances this store from `InternalSceneSystem::UpdateEvents`; standalone callers
/// can use `World::clear_trackers`. Retention may advance past a slow reader, which is reported by
/// that reader's `dropped_count` instead of retaining historical removals indefinitely.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RemovedComponentEvents {
    channels: HashMap<TypeId, RemovedComponentChannel>,
    type_names: HashMap<TypeId, String>,
    active_channels: BTreeSet<TypeId>,
    frame: u64,
    last_advance_channel_visits: usize,
}

impl RemovedComponentEvents {
    pub fn push<T>(&mut self, entity: EntityId) -> RemovedComponentWriteReceipt
    where
        T: 'static,
    {
        self.push_type_id(TypeId::of::<T>(), type_name::<T>(), entity)
    }

    pub(crate) fn push_type_id(
        &mut self,
        type_id: TypeId,
        type_name: impl Into<String>,
        entity: EntityId,
    ) -> RemovedComponentWriteReceipt {
        self.type_names
            .entry(type_id)
            .or_insert_with(|| type_name.into());
        let receipt = self
            .channels
            .entry(type_id)
            .or_default()
            .push(entity, self.frame);
        self.refresh_active_channel(type_id);
        receipt
    }

    pub fn events<T>(&self) -> RemovedComponentEventIter<'_>
    where
        T: 'static,
    {
        RemovedComponentEventIter::new(
            self.channels
                .get(&TypeId::of::<T>())
                .map(|channel| channel.entries.iter()),
        )
    }

    pub fn configure_retention<T>(&mut self, retention: RemovedComponentRetention)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        self.type_names
            .entry(type_id)
            .or_insert_with(|| type_name::<T>().to_string());
        self.channels
            .entry(type_id)
            .or_default()
            .configure_retention(retention);
        self.refresh_active_channel(type_id);
    }

    pub fn retention<T>(&self) -> Option<RemovedComponentRetention>
    where
        T: 'static,
    {
        self.channels
            .get(&TypeId::of::<T>())
            .map(|channel| channel.retention)
    }

    pub fn retention_metrics<T>(&self) -> Option<RemovedComponentRetentionMetrics>
    where
        T: 'static,
    {
        self.channels
            .get(&TypeId::of::<T>())
            .map(|channel| channel.metrics)
    }

    pub fn clear<T>(&mut self)
    where
        T: 'static,
    {
        let type_id = TypeId::of::<T>();
        if let Some(channel) = self.channels.get_mut(&type_id) {
            channel.clear();
        }
        self.active_channels.remove(&type_id);
    }

    pub fn registered_type_names(&self) -> Vec<&str> {
        let mut names = Vec::with_capacity(self.type_names.len());
        for name in self.type_names.values() {
            names.push(name.as_str());
        }
        names.sort_unstable();
        names
    }

    pub fn last_advance_channel_visits(&self) -> usize {
        self.last_advance_channel_visits
    }

    pub(crate) fn advance_frame(&mut self) {
        self.frame = self.frame.saturating_add(1);
        let active_channels = std::mem::take(&mut self.active_channels);
        self.last_advance_channel_visits = active_channels.len();
        for type_id in active_channels {
            let keep_active = self.channels.get_mut(&type_id).is_some_and(|channel| {
                channel.advance_frame(self.frame);
                !channel.entries.is_empty()
            });
            if keep_active {
                self.active_channels.insert(type_id);
            }
        }
    }

    fn channel<T>(&self) -> Option<&RemovedComponentChannel>
    where
        T: 'static,
    {
        self.channels.get(&TypeId::of::<T>())
    }

    fn refresh_active_channel(&mut self, type_id: TypeId) {
        if self
            .channels
            .get(&type_id)
            .is_some_and(|channel| !channel.entries.is_empty())
        {
            self.active_channels.insert(type_id);
        } else {
            self.active_channels.remove(&type_id);
        }
    }
}

pub struct RemovedComponentEventIter<'events> {
    inner: Option<std::collections::vec_deque::Iter<'events, RemovedComponentEntry>>,
}

impl<'events> RemovedComponentEventIter<'events> {
    fn new(
        inner: Option<std::collections::vec_deque::Iter<'events, RemovedComponentEntry>>,
    ) -> Self {
        Self { inner }
    }
}

impl Iterator for RemovedComponentEventIter<'_> {
    type Item = RemovedComponentEvent;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.inner.as_mut()?.next()?.event)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemovedComponentReader<T> {
    next_sequence: u64,
    generation: u64,
    dropped_count: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Default for RemovedComponentReader<T> {
    fn default() -> Self {
        Self {
            next_sequence: 0,
            generation: 0,
            dropped_count: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> RemovedComponentReader<T> {
    pub fn read<'reader, 'events>(
        &'reader mut self,
        events: &'events RemovedComponentEvents,
    ) -> RemovedComponentReadIter<'reader, 'events, T>
    where
        T: 'static,
    {
        let Some(channel) = events.channel::<T>() else {
            self.next_sequence = 0;
            self.generation = 0;
            return RemovedComponentReadIter::empty(self);
        };
        if self.generation != channel.generation {
            self.next_sequence = channel
                .entries
                .front()
                .map(|entry| entry.sequence)
                .unwrap_or(channel.next_sequence);
            self.generation = channel.generation;
            return RemovedComponentReadIter::new(self, channel.entries.iter(), 0);
        }
        let (start, dropped) = channel.read_window_start(self.next_sequence, self.generation);
        self.dropped_count = self.dropped_count.saturating_add(dropped);
        self.generation = channel.generation;
        RemovedComponentReadIter::new(self, channel.entries.iter(), start)
    }

    pub fn len(&self, events: &RemovedComponentEvents) -> usize
    where
        T: 'static,
    {
        events
            .channel::<T>()
            .map(|channel| channel.unread_count(self.next_sequence, self.generation))
            .unwrap_or(0)
    }

    pub fn is_empty(&self, events: &RemovedComponentEvents) -> bool
    where
        T: 'static,
    {
        self.len(events) == 0
    }

    pub fn clear(&mut self, events: &RemovedComponentEvents)
    where
        T: 'static,
    {
        let Some(channel) = events.channel::<T>() else {
            self.next_sequence = 0;
            self.generation = 0;
            return;
        };
        self.next_sequence = channel.next_sequence;
        self.generation = channel.generation;
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped_count
    }
}

pub struct RemovedComponentReadIter<'reader, 'events, T> {
    reader: &'reader mut RemovedComponentReader<T>,
    inner: Option<std::collections::vec_deque::Iter<'events, RemovedComponentEntry>>,
}

impl<'reader, 'events, T> RemovedComponentReadIter<'reader, 'events, T> {
    fn new(
        reader: &'reader mut RemovedComponentReader<T>,
        mut inner: std::collections::vec_deque::Iter<'events, RemovedComponentEntry>,
        start: usize,
    ) -> Self {
        for _ in 0..start {
            let _ = inner.next();
        }
        Self {
            reader,
            inner: Some(inner),
        }
    }

    fn empty(reader: &'reader mut RemovedComponentReader<T>) -> Self {
        Self {
            reader,
            inner: None,
        }
    }
}

impl<T> Iterator for RemovedComponentReadIter<'_, '_, T> {
    type Item = EntityId;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = self.inner.as_mut()?.next()?;
        self.reader.next_sequence = entry.sequence.saturating_add(1);
        Some(entry.event.entity())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner
            .as_ref()
            .map(|inner| inner.size_hint())
            .unwrap_or((0, Some(0)))
    }
}

impl<T> ExactSizeIterator for RemovedComponentReadIter<'_, '_, T> {}
