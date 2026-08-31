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
