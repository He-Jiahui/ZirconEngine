use std::sync::{Arc, Mutex, MutexGuard};

use super::super::{
    EditorEventRecord, EditorEventRetentionPage, EditorEventRetentionStore, SharedEditorEventRecord,
};
use super::{EditorEventListenerDescriptor, EditorEventListenerFilter, EditorEventListenerStatus};

const MAX_EDITOR_EVENT_LISTENER_DELIVERY_PAGE_SIZE: usize = 256;

#[derive(Clone, Debug)]
pub(crate) struct EditorEventListenerRoute {
    filter: Option<EditorEventListenerFilter>,
    inbox: Arc<Mutex<EditorEventRetentionStore>>,
}

impl EditorEventListenerRoute {
    pub(crate) fn new(
        filter: Option<EditorEventListenerFilter>,
        inbox: Arc<Mutex<EditorEventRetentionStore>>,
    ) -> Self {
        Self { filter, inbox }
    }

    pub(crate) fn accepts(&self, record: &EditorEventRecord) -> bool {
        self.filter
            .as_ref()
            .is_none_or(|filter| filter.accepts(record))
    }

    pub(crate) fn enqueue(&self, record: Arc<SharedEditorEventRecord>) {
        self.lock_inbox().push(record);
    }

    fn lock_inbox(&self) -> MutexGuard<'_, EditorEventRetentionStore> {
        self.inbox
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct EditorEventListenerHandle {
    descriptor: EditorEventListenerDescriptor,
    inbox: Arc<Mutex<EditorEventRetentionStore>>,
}

impl EditorEventListenerHandle {
    pub(crate) fn new(
        descriptor: EditorEventListenerDescriptor,
        inbox: Arc<Mutex<EditorEventRetentionStore>>,
    ) -> Self {
        Self { descriptor, inbox }
    }

    pub(crate) fn status(&self) -> EditorEventListenerStatus {
        let mut inbox = self.lock_inbox();
        let retention = inbox.diagnostics();
        let first_pending_sequence = retention.first_retained_sequence();
        let last_pending_sequence = retention.last_retained_sequence();
        let retention_budgets = inbox.budgets();
        EditorEventListenerStatus {
            listener_id: self.descriptor.listener_id.clone(),
            descriptor: self.descriptor.clone(),
            pending_delivery_count: retention.retained_records(),
            pending_delivery_bytes: retention.retained_bytes(),
            first_pending_sequence,
            last_pending_sequence,
            dropped_delivery_count: retention.dropped_records(),
            coalesced_delivery_count: retention.coalesced_records(),
            lagged_since_sequence: retention.first_dropped_sequence(),
            last_dropped_sequence: retention.last_dropped_sequence(),
            retention_budgets,
            retention,
        }
    }

    pub(crate) fn delivery_records_page_after_cursor(
        &self,
        after_delivery_cursor: u64,
        max_deliveries: usize,
    ) -> Result<EditorEventRetentionPage, String> {
        if !(1..=MAX_EDITOR_EVENT_LISTENER_DELIVERY_PAGE_SIZE).contains(&max_deliveries) {
            return Err(format!(
                "editor event listener delivery page size must be between 1 and {MAX_EDITOR_EVENT_LISTENER_DELIVERY_PAGE_SIZE}"
            ));
        }
        Ok(self
            .lock_inbox()
            .records_page_after_delivery_cursor(after_delivery_cursor, max_deliveries))
    }

    pub(crate) fn acknowledge_through_delivery_cursor(&self, delivery_cursor: u64) -> usize {
        self.lock_inbox()
            .acknowledge_through_delivery_cursor(delivery_cursor)
    }

    fn lock_inbox(&self) -> MutexGuard<'_, EditorEventRetentionStore> {
        self.inbox
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
