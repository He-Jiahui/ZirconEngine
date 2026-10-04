use serde_json::{json, Value};

use super::{
    EditorEventListenerDelivery, EditorEventListenerDescriptor, EditorEventListenerStatus,
};

#[cfg(test)]
#[path = "projection/tests/capacity_tests.rs"]
mod capacity_tests;

pub(crate) fn listener_descriptors(listeners: &[EditorEventListenerDescriptor]) -> Vec<Value> {
    let mut projected = Vec::with_capacity(listeners.len());
    projected.extend(listeners.iter().map(listener_descriptor));
    projected
}

pub(crate) fn listener_status(status: &EditorEventListenerStatus) -> Value {
    super::types::status_json(status)
}

pub(super) fn listener_descriptor(listener: &EditorEventListenerDescriptor) -> Value {
    json!({
        "listener_id": listener.listener_id,
        "display_name": listener.display_name,
        "enabled": listener.enabled,
        "filter": listener.filter,
    })
}

pub(crate) fn listener_deliveries(deliveries: &[EditorEventListenerDelivery]) -> Vec<Value> {
    let mut projected = Vec::with_capacity(deliveries.len());
    projected.extend(deliveries.iter().map(|delivery| {
        json!({
            "listener_id": delivery.listener_id,
            "delivery_cursor": delivery.delivery_cursor,
            "event_id": delivery.event_id,
            "sequence": delivery.sequence,
            "source": delivery.source,
            "operation_id": delivery.operation_id,
            "operation_display_name": delivery.operation_display_name,
            "operation_arguments": delivery.operation_arguments,
            "operation_group": delivery.operation_group,
            "result": delivery.result,
        })
    }));
    projected
}
