use serde_json::{json, Value};

use crate::core::editor_event::{
    listener_descriptors, listener_status, EditorEventListenerControlRequest,
    EditorEventListenerControlResponse, SharedEditorEventRecord,
};

use super::EditorEventService;

impl EditorEventService {
    /// 处理监听器登记、筛选和分页确认的控制入口；分页只是读取保留队列，调用方确认游标后才移除已处理投递。
    /// 监听器句柄在控制锁外查询和投影，配置变化不会使已取得的在途句柄失效。
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
                            .iter()
                            .map(|record| {
                                listener_delivery_json(
                                    &listener_id,
                                    record.delivery_cursor,
                                    record.payload.as_ref(),
                                )
                            })
                            .collect::<Vec<_>>();
                        EditorEventListenerControlResponse::success(json!({
                            "listener_id": listener_id,
                            "after_delivery_cursor": after_delivery_cursor,
                            "max_deliveries": max_deliveries,
                            "deliveries": deliveries,
                            "next_delivery_cursor": next_delivery_cursor,
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

// 控制接口只投影观察所需的结果与操作关联字段；持有共享记录即可生成回复，避免序列化期间占用监听器锁。
fn listener_delivery_json(
    listener_id: &str,
    delivery_cursor: u64,
    payload: &SharedEditorEventRecord,
) -> Value {
    let record = payload.record();
    json!({
        "listener_id": listener_id,
        "delivery_cursor": delivery_cursor,
        "event_id": record.event_id.0,
        "sequence": record.sequence.0,
        "source": record.source,
        "operation_id": record.operation_id,
        "operation_display_name": record.operation_display_name,
        "operation_arguments": record.operation_arguments,
        "operation_group": record.operation_group,
        "result": record.result,
    })
}

#[cfg(test)]
#[path = "tests/listener_control.rs"]
mod tests;
