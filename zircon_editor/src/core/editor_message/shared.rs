use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editor_event::{EditorEventSequence, ViewInstanceId};
use zircon_runtime_interface::ui::event_ui::UiReflectionNodePatch;

use super::bus::EditorMessageDispatchPlan;
use super::{
    EditorMessage, EditorMessageBus, EditorMessageBusError, EditorMessageDelivery,
    EditorMessageDispatchReport, EditorMessageInboxLimits, EditorMessageInboxStats,
    EditorMessageResponse, EditorRequestHandler, EditorSubscriberId, EditorTopic,
    EditorUiDeltaBarrierKind, EditorUiDeltaBatch, EditorViewInvalidationMask, ViewDirtySet,
};

/// Thread-safe boundary for editor messaging; handlers run outside the bus lock.
#[derive(Clone, Debug, Default)]
pub struct SharedEditorMessageBus {
    inner: Arc<Mutex<EditorMessageBus>>,
}

impl SharedEditorMessageBus {
    pub fn with_inbox_limits(limits: EditorMessageInboxLimits) -> Self {
        Self {
            inner: Arc::new(Mutex::new(EditorMessageBus::with_inbox_limits(limits))),
        }
    }

    pub fn register_subscriber(
        &self,
        topics: impl IntoIterator<Item = EditorTopic>,
    ) -> Result<EditorSubscriberId, EditorMessageBusError> {
        self.lock().register_subscriber(topics)
    }

    pub fn unregister_subscriber(&self, subscriber: EditorSubscriberId) -> bool {
        self.lock().unregister_subscriber(subscriber)
    }

    /// 按主题订阅投递并返回逐目标结果；要求逐条保留的生产者须处理背压后重试，不能将报告忽略为已交付。
    pub fn publish(
        &self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        let prepared = { self.lock().prepare_publish(topic, message) };
        match prepared {
            Ok(plan) => self.finish_dispatch(plan),
            Err((report, _message)) => report,
        }
    }

    /// 向所有当前订阅者投递，绕过主题筛选；接收者仍需检查协议、主题及载荷。
    pub fn broadcast(
        &self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        let prepared = { self.lock().prepare_broadcast(topic, message) };
        match prepared {
            Ok(plan) => self.finish_dispatch(plan),
            Err((report, _message)) => report,
        }
    }

    /// 指定目标的同步回调入口；先保证请求入队，再在总线锁外调用处理器，因此处理器可重入。
    /// 返回错误可能来自回调后的目标复核；调用方不能把错误当作处理器从未执行或自动回滚。
    pub fn request(
        &self,
        target: EditorSubscriberId,
        topic: EditorTopic,
        message: EditorMessage,
        handler: &mut impl EditorRequestHandler,
    ) -> Result<EditorMessageResponse, EditorMessageBusError> {
        let (request, plan) = { self.lock().prepare_request(target, topic, message) }?;
        let report = self.finish_dispatch(plan);
        if report.backpressured().contains(&target) {
            return Err(EditorMessageBusError::Backpressured { subscriber: target });
        }
        let response = handler.handle_editor_request(&request);
        self.lock().complete_request(target, &response)?;
        Ok(response)
    }

    #[cfg(test)]
    pub fn deliveries_for(&self, subscriber: EditorSubscriberId) -> Vec<EditorMessageDelivery> {
        let inbox = { self.lock().inbox_handle(subscriber) };
        inbox
            .map(|inbox| {
                inbox
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .deliveries()
            })
            .unwrap_or_default()
    }

    /// 取出并移除该订阅的当前保留消息；最新态可能已经合并，未知或已注销的订阅返回空。
    pub fn drain_deliveries(&self, subscriber: EditorSubscriberId) -> Vec<EditorMessageDelivery> {
        let inbox = { self.lock().inbox_handle(subscriber) };
        inbox
            .map(|inbox| {
                inbox
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .drain()
            })
            .unwrap_or_default()
    }

    pub fn inbox_stats(&self, subscriber: EditorSubscriberId) -> Option<EditorMessageInboxStats> {
        let snapshot = { self.lock().inbox_stats_snapshot(subscriber) };
        snapshot.map(|(inbox, sequence)| {
            inbox
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .stats(sequence)
        })
    }

    pub fn mark_message_dirty(&self, message: &EditorMessage) {
        self.lock().mark_message_dirty(message);
    }

    pub fn mark_view_dirty(&self, view: ViewInstanceId, mask: EditorViewInvalidationMask) {
        self.lock().mark_view_dirty(view, mask);
    }

    /// Marks an existing view dirty while preserving its borrowed identity through the bus.
    pub fn mark_view_dirty_ref(&self, view: &ViewInstanceId, mask: EditorViewInvalidationMask) {
        self.lock().mark_view_dirty_ref(view, mask);
    }

    /// Merges an already-projected dirty set through one bus lock acquisition.
    pub fn mark_view_dirty_set(&self, dirty: &ViewDirtySet) {
        self.lock().mark_view_dirty_set(dirty);
    }

    pub fn dirty_set(&self) -> ViewDirtySet {
        self.lock().dirty_set().clone()
    }

    pub fn drain_dirty(&self) -> ViewDirtySet {
        self.lock().drain_dirty()
    }

    pub fn push_editor_ui_patch(&self, view: ViewInstanceId, patch: UiReflectionNodePatch) {
        self.lock().push_editor_ui_patch(view, patch);
    }

    pub fn push_editor_ui_barrier(
        &self,
        kind: EditorUiDeltaBarrierKind,
        sequence: EditorEventSequence,
    ) {
        self.lock().push_editor_ui_barrier(kind, sequence);
    }

    /// 在一次总线锁内同时取走脏集合与增量批次，供宿主将本轮失效和补丁作为同一刷新输入。
    pub fn drain_view_updates(&self) -> (ViewDirtySet, EditorUiDeltaBatch) {
        self.lock().drain_view_updates()
    }

    fn lock(&self) -> MutexGuard<'_, EditorMessageBus> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn finish_dispatch(&self, plan: EditorMessageDispatchPlan) -> EditorMessageDispatchReport {
        let enqueue = plan.dispatch();
        let report = plan.into_report(enqueue);
        if !report.delivered().is_empty() {
            self.lock().mark_message_dirty(plan.message());
        }
        report
    }
}
