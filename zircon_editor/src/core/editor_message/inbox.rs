use std::collections::{BTreeMap, BTreeSet};

use super::retention::{EditorMessageCoalescingKey, EditorMessageRetention};
use super::EditorMessageDelivery;

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
    latest_order: BTreeMap<u64, EditorMessageCoalescingKey>,
    bounded_order: BTreeSet<u64>,
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
            latest_order: BTreeMap::new(),
            bounded_order: BTreeSet::new(),
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
        if let Some(previous_sequence) = self.latest_by_key.get(&key).copied() {
            if previous_sequence >= delivery.sequence() {
                self.coalesced = self.coalesced.saturating_add(1);
                return EditorMessageInboxEnqueue::Coalesced;
            }
            let Some(previous) = self.deliveries.get(&previous_sequence) else {
                self.dropped = self.dropped.saturating_add(1);
                return EditorMessageInboxEnqueue::Dropped;
            };
            let delivery = delivery.coalesce_latest_from(previous);
            if delivery.retained_bytes() > self.limits.max_delivery_bytes {
                self.dropped = self.dropped.saturating_add(1);
                return EditorMessageInboxEnqueue::Dropped;
            }
            let Some(eviction_count) = self.latest_replacement_eviction_count_for(
                key,
                previous_sequence,
                delivery.sequence(),
                delivery.retained_bytes(),
            ) else {
                self.dropped = self.dropped.saturating_add(1);
                return EditorMessageInboxEnqueue::Dropped;
            };

            self.remove_latest(key, previous_sequence);
            for _ in 0..eviction_count {
                self.remove_oldest_latest();
            }
            self.dropped = self
                .dropped
                .saturating_add(u64::try_from(eviction_count).unwrap_or(u64::MAX));
            let sequence = delivery.sequence();
            self.insert_delivery(delivery);
            self.latest_by_key.insert(key, sequence);
            self.latest_order.insert(sequence, key);
            self.latest_depth = self.latest_depth.saturating_add(1);
            self.coalesced = self.coalesced.saturating_add(1);
            return if eviction_count == 0 {
                EditorMessageInboxEnqueue::Coalesced
            } else {
                EditorMessageInboxEnqueue::CoalescedAfterDrop
            };
        }

        if delivery.retained_bytes() > self.limits.max_delivery_bytes {
            self.dropped = self.dropped.saturating_add(1);
            return EditorMessageInboxEnqueue::Dropped;
        }

        let Some(eviction_count) =
            self.latest_eviction_count_for(delivery.sequence(), delivery.retained_bytes())
        else {
            self.dropped = self.dropped.saturating_add(1);
            return EditorMessageInboxEnqueue::Dropped;
        };
        for _ in 0..eviction_count {
            self.remove_oldest_latest();
        }
        self.dropped = self
            .dropped
            .saturating_add(u64::try_from(eviction_count).unwrap_or(u64::MAX));

        let sequence = delivery.sequence();
        self.insert_delivery(delivery);
        self.latest_by_key.insert(key, sequence);
        self.latest_order.insert(sequence, key);
        self.latest_depth = self.latest_depth.saturating_add(1);
        if eviction_count == 0 {
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
        let Some(eviction_count) =
            self.bounded_eviction_count_for(delivery.sequence(), delivery.retained_bytes())
        else {
            self.dropped = self.dropped.saturating_add(1);
            return EditorMessageInboxEnqueue::Dropped;
        };
        for _ in 0..eviction_count {
            self.remove_oldest_bounded();
        }
        self.dropped = self
            .dropped
            .saturating_add(u64::try_from(eviction_count).unwrap_or(u64::MAX));

        let sequence = delivery.sequence();
        self.insert_delivery(delivery);
        self.bounded_order.insert(sequence);
        self.bounded_depth = self.bounded_depth.saturating_add(1);
        if eviction_count == 0 {
            EditorMessageInboxEnqueue::Enqueued
        } else {
            EditorMessageInboxEnqueue::EnqueuedAfterDrop
        }
    }

    fn latest_eviction_count_for(
        &self,
        incoming_sequence: u64,
        incoming_bytes: usize,
    ) -> Option<usize> {
        let mut retained_bytes = self.retained_bytes;
        let mut latest_depth = self.latest_depth;
        let mut eviction_count = 0;
        for sequence in self.latest_order.keys() {
            if latest_depth < self.limits.latest_capacity
                && retained_bytes
                    .checked_add(incoming_bytes)
                    .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity)
            {
                return Some(eviction_count);
            }
            if *sequence >= incoming_sequence {
                return None;
            }
            let delivery = self.deliveries.get(sequence)?;
            retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
            latest_depth = latest_depth.checked_sub(1)?;
            eviction_count += 1;
        }
        (latest_depth < self.limits.latest_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity))
        .then_some(eviction_count)
    }

    fn latest_replacement_eviction_count_for(
        &self,
        replaced_key: EditorMessageCoalescingKey,
        replaced_sequence: u64,
        incoming_sequence: u64,
        incoming_bytes: usize,
    ) -> Option<usize> {
        let replaced = self.deliveries.get(&replaced_sequence)?;
        let mut retained_bytes = self.retained_bytes.checked_sub(replaced.retained_bytes())?;
        let mut latest_depth = self.latest_depth.checked_sub(1)?;
        let mut eviction_count = 0;
        for (sequence, key) in &self.latest_order {
            if (*key, *sequence) == (replaced_key, replaced_sequence) {
                continue;
            }
            if latest_depth < self.limits.latest_capacity
                && retained_bytes
                    .checked_add(incoming_bytes)
                    .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity)
            {
                return Some(eviction_count);
            }
            if *sequence >= incoming_sequence {
                return None;
            }
            let delivery = self.deliveries.get(sequence)?;
            retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
            latest_depth = latest_depth.checked_sub(1)?;
            eviction_count += 1;
        }
        (latest_depth < self.limits.latest_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity))
        .then_some(eviction_count)
    }

    fn bounded_eviction_count_for(
        &self,
        incoming_sequence: u64,
        incoming_bytes: usize,
    ) -> Option<usize> {
        let mut retained_bytes = self.retained_bytes;
        let mut bounded_depth = self.bounded_depth;
        let mut eviction_count = 0;
        for sequence in &self.bounded_order {
            if bounded_depth < self.limits.bounded_capacity
                && retained_bytes
                    .checked_add(incoming_bytes)
                    .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity)
            {
                return Some(eviction_count);
            }
            if *sequence >= incoming_sequence {
                return None;
            }
            let delivery = self.deliveries.get(sequence)?;
            retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
            bounded_depth = bounded_depth.checked_sub(1)?;
            eviction_count += 1;
        }
        (bounded_depth < self.limits.bounded_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= self.limits.retained_bytes_capacity))
        .then_some(eviction_count)
    }

    fn insert_delivery(&mut self, delivery: EditorMessageDelivery) {
        self.retained_bytes = self
            .retained_bytes
            .saturating_add(delivery.retained_bytes());
        self.deliveries.insert(delivery.sequence(), delivery);
    }

    fn remove_latest(&mut self, key: EditorMessageCoalescingKey, sequence: u64) {
        self.latest_by_key.remove(&key);
        self.latest_order.remove(&sequence);
        if let Some(delivery) = self.deliveries.remove(&sequence) {
            self.retained_bytes = self
                .retained_bytes
                .saturating_sub(delivery.retained_bytes());
            self.latest_depth = self.latest_depth.saturating_sub(1);
        }
    }

    fn remove_oldest_latest(&mut self) {
        let (sequence, key) = self
            .latest_order
            .pop_first()
            .expect("a planned latest eviction must have an indexed delivery");
        self.latest_by_key.remove(&key);
        let delivery = self
            .deliveries
            .remove(&sequence)
            .expect("a planned latest eviction must have a retained delivery");
        self.retained_bytes = self
            .retained_bytes
            .saturating_sub(delivery.retained_bytes());
        self.latest_depth = self.latest_depth.saturating_sub(1);
    }

    fn remove_oldest_bounded(&mut self) {
        let sequence = self
            .bounded_order
            .pop_first()
            .expect("a planned bounded eviction must have an indexed delivery");
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

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use crate::core::editor_message::{
        EditorMessage, EditorMessagePayload, EditorMessageProtocol, EditorTopic, FocusMessage,
        SelectionDomain,
    };
    use crate::core::play::PlayInstanceId;

    use super::{EditorMessageDelivery, EditorMessageInbox, EditorMessageInboxLimits};

    const PLANNER_ITERATIONS: usize = 50_000;
    const SAMPLE_PAIRS: usize = 17;

    fn latest_delivery(sequence: u64, revision: u64) -> EditorMessageDelivery {
        latest_delivery_in_domain(sequence, revision, SelectionDomain::edit_scene())
    }

    fn latest_delivery_in_domain(
        sequence: u64,
        revision: u64,
        domain: SelectionDomain,
    ) -> EditorMessageDelivery {
        EditorMessageDelivery::with_sequence(
            EditorMessageProtocol::Publish,
            EditorTopic::parse("editor.inbox.order").expect("valid inbox test topic"),
            EditorMessage::new(EditorMessagePayload::Focus(
                FocusMessage::SelectionChanged { domain, revision },
            )),
            sequence,
        )
    }

    fn bounded_delivery(sequence: u64, revision: u64) -> EditorMessageDelivery {
        EditorMessageDelivery::with_sequence(
            EditorMessageProtocol::Publish,
            EditorTopic::parse("editor.inbox.order").expect("valid inbox test topic"),
            EditorMessage::custom(
                crate::core::editor_message::EditorMessageSchemaId::editor("inbox.order.v1")
                    .unwrap(),
                serde_json::json!({ "revision": revision }),
            ),
            sequence,
        )
    }

    fn legacy_bounded_evictions_for(
        inbox: &EditorMessageInbox,
        incoming_sequence: u64,
        incoming_bytes: usize,
    ) -> Option<Vec<u64>> {
        let mut retained_bytes = inbox.retained_bytes;
        let mut bounded_depth = inbox.bounded_depth;
        let mut evictions = Vec::new();
        for sequence in &inbox.bounded_order {
            if bounded_depth < inbox.limits.bounded_capacity
                && retained_bytes
                    .checked_add(incoming_bytes)
                    .is_some_and(|bytes| bytes <= inbox.limits.retained_bytes_capacity)
            {
                return Some(evictions);
            }
            if *sequence >= incoming_sequence {
                return None;
            }
            let delivery = inbox.deliveries.get(sequence)?;
            retained_bytes = retained_bytes.checked_sub(delivery.retained_bytes())?;
            bounded_depth = bounded_depth.checked_sub(1)?;
            evictions.push(*sequence);
        }
        (bounded_depth < inbox.limits.bounded_capacity
            && retained_bytes
                .checked_add(incoming_bytes)
                .is_some_and(|bytes| bytes <= inbox.limits.retained_bytes_capacity))
        .then_some(evictions)
    }

    fn full_bounded_inbox(capacity: usize) -> EditorMessageInbox {
        let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::new(1, capacity, 1));
        for sequence in 0..u64::try_from(capacity).unwrap() {
            inbox.enqueue(bounded_delivery(sequence, sequence));
        }
        inbox
    }

    fn elapsed_micros(run: impl FnOnce()) -> u128 {
        let started = Instant::now();
        run();
        started.elapsed().as_micros()
    }

    fn nearest_rank_p95(samples: &mut [u128]) -> u128 {
        samples.sort_unstable();
        let rank = (samples.len() * 95).div_ceil(100);
        samples[rank.saturating_sub(1)]
    }

    #[test]
    fn out_of_order_latest_delivery_keeps_the_highest_sequence() {
        let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::default());
        inbox.enqueue(latest_delivery(2, 2));
        inbox.enqueue(latest_delivery(1, 1));

        let deliveries = inbox.deliveries();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].sequence(), 2);
        assert_eq!(
            deliveries[0].message(),
            &EditorMessage::new(EditorMessagePayload::Focus(
                FocusMessage::SelectionChanged {
                    domain: SelectionDomain::edit_scene(),
                    revision: 2,
                },
            ))
        );
    }

    #[test]
    fn latest_selection_delivery_is_partitioned_by_play_instance() {
        let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::default());
        let first = PlayInstanceId::for_test(1);
        let second = PlayInstanceId::for_test(2);

        inbox.enqueue(latest_delivery_in_domain(
            1,
            1,
            SelectionDomain::edit_scene(),
        ));
        inbox.enqueue(latest_delivery_in_domain(
            2,
            1,
            SelectionDomain::play_scene(first),
        ));
        inbox.enqueue(latest_delivery_in_domain(
            3,
            1,
            SelectionDomain::play_scene(second),
        ));
        inbox.enqueue(latest_delivery_in_domain(
            4,
            2,
            SelectionDomain::play_scene(first),
        ));

        assert_eq!(
            inbox
                .deliveries()
                .iter()
                .map(EditorMessageDelivery::sequence)
                .collect::<Vec<_>>(),
            [1, 3, 4]
        );
        assert_eq!(inbox.stats(4).coalesced(), 1);
    }

    #[test]
    fn out_of_order_bounded_delivery_evicts_the_lowest_sequence() {
        let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::new(1, 1, 1));
        inbox.enqueue(bounded_delivery(2, 2));
        inbox.enqueue(bounded_delivery(1, 1));

        let deliveries = inbox.deliveries();
        assert_eq!(deliveries.len(), 1);
        assert_eq!(deliveries[0].sequence(), 2);
    }

    #[test]
    fn optimization_batch_20260826d_editor48_inbox_eviction_plans_store_only_counts() {
        let source = include_str!("inbox.rs")
            .split_once("#[cfg(test)]\nmod tests")
            .unwrap()
            .0;

        assert!(source.contains("fn bounded_eviction_count_for("));
        assert!(source.contains("fn latest_eviction_count_for("));
        assert!(source.contains("fn latest_replacement_eviction_count_for("));
        assert!(source.contains("bounded_order.pop_first()"));
        assert!(source.contains("latest_order.pop_first()"));
        assert!(!source.contains("let mut evictions = Vec::new()"));
    }

    #[test]
    fn optimization_batch_20260826d_editor48_inbox_rolling_eviction_preserves_order_and_stats() {
        let mut inbox = EditorMessageInbox::new(EditorMessageInboxLimits::new(1, 8, 1));
        for sequence in 0..64 {
            inbox.enqueue(bounded_delivery(sequence, sequence));
        }

        assert_eq!(
            inbox
                .deliveries()
                .iter()
                .map(EditorMessageDelivery::sequence)
                .collect::<Vec<_>>(),
            (56..64).collect::<Vec<_>>()
        );
        let stats = inbox.stats(64);
        assert_eq!(stats.depth(), 8);
        assert_eq!(stats.bounded_depth(), 8);
        assert_eq!(stats.dropped(), 56);
    }

    #[test]
    #[ignore = "release performance evidence for the managed validation coordinator"]
    fn optimization_batch_20260826d_editor48_inbox_eviction_plan_performance_evidence() {
        let inbox = full_bounded_inbox(1_024);
        let incoming = bounded_delivery(1_024, 1_024);
        let incoming_bytes = incoming.retained_bytes();

        assert_eq!(
            legacy_bounded_evictions_for(&inbox, incoming.sequence(), incoming_bytes)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            inbox
                .bounded_eviction_count_for(incoming.sequence(), incoming_bytes)
                .unwrap(),
            1
        );

        let measure_legacy = || {
            elapsed_micros(|| {
                for _ in 0..PLANNER_ITERATIONS {
                    black_box(
                        legacy_bounded_evictions_for(
                            black_box(&inbox),
                            incoming.sequence(),
                            incoming_bytes,
                        )
                        .unwrap(),
                    );
                }
            })
        };
        let measure_optimized = || {
            elapsed_micros(|| {
                for _ in 0..PLANNER_ITERATIONS {
                    black_box(
                        inbox
                            .bounded_eviction_count_for(incoming.sequence(), incoming_bytes)
                            .unwrap(),
                    );
                }
            })
        };
        for _ in 0..3 {
            black_box(measure_legacy());
            black_box(measure_optimized());
        }

        let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
        for sample_index in 0..SAMPLE_PAIRS {
            if sample_index % 2 == 0 {
                legacy_samples.push(measure_legacy());
                optimized_samples.push(measure_optimized());
            } else {
                optimized_samples.push(measure_optimized());
                legacy_samples.push(measure_legacy());
            }
        }

        let legacy_p95 = nearest_rank_p95(&mut legacy_samples);
        let optimized_p95 = nearest_rank_p95(&mut optimized_samples);
        println!(
            "EDITOR48_INBOX_EVICTION_COUNT_PLAN_BENCH_V1 sample_pairs={} planner_iterations={} retained_deliveries={} evictions_per_plan=1 legacy_temporary_plan_allocations={} optimized_temporary_plan_allocations=0 legacy_copied_sequences={} optimized_copied_sequences=0 legacy_p95_us={} optimized_p95_us={} legacy_samples_us={:?} optimized_samples_us={:?}",
            SAMPLE_PAIRS,
            PLANNER_ITERATIONS,
            inbox.deliveries.len(),
            PLANNER_ITERATIONS,
            PLANNER_ITERATIONS,
            legacy_p95,
            optimized_p95,
            legacy_samples,
            optimized_samples,
        );
        assert!(
            optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(70),
            "count-only eviction planning p95 must be at least 30% below allocating sequence plans: legacy={legacy_p95}us optimized={optimized_p95}us"
        );
    }
}
