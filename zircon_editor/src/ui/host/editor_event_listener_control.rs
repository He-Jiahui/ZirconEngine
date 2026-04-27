use crate::core::editor_event::{
    listener_deliveries, listener_descriptors, EditorEventListenerControlRequest,
    EditorEventListenerControlResponse, EditorEventRuntime,
};
use serde_json::json;

impl EditorEventRuntime {
    pub fn handle_event_listener_control_request(
        &self,
        request: EditorEventListenerControlRequest,
    ) -> EditorEventListenerControlResponse {
        match request {
            EditorEventListenerControlRequest::Register {
                listener_id,
                display_name,
            } => {
                let mut inner = self.inner.lock().unwrap();
                match inner
                    .event_listeners
                    .register(listener_id.clone(), display_name)
                {
                    Ok(()) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::Unregister { listener_id } => {
                let mut inner = self.inner.lock().unwrap();
                match inner.event_listeners.unregister(&listener_id) {
                    Ok(()) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::SetEnabled {
                listener_id,
                enabled,
            } => {
                let mut inner = self.inner.lock().unwrap();
                match inner.event_listeners.set_enabled(&listener_id, enabled) {
                    Ok(()) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                        "enabled": enabled,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::SetFilter {
                listener_id,
                filter,
            } => {
                let mut inner = self.inner.lock().unwrap();
                match inner.event_listeners.set_filter(&listener_id, filter) {
                    Ok(()) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::ClearFilter { listener_id } => {
                let mut inner = self.inner.lock().unwrap();
                match inner.event_listeners.clear_filter(&listener_id) {
                    Ok(()) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::ListListeners => {
                let inner = self.inner.lock().unwrap();
                EditorEventListenerControlResponse::success(json!({
                    "listeners": listener_descriptors(inner.event_listeners.listeners()),
                }))
            }
            EditorEventListenerControlRequest::QueryDeliveries { listener_id } => {
                let inner = self.inner.lock().unwrap();
                let deliveries = inner.event_listeners.deliveries_for(&listener_id);
                EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                    "deliveries": listener_deliveries(&deliveries),
                }))
            }
            EditorEventListenerControlRequest::QueryDeliveriesSince {
                listener_id,
                after_sequence,
            } => {
                let inner = self.inner.lock().unwrap();
                let deliveries = inner
                    .event_listeners
                    .deliveries_after(&listener_id, after_sequence);
                EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                    "after_sequence": after_sequence,
                    "deliveries": listener_deliveries(&deliveries),
                }))
            }
            EditorEventListenerControlRequest::AckDeliveriesThrough {
                listener_id,
                sequence,
            } => {
                let mut inner = self.inner.lock().unwrap();
                let removed = inner
                    .event_listeners
                    .acknowledge_through(&listener_id, sequence);
                EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                    "sequence": sequence,
                    "removed": removed,
                }))
            }
        }
    }
}
