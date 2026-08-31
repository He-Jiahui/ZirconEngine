use std::sync::Arc;
use std::time::Instant;

use crate::core::editing::operation::{
    DeferredOperationInvocation, EditOperationTarget, PendingEditRetention,
};
use crate::core::editor_operation::EditorOperationInvocation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PendingEditId(u64);

impl PendingEditId {
    pub(super) const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PendingEditIntent {
    pub id: PendingEditId,
    pub target: EditOperationTarget,
    pub invocation: Arc<EditorOperationInvocation>,
    pub retention: PendingEditRetention,
    payload_bytes: usize,
    enqueued_at: Instant,
    retry_count: u32,
}

impl PendingEditIntent {
    pub(super) fn new(
        id: PendingEditId,
        target: EditOperationTarget,
        deferred: DeferredOperationInvocation,
        payload_bytes: usize,
        enqueued_at: Instant,
    ) -> Self {
        let (invocation, registered_target, retention) = deferred.into_parts();
        Self {
            id,
            target: registered_target,
            invocation: Arc::new(invocation),
            retention,
            payload_bytes,
            enqueued_at,
            retry_count: 0,
        }
    }

    pub(super) fn replace(
        &mut self,
        target: EditOperationTarget,
        deferred: DeferredOperationInvocation,
        payload_bytes: usize,
        enqueued_at: Instant,
    ) {
        let (invocation, registered_target, retention) = deferred.into_parts();
        debug_assert_eq!(target, registered_target);
        self.target = registered_target;
        self.invocation = Arc::new(invocation);
        self.retention = retention;
        self.payload_bytes = payload_bytes;
        self.enqueued_at = enqueued_at;
        self.retry_count = 0;
    }

    pub(super) const fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }

    pub(super) const fn enqueued_at(&self) -> Instant {
        self.enqueued_at
    }

    pub(super) const fn retry_count(&self) -> u32 {
        self.retry_count
    }

    pub(super) fn mark_retry(&mut self) {
        self.retry_count = self.retry_count.saturating_add(1);
    }

    pub(super) fn belongs_to_cohort(
        &self,
        target: EditOperationTarget,
        invocation: &EditorOperationInvocation,
        retention: &PendingEditRetention,
    ) -> bool {
        self.target == target
            && self.invocation.operation_id == invocation.operation_id
            && self.retention.cohort_kind() == retention.cohort_kind()
    }
}
