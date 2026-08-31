use std::collections::VecDeque;
use std::time::Instant;

use zircon_runtime_interface::ZrRuntimePluginEventDeliveryV1;

use super::super::EditorRuntimeEventConsumerDeliveryDisposition;
use super::{ActiveConsumerSnapshot, EditorRuntimeEventConsumerHost};

const DEFAULT_MAX_RETAINED_PENDING_BYTES: usize = 1024 * 1024;

/// Host-wide admission limit for plugin deliveries retained between editor pump ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorRuntimeEventConsumerPendingDeliveryBudget {
    max_retained_bytes: usize,
}

impl EditorRuntimeEventConsumerPendingDeliveryBudget {
    pub const fn new(max_retained_bytes: usize) -> Self {
        Self { max_retained_bytes }
    }

    pub const fn max_retained_bytes(self) -> usize {
        self.max_retained_bytes
    }
}

impl Default for EditorRuntimeEventConsumerPendingDeliveryBudget {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_RETAINED_PENDING_BYTES)
    }
}

pub(super) struct PendingDelivery {
    delivery: ZrRuntimePluginEventDeliveryV1,
    disposition: Option<EditorRuntimeEventConsumerDeliveryDisposition>,
    retry_count: u32,
    first_seen: Instant,
    retained_bytes_upper_bound: usize,
}

impl PendingDelivery {
    fn new(
        delivery: ZrRuntimePluginEventDeliveryV1,
        first_seen: Instant,
        retained_bytes_upper_bound: usize,
    ) -> Self {
        Self {
            delivery,
            disposition: None,
            retry_count: 0,
            first_seen,
            retained_bytes_upper_bound,
        }
    }

    pub(super) fn delivery(&self) -> &ZrRuntimePluginEventDeliveryV1 {
        &self.delivery
    }

    pub(super) fn first_seen(&self) -> Instant {
        self.first_seen
    }

    pub(super) fn retained_bytes_upper_bound(&self) -> usize {
        self.retained_bytes_upper_bound
    }

    pub(super) fn mark_disposition(
        &mut self,
        disposition: EditorRuntimeEventConsumerDeliveryDisposition,
    ) {
        self.disposition = Some(disposition);
    }

    pub(super) fn mark_retryable(&mut self) {
        self.disposition = Some(EditorRuntimeEventConsumerDeliveryDisposition::Retryable);
        self.retry_count = self.retry_count.saturating_add(1);
    }

    pub(super) fn disposition(&self) -> Option<EditorRuntimeEventConsumerDeliveryDisposition> {
        self.disposition
    }

    pub(super) fn retry_count(&self) -> u32 {
        self.retry_count
    }
}

pub(super) struct PendingDeliveryBatch {
    deliveries: VecDeque<PendingDelivery>,
    current: Option<PendingDelivery>,
    last_sequence: Option<u64>,
    retained_bytes_upper_bound: usize,
}

impl PendingDeliveryBatch {
    pub(super) fn from_page(
        deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
        encoded_bytes_upper_bound: usize,
    ) -> Self {
        let delivery_count = deliveries.len();
        let base_bytes = encoded_bytes_upper_bound / delivery_count.max(1);
        let remainder = encoded_bytes_upper_bound % delivery_count.max(1);
        let first_seen = Instant::now();
        let deliveries = deliveries
            .into_iter()
            .enumerate()
            .map(|(index, delivery)| {
                PendingDelivery::new(
                    delivery,
                    first_seen,
                    base_bytes + usize::from(index < remainder),
                )
            })
            .collect();
        Self {
            deliveries,
            current: None,
            last_sequence: None,
            retained_bytes_upper_bound: if delivery_count == 0 {
                0
            } else {
                encoded_bytes_upper_bound
            },
        }
    }

    pub(super) fn from_pending(
        deliveries: VecDeque<PendingDelivery>,
        last_sequence: Option<u64>,
        retained_bytes_upper_bound: usize,
    ) -> Self {
        Self {
            deliveries,
            current: None,
            last_sequence,
            retained_bytes_upper_bound,
        }
    }

    pub(super) fn is_empty(&self) -> bool {
        self.current.is_none() && self.deliveries.is_empty()
    }

    pub(super) fn len(&self) -> usize {
        self.deliveries.len() + usize::from(self.current.is_some())
    }

    pub(super) fn retained_bytes_upper_bound(&self) -> usize {
        self.retained_bytes_upper_bound
    }

    pub(super) fn last_sequence(&self) -> Option<u64> {
        self.last_sequence
    }

    pub(super) fn set_last_sequence(&mut self, sequence: u64) {
        self.last_sequence = Some(sequence);
    }

    pub(super) fn begin_current(&mut self) -> Option<&PendingDelivery> {
        debug_assert!(self.current.is_none());
        self.current = self.deliveries.pop_front();
        self.current.as_ref()
    }

    pub(super) fn complete_current(
        &mut self,
        disposition: EditorRuntimeEventConsumerDeliveryDisposition,
    ) -> PendingDelivery {
        let mut delivery = self
            .current
            .take()
            .expect("an executing pending delivery must be present");
        self.retained_bytes_upper_bound = self
            .retained_bytes_upper_bound
            .saturating_sub(delivery.retained_bytes_upper_bound());
        delivery.mark_disposition(disposition);
        delivery
    }

    pub(super) fn retry_current(&mut self) {
        let mut delivery = self
            .current
            .take()
            .expect("an executing pending delivery must be present");
        delivery.mark_retryable();
        self.deliveries.push_front(delivery);
    }

    pub(super) fn discard(&mut self) -> (usize, usize) {
        let discarded = (self.len(), self.retained_bytes_upper_bound);
        self.deliveries.clear();
        self.current = None;
        self.retained_bytes_upper_bound = 0;
        discarded
    }

    pub(super) fn into_pending(mut self) -> (VecDeque<PendingDelivery>, Option<u64>) {
        if let Some(current) = self.current.take() {
            self.deliveries.push_front(current);
        }
        (self.deliveries, self.last_sequence)
    }

    pub(super) fn first_sequence(&self) -> Option<u64> {
        self.current
            .as_ref()
            .or_else(|| self.deliveries.front())
            .map(|delivery| delivery.delivery().sequence)
    }
}

/// Restores the unprocessed delivery tail if a callback cannot reach its normal commit.
///
/// The pump owns the executing delivery's disposition separately. It can commit an applied
/// sequence, record a poisoned callback delivery, or later apply a retry policy without conflating
/// that decision with the unprocessed tail.
pub(super) struct PendingDeliveryBatchRestoreGuard<'a> {
    host: &'a EditorRuntimeEventConsumerHost,
    snapshot: &'a ActiveConsumerSnapshot,
    batch: Option<PendingDeliveryBatch>,
}

impl<'a> PendingDeliveryBatchRestoreGuard<'a> {
    pub(super) fn new(
        host: &'a EditorRuntimeEventConsumerHost,
        snapshot: &'a ActiveConsumerSnapshot,
        batch: PendingDeliveryBatch,
    ) -> Self {
        Self {
            host,
            snapshot,
            batch: Some(batch),
        }
    }

    pub(super) fn batch(&self) -> &PendingDeliveryBatch {
        self.batch
            .as_ref()
            .expect("pending delivery batch remains owned until it is restored")
    }

    pub(super) fn batch_mut(&mut self) -> &mut PendingDeliveryBatch {
        self.batch
            .as_mut()
            .expect("pending delivery batch remains owned until it is restored")
    }

    pub(super) fn restore(&mut self) -> bool {
        let Some(batch) = self.batch.take() else {
            return true;
        };
        let retained_bytes = batch.retained_bytes_upper_bound();
        if self.host.restore_pending_batch(self.snapshot, batch) {
            true
        } else {
            // A replacement or local retirement won the ownership race. The batch has no
            // surviving owner, so its reservation must leave the shared retention ledger.
            self.host.release_pending_bytes(retained_bytes);
            false
        }
    }

    pub(super) fn discard(&mut self) -> (usize, usize) {
        self.batch
            .take()
            .map_or((0, 0), |mut batch| batch.discard())
    }
}

impl Drop for PendingDeliveryBatchRestoreGuard<'_> {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

impl EditorRuntimeEventConsumerHost {
    pub(super) fn append_drained_deliveries(
        &self,
        snapshot: &ActiveConsumerSnapshot,
        deliveries: Vec<ZrRuntimePluginEventDeliveryV1>,
        encoded_bytes_upper_bound: usize,
        runtime_remaining_deliveries: usize,
        runtime_oldest_pending_age_millis: u64,
    ) -> usize {
        let pending = PendingDeliveryBatch::from_page(deliveries, encoded_bytes_upper_bound);
        let dropped = pending.len();
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(consumer) = active.get_mut(&snapshot.consumer_id).filter(|consumer| {
            consumer.generation == snapshot.generation
                && consumer.subscription == snapshot.subscription
        }) else {
            return dropped;
        };
        debug_assert!(consumer.pending.is_empty());
        consumer.last_observed_runtime_remaining_deliveries = Some(runtime_remaining_deliveries);
        consumer.last_observed_runtime_oldest_pending_age_millis =
            Some(runtime_oldest_pending_age_millis);
        consumer.runtime_backlog_observed_at = Some(Instant::now());
        if pending.is_empty() {
            return 0;
        }
        let retained_bytes_upper_bound = pending.retained_bytes_upper_bound();
        if !self.try_reserve_pending_bytes(retained_bytes_upper_bound) {
            return dropped;
        }
        let (deliveries, _) = pending.into_pending();
        consumer.pending = deliveries;
        consumer.pending_retained_bytes = retained_bytes_upper_bound;
        0
    }

    pub(super) fn take_pending_batch(
        &self,
        snapshot: &ActiveConsumerSnapshot,
    ) -> Option<PendingDeliveryBatch> {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let consumer = active.get_mut(&snapshot.consumer_id)?;
        if consumer.generation != snapshot.generation
            || consumer.subscription != snapshot.subscription
        {
            return None;
        }
        if consumer.pending.is_empty() {
            return None;
        }
        Some(PendingDeliveryBatch::from_pending(
            std::mem::take(&mut consumer.pending),
            consumer.last_sequence,
            std::mem::take(&mut consumer.pending_retained_bytes),
        ))
    }

    pub(super) fn restore_pending_batch(
        &self,
        snapshot: &ActiveConsumerSnapshot,
        pending: PendingDeliveryBatch,
    ) -> bool {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(consumer) = active.get_mut(&snapshot.consumer_id).filter(|consumer| {
            consumer.generation == snapshot.generation
                && consumer.subscription == snapshot.subscription
        }) else {
            return false;
        };
        let retained_bytes_upper_bound = pending.retained_bytes_upper_bound();
        let (deliveries, last_sequence) = pending.into_pending();
        consumer.last_sequence = last_sequence;
        consumer.pending = deliveries;
        consumer.pending_retained_bytes = retained_bytes_upper_bound;
        true
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{
        ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
    };

    use super::{EditorRuntimeEventConsumerDeliveryDisposition, PendingDeliveryBatch};

    #[test]
    fn page_bytes_are_partitioned_across_pending_deliveries() {
        let deliveries = (1..=3)
            .map(|sequence| {
                ZrRuntimePluginEventDeliveryV1::new(
                    7,
                    ZrRuntimePluginEventSubscriptionHandle::new(11),
                    "tests.events.pending",
                    "tests.events.pending.v1",
                    sequence,
                    serde_json::json!({ "value": sequence }),
                )
            })
            .collect();
        let mut batch = PendingDeliveryBatch::from_page(deliveries, 10);

        assert_eq!(batch.retained_bytes_upper_bound(), 10);
        assert_eq!(
            batch.begin_current().unwrap().retained_bytes_upper_bound(),
            4
        );
        batch.complete_current(EditorRuntimeEventConsumerDeliveryDisposition::Applied);
        assert_eq!(batch.retained_bytes_upper_bound(), 6);
        assert_eq!(batch.first_sequence(), Some(2));

        batch.begin_current();
        batch.retry_current();
        let retrying = batch.begin_current().unwrap();
        assert_eq!(
            retrying.disposition(),
            Some(EditorRuntimeEventConsumerDeliveryDisposition::Retryable)
        );
        assert_eq!(retrying.retry_count(), 1);
    }
}
