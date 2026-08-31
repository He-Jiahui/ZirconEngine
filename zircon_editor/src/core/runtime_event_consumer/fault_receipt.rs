use std::collections::VecDeque;
use std::sync::Arc;

use zircon_runtime_interface::ZrRuntimePluginEventDeliveryV1;

use super::{
    EditorRuntimeEventConsumerCallbackPhase, EditorRuntimeEventConsumerDeliveryDisposition,
};

const DEFAULT_MAX_FAULT_RECEIPTS: usize = 64;
const DEFAULT_MAX_RETAINED_FAULT_PAYLOAD_BYTES: usize = 256 * 1024;

/// Bounded retention policy for event-consumer callback fault receipts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorRuntimeEventConsumerFaultReceiptBudget {
    max_receipts: usize,
    max_retained_payload_bytes: usize,
}

impl EditorRuntimeEventConsumerFaultReceiptBudget {
    pub const fn new(max_receipts: usize, max_retained_payload_bytes: usize) -> Self {
        Self {
            max_receipts: if max_receipts == 0 { 1 } else { max_receipts },
            max_retained_payload_bytes,
        }
    }

    pub const fn max_receipts(self) -> usize {
        self.max_receipts
    }

    pub const fn max_retained_payload_bytes(self) -> usize {
        self.max_retained_payload_bytes
    }
}

impl Default for EditorRuntimeEventConsumerFaultReceiptBudget {
    fn default() -> Self {
        Self::new(
            DEFAULT_MAX_FAULT_RECEIPTS,
            DEFAULT_MAX_RETAINED_FAULT_PAYLOAD_BYTES,
        )
    }
}

/// Immutable evidence for one callback fault. The host owns retention; logging and plugin UI
/// consume this projection rather than maintaining separate dead-letter stores.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorRuntimeEventConsumerFaultReceipt {
    consumer_id: Arc<str>,
    play_session_id: u64,
    phase: EditorRuntimeEventConsumerCallbackPhase,
    delivery_disposition: Option<EditorRuntimeEventConsumerDeliveryDisposition>,
    delivery_sequence: Option<u64>,
    event_id: Option<Arc<str>>,
    payload_schema: Option<Arc<str>>,
    payload_json: Option<Arc<str>>,
    payload_digest: Option<[u8; blake3::OUT_LEN]>,
    payload_was_truncated: bool,
    remote_cleanup_error: Option<Arc<str>>,
}

impl EditorRuntimeEventConsumerFaultReceipt {
    pub fn consumer_id(&self) -> &str {
        &self.consumer_id
    }

    pub const fn play_session_id(&self) -> u64 {
        self.play_session_id
    }

    pub const fn phase(&self) -> EditorRuntimeEventConsumerCallbackPhase {
        self.phase
    }

    pub const fn delivery_disposition(
        &self,
    ) -> Option<EditorRuntimeEventConsumerDeliveryDisposition> {
        self.delivery_disposition
    }

    pub const fn delivery_sequence(&self) -> Option<u64> {
        self.delivery_sequence
    }

    pub fn event_id(&self) -> Option<&str> {
        self.event_id.as_deref()
    }

    pub fn payload_schema(&self) -> Option<&str> {
        self.payload_schema.as_deref()
    }

    pub fn payload_json(&self) -> Option<&str> {
        self.payload_json.as_deref()
    }

    pub const fn payload_digest(&self) -> Option<[u8; blake3::OUT_LEN]> {
        self.payload_digest
    }

    pub const fn payload_was_truncated(&self) -> bool {
        self.payload_was_truncated
    }

    pub fn remote_cleanup_error(&self) -> Option<&str> {
        self.remote_cleanup_error.as_deref()
    }

    pub(super) fn callback_panicked(
        consumer_id: &str,
        play_session_id: u64,
        phase: EditorRuntimeEventConsumerCallbackPhase,
        delivery: Option<&ZrRuntimePluginEventDeliveryV1>,
        payload_json: Option<Arc<str>>,
        payload_was_truncated: bool,
        remote_cleanup_error: Option<Arc<str>>,
    ) -> Self {
        Self {
            consumer_id: Arc::from(consumer_id),
            play_session_id,
            phase,
            delivery_disposition: delivery
                .is_some()
                .then_some(EditorRuntimeEventConsumerDeliveryDisposition::Poison),
            delivery_sequence: delivery.map(|delivery| delivery.sequence),
            event_id: delivery.map(|delivery| Arc::from(delivery.event_id.as_str())),
            payload_schema: delivery.map(|delivery| Arc::from(delivery.payload_schema.as_str())),
            payload_digest: delivery
                .map(|delivery| *blake3::hash(delivery.payload.get().as_bytes()).as_bytes()),
            payload_json,
            payload_was_truncated,
            remote_cleanup_error,
        }
    }

    fn retained_payload_bytes(&self) -> usize {
        self.payload_json
            .as_ref()
            .map_or(0, |payload| payload.len())
    }
}

pub(super) struct EditorRuntimeEventConsumerFaultReceiptJournal {
    budget: EditorRuntimeEventConsumerFaultReceiptBudget,
    receipts: VecDeque<EditorRuntimeEventConsumerFaultReceipt>,
    retained_payload_bytes: usize,
}

impl EditorRuntimeEventConsumerFaultReceiptJournal {
    pub(super) fn new(budget: EditorRuntimeEventConsumerFaultReceiptBudget) -> Self {
        Self {
            budget,
            receipts: VecDeque::new(),
            retained_payload_bytes: 0,
        }
    }

    pub(super) fn record_callback_panic(
        &mut self,
        consumer_id: &str,
        play_session_id: u64,
        phase: EditorRuntimeEventConsumerCallbackPhase,
        delivery: Option<&ZrRuntimePluginEventDeliveryV1>,
        remote_cleanup_error: Option<&str>,
        mut reserve_payload_bytes: impl FnMut(usize) -> bool,
        mut release_payload_bytes: impl FnMut(usize),
    ) {
        let candidate_payload = delivery
            .map(|delivery| delivery.payload.get())
            .filter(|payload| payload.len() <= self.budget.max_retained_payload_bytes);
        let candidate_payload_bytes = candidate_payload.map_or(0, str::len);
        while self.receipts.len() >= self.budget.max_receipts
            || self
                .retained_payload_bytes
                .saturating_add(candidate_payload_bytes)
                > self.budget.max_retained_payload_bytes
        {
            let Some(evicted) = self.receipts.pop_front() else {
                break;
            };
            self.retained_payload_bytes = self
                .retained_payload_bytes
                .saturating_sub(evicted.retained_payload_bytes());
            release_payload_bytes(evicted.retained_payload_bytes());
        }
        let retained_payload =
            candidate_payload.filter(|payload| reserve_payload_bytes(payload.len()));
        let payload_was_truncated = delivery.is_some_and(|_| retained_payload.is_none());
        let receipt = EditorRuntimeEventConsumerFaultReceipt::callback_panicked(
            consumer_id,
            play_session_id,
            phase,
            delivery,
            retained_payload.map(Arc::from),
            payload_was_truncated,
            remote_cleanup_error.map(Arc::from),
        );
        self.retained_payload_bytes = self
            .retained_payload_bytes
            .saturating_add(receipt.retained_payload_bytes());
        self.receipts.push_back(receipt);
    }

    pub(super) fn snapshot(&self) -> Vec<EditorRuntimeEventConsumerFaultReceipt> {
        self.receipts.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{
        ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle,
    };

    use super::{
        EditorRuntimeEventConsumerCallbackPhase, EditorRuntimeEventConsumerFaultReceiptBudget,
        EditorRuntimeEventConsumerFaultReceiptJournal,
    };

    #[test]
    fn zero_receipt_capacity_keeps_one_fault_record() {
        assert_eq!(
            EditorRuntimeEventConsumerFaultReceiptBudget::new(0, 0).max_receipts(),
            1
        );

        let delivery = ZrRuntimePluginEventDeliveryV1::new(
            7,
            ZrRuntimePluginEventSubscriptionHandle::new(11),
            "tests.events.panic",
            "tests.events.panic.v1",
            1,
            serde_json::json!({ "payload": "must-not-be-copied" }),
        );
        let mut journal = EditorRuntimeEventConsumerFaultReceiptJournal::new(
            EditorRuntimeEventConsumerFaultReceiptBudget::new(0, 0),
        );
        journal.record_callback_panic(
            "tests.consumer.panic",
            7,
            EditorRuntimeEventConsumerCallbackPhase::Consume,
            Some(&delivery),
            None,
            |_| true,
            |_| {},
        );

        let receipts = journal.snapshot();
        assert_eq!(receipts.len(), 1);
        assert!(receipts[0].payload_json().is_none());
        assert!(receipts[0].payload_was_truncated());
        assert!(receipts[0].payload_digest().is_some());
    }
}
