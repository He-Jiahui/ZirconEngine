use serde_json::json;

use crate::core::editor_event::{
    listener_deliveries, listener_descriptors, listener_status, EditorEventListenerControlRequest,
    EditorEventListenerControlResponse, EditorEventListenerDelivery,
    EditorEventListenerDeliveryPage,
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
                let listener = { self.lock_listeners().listener_handle(&listener_id) };
                match listener {
                    Ok(listener) => {
                        let status = listener.status();
                        EditorEventListenerControlResponse::success(listener_status(&status))
                    }
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::QueryDeliveriesPage {
                listener_id,
                after_delivery_cursor,
                max_deliveries,
            } => {
                let listener = { self.lock_listeners().listener_handle(&listener_id) };
                let page = listener.and_then(|listener| {
                    listener
                        .delivery_records_page_after_cursor(after_delivery_cursor, max_deliveries)
                });
                match page {
                    Ok(page) => {
                        let next_delivery_cursor =
                            page.records.last().map(|record| record.delivery_cursor);
                        let deliveries = page
                            .records
                            .into_iter()
                            .map(|record| {
                                EditorEventListenerDelivery::from_shared(
                                    &listener_id,
                                    record.delivery_cursor,
                                    record.payload.as_ref(),
                                )
                            })
                            .collect::<Vec<_>>();
                        let page = EditorEventListenerDeliveryPage {
                            deliveries,
                            next_delivery_cursor,
                            has_more: page.has_more,
                        };
                        EditorEventListenerControlResponse::success(json!({
                            "listener_id": listener_id,
                        "after_delivery_cursor": after_delivery_cursor,
                            "max_deliveries": max_deliveries,
                            "deliveries": listener_deliveries(&page.deliveries),
                        "next_delivery_cursor": page.next_delivery_cursor,
                            "has_more": page.has_more,
                                }))
                    }
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
            EditorEventListenerControlRequest::AckDeliveriesThrough {
                listener_id,
                delivery_cursor,
            } => {
                let listener = { self.lock_listeners().listener_handle(&listener_id) };
                match listener
                    .map(|listener| listener.acknowledge_through_delivery_cursor(delivery_cursor))
                {
                    Ok(removed) => EditorEventListenerControlResponse::success(json!({
                        "listener_id": listener_id,
                        "delivery_cursor": delivery_cursor,
                        "removed": removed,
                    })),
                    Err(error) => EditorEventListenerControlResponse::failure(error),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn delivery_page_dto_projection_stays_outside_the_listener_lock_scope() {
        let source = include_str!("listener_control.rs");
        let page_body = source
            .split("EditorEventListenerControlRequest::QueryDeliveriesPage")
            .nth(1)
            .expect("delivery page control branch should remain available");
        let lock_scope_end = page_body
            .find("};\n                let page")
            .expect("listener handle must be captured before the page result");
        let projection = page_body
            .find("EditorEventListenerDelivery::from_shared")
            .expect("delivery DTO projection should remain explicit");
        assert!(projection > lock_scope_end);
    }
}
