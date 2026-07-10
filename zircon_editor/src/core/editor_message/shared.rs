use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::editor_event::ViewInstanceId;

use super::{
    EditorMessage, EditorMessageBus, EditorMessageBusError, EditorMessageDelivery,
    EditorMessageDispatchReport, EditorMessageResponse, EditorRequestHandler, EditorSubscriberId,
    EditorTopic, EditorViewInvalidationMask, ViewDirtySet,
};

/// Thread-safe boundary for editor messaging; handlers run outside the bus lock.
#[derive(Clone, Debug, Default)]
pub struct SharedEditorMessageBus {
    inner: Arc<Mutex<EditorMessageBus>>,
}

impl SharedEditorMessageBus {
    pub fn register_subscriber(
        &self,
        topics: impl IntoIterator<Item = EditorTopic>,
    ) -> EditorSubscriberId {
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
        self.lock().publish(topic, message)
    }

    pub fn broadcast(
        &self,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> EditorMessageDispatchReport {
        self.lock().broadcast(topic, message)
    }

    pub fn request(
        &self,
        target: EditorSubscriberId,
        topic: EditorTopic,
        message: EditorMessage,
        handler: &mut impl EditorRequestHandler,
    ) -> Result<EditorMessageResponse, EditorMessageBusError> {
        let request = self.lock().begin_request(target, topic, message)?;
        let response = handler.handle_editor_request(&request);
        self.lock().complete_request(target, &response)?;
        Ok(response)
    }

    pub fn deliveries_for(&self, subscriber: EditorSubscriberId) -> Vec<EditorMessageDelivery> {
        self.lock().deliveries_for(subscriber).to_vec()
    }

    pub fn drain_deliveries(&self, subscriber: EditorSubscriberId) -> Vec<EditorMessageDelivery> {
        self.lock().drain_deliveries(subscriber)
    }

    pub fn mark_message_dirty(&self, message: &EditorMessage) {
        self.lock().mark_message_dirty(message);
    }

    pub fn mark_view_dirty(&self, view: ViewInstanceId, mask: EditorViewInvalidationMask) {
        self.lock().mark_view_dirty(view, mask);
    }

    pub fn dirty_set(&self) -> ViewDirtySet {
        self.lock().dirty_set().clone()
    }

    pub fn drain_dirty(&self) -> ViewDirtySet {
        self.lock().drain_dirty()
    }

    fn lock(&self) -> MutexGuard<'_, EditorMessageBus> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
