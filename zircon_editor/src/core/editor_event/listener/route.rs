use std::sync::{Arc, Mutex, MutexGuard};

use super::super::{
    EditorEventRecord, EditorEventRetentionPage, EditorEventRetentionStore, SharedEditorEventRecord,
};
use super::{EditorEventListenerDescriptor, EditorEventListenerFilter, EditorEventListenerStatus};

const MAX_EDITOR_EVENT_LISTENER_DELIVERY_PAGE_SIZE: usize = 256;

#[derive(Clone, Debug)]
// 一次控制配置的投递快照；只共享收件箱，筛选条件固定在取快照时，允许注册表锁外完成入队。
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
// 查询句柄保留取得时的描述符和收件箱；配置已变化或监听器已注销时，句柄仍代表原来的在途状态。
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

    // 分页按收件箱投递游标推进，不能用全局事件序号代替；读取不确认，确认需要调用方处理完成后单独提交。
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
