use serde_json::json;

use crate::core::editor_event::{
    EditorEventListenerControlRequest, EditorEventListenerControlResponse, listener_deliveries,
    listener_descriptors, listener_status,
};

use super::EditorEventService;

impl EditorEventService {
    pub fn handle_listener_control_request(
        &self,
        request: EditorEventListenerControlRequest,
    ) -> EditorEventListenerControlResponse {
        match request {
            EditorEventListenerControlRequest::Register {
                listener_id,
                display_name,
            } => match self
                .lock_listeners()
                .register(listener_id.clone(), display_name)
            {
                Ok(()) => EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                })),
                Err(error) => EditorEventListenerControlResponse::failure(error),
            },
            EditorEventListenerControlRequest::Unregister { listener_id } => {
                match self.lock_listeners().unregister(&listener_id) {
                    Ok(()) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::SetEnabled {
                listener_id,
                enabled,
            } => match self.lock_listeners().set_enabled(&listener_id, enabled) {
                Ok(()) => EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                    "enabled": enabled,
                })),
                Err(error) => EditorEventListenerControlResponse::failure(error),
            },
            EditorEventListenerControlRequest::SetFilter {
                listener_id,
                filter,
            } => match self.lock_listeners().set_filter(&listener_id, filter) {
                Ok(()) => EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                })),
                Err(error) => EditorEventListenerControlResponse::failure(error),
            },
            EditorEventListenerControlRequest::ClearFilter { listener_id } => {
                match self.lock_listeners().clear_filter(&listener_id) {
                    Ok(()) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::ListListeners => {
                let listeners = self.lock_listeners().listeners();
                EditorEventListenerControlResponse::success(json!({
                    "listeners": listener_descriptors(&listeners),
                }))
            }
            EditorEventListenerControlRequest::QueryListenerStatus { listener_id } => {
                match self.lock_listeners().status_for(&listener_id) {
                    Ok(status) => {
                        EditorEventListenerControlResponse::success(listener_status(&status))
                    }
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::QueryDeliveries { listener_id } => {
                match self.lock_listeners().deliveries_for(&listener_id) {
                    Ok(deliveries) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                        "deliveries": listener_deliveries(&deliveries),
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::QueryDeliveriesSince {
                listener_id,
                after_sequence,
            } => match self
                .lock_listeners()
                .deliveries_after_sequence(&listener_id, after_sequence)
            {
                Ok(deliveries) => EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                    "after_sequence": after_sequence,
                    "deliveries": listener_deliveries(&deliveries),
                })),
                Err(error) => EditorEventListenerControlResponse::failure(error),
            },
            EditorEventListenerControlRequest::AckDeliveriesThrough {
                listener_id,
                sequence,
            } => match self
                .lock_listeners()
                .acknowledge_through(&listener_id, sequence)
            {
                Ok(removed) => EditorEventListenerControlResponse::success(json!({
                    "listener_id": listener_id,
                    "sequence": sequence,
                    "removed": removed,
                })),
                Err(error) => EditorEventListenerControlResponse::failure(error),
            },
        }
    }
}
