use std::collections::{BTreeMap, VecDeque};

use super::EditorMessageDelivery;
use super::retention::{EditorMessageCoalescingKey, EditorMessageRetention};

const DEFAULT_LOSSLESS_CAPACITY: usize = 4_096;
const DEFAULT_BOUNDED_CAPACITY: usize = 256;
const DEFAULT_LATEST_CAPACITY: usize = 256;
const DEFAULT_MAX_DELIVERY_BYTES: usize = 2 * 1024 * 1024;
const DEFAULT_RETAINED_BYTES_CAPACITY: usize = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorMessageInboxLimits {
    lossless_capacity: usize,
    bounded_capacity: usize,
    latest_capacity: usize,
    max_delivery_bytes: usize,
    retained_bytes_capacity: usize,
}

impl EditorMessageInboxLimits {
    pub const fn new(
        lossless_capacity: usize,
        bounded_capacity: usize,
        latest_capacity: usize,
    ) -> Self {
        Self {
            lossless_capacity,
            bounded_capacity,
            latest_capacity,
            max_delivery_bytes: DEFAULT_MAX_DELIVERY_BYTES,
            retained_bytes_capacity: DEFAULT_RETAINED_BYTES_CAPACITY,
        }
    }

    pub const fn with_byte_limits(
        mut self,
        max_delivery_bytes: usize,
        retained_bytes_capacity: usize,
    ) -> Self {
        self.max_delivery_bytes = max_delivery_bytes;
        self.retained_bytes_capacity = retained_bytes_capacity;
        self
    }
}

impl Default for EditorMessageInboxLimits {
    fn default() -> Self {
        Self::new(
            DEFAULT_LOSSLESS_CAPACITY,
            DEFAULT_BOUNDED_CAPACITY,
            DEFAULT_LATEST_CAPACITY,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditorMessageInboxStats {
    depth: usize,
    lossless_depth: usize,
    bounded_depth: usize,
    latest_depth: usize,
    retained_bytes: usize,
    drained: u64,
    coalesced: u64,
    dropped: u64,
    backpressured: u64,
    age_in_messages: u64,
}

impl EditorMessageInboxStats {
    pub fn depth(self) -> usize {
        self.depth
    }

    pub fn lossless_depth(self) -> usize {
        self.lossless_depth
    }

    pub fn bounded_depth(self) -> usize {
        self.bounded_depth
    }

    pub fn latest_depth(self) -> usize {
        self.latest_depth
    }

    pub fn retained_bytes(self) -> usize {
        self.retained_bytes
    }

    pub fn drained(self) -> u64 {
        self.drained
    }

    pub fn coalesced(self) -> u64 {
        self.coalesced
    }

    pub fn dropped(self) -> u64 {
        self.dropped
    }

    pub fn backpressured(self) -> u64 {
        self.backpressured
    }

    pub fn age_in_messages(self) -> u64 {
        self.age_in_messages
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EditorMessageInboxEnqueue {
    Enqueued,
    Coalesced,
    CoalescedAfterDrop,
    EnqueuedAfterDrop,
    Dropped,
    Backpressured,
}

#[derive(Clone, Debug)]
pub(super) struct EditorMessageInbox {
    limits: EditorMessageInboxLimits,
    deliveries: BTreeMap<u64, EditorMessageDelivery>,
    latest_by_key: BTreeMap<EditorMessageCoalescingKey, u64>,
    latest_order: VecDeque<(EditorMessageCoalescingKey, u64)>,
    bounded_order: VecDeque<u64>,
    lossless_depth: usize,
    bounded_depth: usize,
    latest_depth: usize,
    retained_bytes: usize,
    drained: u64,
    coalesced: u64,
    dropped: u64,
    backpressured: u64,
}

impl EditorMessageInbox {
    pub(super) fn new(limits: EditorMessageInboxLimits) -> Self {
        Self {
            limits,
            deliveries: BTreeMap::new(),
            latest_by_key: BTreeMap::new(),
            latest_order: VecDeque::new(),
            bounded_order: VecDeque::new(),
            lossless_depth: 0,
            bounded_depth: 0,
            latest_depth: 0,
            retained_bytes: 0,
            drained: 0,
            coalesced: 0,
            dropped: 0,
            backpressured: 0,
        }
    }

    #[cfg(test)]
    pub(super) fn deliveries(&self) -> Vec<EditorMessageDelivery> {
        self.deliveries.values().cloned().collect()
    }

    pub(super) fn drain(&mut self) -> Vec<EditorMessageDelivery> {
        self.drained = self
            .drained
            .saturating_add(u64::try_from(self.deliveries.len()).unwrap_or(u64::MAX));
        self.latest_by_key.clear();
        self.latest_order.clear();
        self.bounded_order.clear();
        self.lossless_depth = 0;
        self.bounded_depth = 0;
        self.latest_depth = 0;
        self.retained_bytes = 0;
        std::mem::take(&mut self.deliveries).into_values().collect()
    }

    pub(super) fn enqueue(&mut self, delivery: EditorMessageDelivery) -> EditorMessageInboxEnqueue {
        match delivery.retention() {
            EditorMessageRetention::Lossless => self.enqueue_lossless(delivery),
            EditorMessageRetention::Latest(key) => self.enqueue_latest(key, delivery),
            EditorMessageRetention::Bounded => self.enqueue_bounded(delivery),
        }
    }

    pub(super) fn can_enqueue_lossless(&self, retained_bytes: usize) -> bool {
        self.lossless_depth < self.limits.lossless_capacity && self.can_add_bytes(retained_bytes)
    }

    pub(super) fn note_lossless_backpressure(&mut self) {
        self.backpressured = self.backpressured.saturating_add(1);
    }

    pub(super) fn stats(&self, current_sequence: u64) -> EditorMessageInboxStats {
        let age_in_messages = self
            .deliveries
            .first_key_value()
            .map(|(_, delivery)| current_sequence.saturating_sub(delivery.sequence()))
            .unwrap_or_default();
        EditorMessageInboxStats {
            depth: self.deliveries.len(),
            lossless_depth: self.lossless_depth,
            bounded_depth: self.bounded_depth,
            latest_depth: self.latest_depth,
            retained_bytes: self.retained_bytes,
            drained: self.drained,
            coalesced: self.coalesced,
            dropped: self.dropped,
            backpressured: self.backpressured,
            age_in_messages,
        }
    }

    fn enqueue_lossless(&mut self, delivery: EditorMessageDelivery) -> EditorMessageInboxEnqueue {
        if !self.can_enqueue_lossless(delivery.retained_bytes()) {
            self.note_lossless_backpressure();
            return EditorMessageInboxEnqueue::Backpressured;
        }
        self.insert_delivery(delivery);
        self.lossless_depth = self.lossless_depth.saturating_add(1);
        EditorMessageInboxEnqueue::Enqueued
    }

    fn enqueue_latest(
        &mut self,
        key: EditorMessageCoalescingKey,
        delivery: EditorMessageDelivery,
    ) -> EditorMessageInboxEnqueue {
        if delivery.retained_bytes() > self.limits.max_delivery_bytes {
            self.dropped = self.dropped.saturating_add(1);
            return EditorMessageInboxEnqueue::Dropped;
        }

        if let Some(previous_sequence) = self.latest_by_key.get(&key).copied() {
            let Some(evictions) = self.latest_replacement_evictions_for(
                key,
                previous_sequence,
                delivery.retained_bytes(),
            ) else {
                self.dropped = self.dropped.saturating_add(1);
                return EditorMessageInboxEnqueue::Dropped;
            };

            self.remove_latest(key, previous_sequence);
            for (evicted_key, sequence) in &evictions {
                self.remove_latest(*evicted_key, *sequence);
            }
            self.dropped = self
                .dropped
                .saturating_add(u64::try_from(evictions.len()).unwrap_or(u64::MAX));
            let sequence = delivery.sequence();
            self.insert_delivery(delivery);
            self.latest_by_key.insert(key, sequence);
            self.latest_order.push_back((key, sequence));
            self.latest_depth = self.latest_depth.saturating_add(1);
            self.coalesced = self.coalesced.saturating_add(1);
            return if evictions.is_empty() {
                EditorMessageInboxEnqueue::Coalesced
            } else {
                EditorMessageInboxEnqueue::CoalescedAfterDrop
            };
        }

        let Some(evictions) = self.latest_evictions_for(delivery.retained_bytes()) else {
            self.dropped = self.dropped.saturating_add(1);
            return EditorMessageInboxEnqueue::Dropped;
        };
        for (evicted_key, sequence) in &evictions {
            self.remove_latest(*evicted_key, *sequence);
        }
        self.dropped = self
            .dropped
            .saturating_add(u64::try_from(evictions.len()).unwrap_or(u64::MAX));

        let sequence = delivery.sequence();
        self.insert_delivery(delivery);
        self.latest_by_key.insert(key, sequence);
        self.latest_order.push_back((key, sequence));
        self.latest_depth = self.latest_depth.saturating_add(1);
        if evictions.is_empty() {
            EditorMessageInboxEnqueue::Enqueued
        } else {
            EditorMessageInboxEnqueue::EnqueuedAfterDrop
        }
    }

    fn enqueue_bounded(&mut self, delivery: EditorMessageDelivery) -> EditorMessageInboxEnqueue {
        if delivery.retained_bytes() > self.limits.max_delivery_bytes {
            self.dropped = self.dropped.saturating_add(1);
            return EditorMessageInboxEnqueue::Dropped;
        }
        let Some(evictions) = self.bounded_evictions_for(delivery.retained_bytes()) else {
            self.dropped = self.dropped.saturating_add(1);
            return EditorMessageInboxEnqueue::Dropped;
        };
        for sequence in &evictions {
            self.remove_bounded(*sequence);
        }
        self.dropped = self
            .dropped
            .saturating_add(u64::try_from(evictions.len()).unwrap_or(u64::MAX));

        let sequence = delivery.sequence();
        self.insert_delivery(delivery);
        self.bounded_order.push_back(sequence);
        self.bounded_depth = self.bounded_depth.saturating_add(1);
        if evictions.is_empty() {
            EditorMessageInboxEnqueue::Enqueued
        } else {
            EditorMessageInboxEnqueue::EnqueuedAfterDrop
        }
    }

    fn latest_evictions_for(
        &self,
        incoming_bytes: usize,
    ) -> Option<Vec<(EditorMessageCoalescingKey, u64)>> {
        let mut retained_bytes = self.retained_bytes;
        let mut latest_depth = self.latest_depth;
        let mut evictions = Vec::new();
        for (key, sequence) in &self.latest_order {
            if latest_depth < self.limits.latest_capacity
                && retained_bytes
                    .checked_add(incoming_bytes)
                    .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity)
            {
                return Some(evictions);
            }
            let delivery = self.deliveries.get(sequence)?;
            retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
            latest_depth = latest_depth.checked_sub(1)?;
            evictions.push((*key, *sequence));
        }
        (latest_depth < self.limits.latest_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity))
        .then_some(evictions)
    }

    fn latest_replacement_evictions_for(
        &self,
        replaced_key: EditorMessageCoalescingKey,
        replaced_sequence: u64,
        incoming_bytes: usize,
    ) -> Option<Vec<(EditorMessageCoalescingKey, u64)>> {
        let replaced = self.deliveries.get(&replaced_sequence)?;
        let mut retained_bytes = self.retained_bytes.checked_sub(replaced.retained_bytes())?;
        let mut latest_depth = self.latest_depth.checked_sub(1)?;
        let mut evictions = Vec::new();
        for (key, sequence) in &self.latest_order {
            if (*key, *sequence) == (replaced_key, replaced_sequence) {
                continue;
            }
            if latest_depth < self.limits.latest_capacity
                && retained_bytes
                    .checked_add(incoming_bytes)
                    .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity)
            {
                return Some(evictions);
            }
            let delivery = self.deliveries.get(sequence)?;
            retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
            latest_depth = latest_depth.checked_sub(1)?;
            evictions.push((*key, *sequence));
        }
        (latest_depth < self.limits.latest_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity))
        .then_some(evictions)
    }

    fn bounded_evictions_for(&self, incoming_bytes: usize) -> Option<Vec<u64>> {
        let mut retained_bytes = self.retained_bytes;
        let mut bounded_depth = self.bounded_depth;
        let mut evictions = Vec::new();
        for sequence in &self.bounded_order {
            if bounded_depth < self.limits.bounded_capacity
                && retained_bytes
                    .checked_add(incoming_bytes)
                    .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity)
            {
                return Some(evictions);
            }
            let delivery = self.deliveries.get(sequence)?;
            retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
            bounded_depth = bounded_depth.checked_sub(1)?;
            evictions.push(*sequence);
        }
        (bounded_depth < self.limits.bounded_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity))
        .then_some(evictions)
    }

    fn insert_delivery(&mut self, delivery: EditorMessageDelivery) {
        self.retained_bytes = self
            .retained_bytes
            .saturating_add(delivery.retained_bytes());
        self.deliveries.insert(delivery.sequence(), delivery);
    }

    fn remove_latest(&mut self, key: EditorMessageCoalescingKey, sequence: u64) {
        self.latest_by_key.remove(&key);
        if let Some(index) = self
            .latest_order
            .iter()
            .position(|candidate| *candidate == (key, sequence))
        {
            self.latest_order.remove(index);
        }
        if let Some(delivery) = self.deliveries.remove(&sequence) {
            self.retained_bytes = self
                .retained_bytes
                .saturating_sub(delivery.retained_bytes());
            self.latest_depth = self.latest_depth.saturating_sub(1);
        }
    }

    fn remove_bounded(&mut self, sequence: u64) {
        if self.bounded_order.front().copied() == Some(sequence) {
            self.bounded_order.pop_front();
        } else if let Some(index) = self
            .bounded_order
            .iter()
            .position(|candidate| *candidate == sequence)
        {
            self.bounded_order.remove(index);
        }
        if let Some(delivery) = self.deliveries.remove(&sequence) {
            self.retained_bytes = self
                .retained_bytes
                .saturating_sub(delivery.retained_bytes());
            self.bounded_depth = self.bounded_depth.saturating_sub(1);
        }
    }

    fn can_add_bytes(&self, incoming_bytes: usize) -> bool {
        incoming_bytes <= self.limits.max_delivery_bytes
            && self
                .retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity)
    }
}
