use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::super::{
    EditorEventResult, EditorEventRetentionBudgetsSnapshot, EditorEventRetentionDiagnostics,
    EditorEventSource, SharedEditorEventRecord,
};
use super::EditorEventListenerFilter;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventListenerDescriptor {
    pub listener_id: String,
    pub display_name: String,
    pub enabled: bool,
    pub filter: Option<EditorEventListenerFilter>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorEventListenerDelivery {
    pub listener_id: String,
    pub event_id: u64,
    pub sequence: u64,
    pub source: EditorEventSource,
    pub operation_id: Option<String>,
    pub operation_display_name: Option<String>,
    pub operation_arguments: Option<Value>,
    pub operation_group: Option<String>,
    pub result: EditorEventResult,
}

impl EditorEventListenerDelivery {
    pub(super) fn from_shared(listener_id: &str, payload: &SharedEditorEventRecord) -> Self {
        let record = payload.record();
        Self {
            listener_id: listener_id.to_string(),
            event_id: record.event_id.0,
            sequence: record.sequence.0,
            source: record.source.clone(),
            operation_id: record.operation_id.clone(),
            operation_display_name: record.operation_display_name.clone(),
            operation_arguments: record.operation_arguments.clone(),
            operation_group: record.operation_group.clone(),
            result: record.result.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventListenerStatus {
    pub listener_id: String,
    pub descriptor: EditorEventListenerDescriptor,
    pub pending_delivery_count: usize,
    pub pending_delivery_bytes: usize,
    pub first_pending_sequence: Option<u64>,
    pub last_pending_sequence: Option<u64>,
    pub dropped_delivery_count: u64,
    pub coalesced_delivery_count: u64,
    pub lagged_since_sequence: Option<u64>,
    pub last_dropped_sequence: Option<u64>,
    pub retention_budgets: EditorEventRetentionBudgetsSnapshot,
    pub retention: EditorEventRetentionDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorEventListenerControlRequest {
    Register {
        listener_id: String,
        display_name: String,
    },
    Unregister {
        listener_id: String,
    },
    SetEnabled {
        listener_id: String,
        enabled: bool,
    },
    SetFilter {
        listener_id: String,
        filter: EditorEventListenerFilter,
    },
    ClearFilter {
        listener_id: String,
    },
    ListListeners,
    QueryListenerStatus {
        listener_id: String,
    },
    QueryDeliveries {
        listener_id: String,
    },
    QueryDeliveriesSince {
        listener_id: String,
        after_sequence: u64,
    },
    AckDeliveriesThrough {
        listener_id: String,
        sequence: u64,
    },
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorEventListenerControlResponse {
    pub value: Value,
    pub error: Option<String>,
}

impl EditorEventListenerControlResponse {
    pub fn success(value: Value) -> Self {
        Self { value, error: None }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            value: Value::Null,
            error: Some(error.into()),
        }
    }
}

pub(super) fn status_json(status: &EditorEventListenerStatus) -> Value {
    json!({
        "listener_id": status.listener_id,
        "descriptor": super::projection::listener_descriptor(&status.descriptor),
        "pending_delivery_count": status.pending_delivery_count,
        "pending_delivery_bytes": status.pending_delivery_bytes,
        "first_pending_sequence": status.first_pending_sequence,
        "last_pending_sequence": status.last_pending_sequence,
        "dropped_delivery_count": status.dropped_delivery_count,
        "coalesced_delivery_count": status.coalesced_delivery_count,
        "lagged_since_sequence": status.lagged_since_sequence,
        "last_dropped_sequence": status.last_dropped_sequence,
        "retention_budgets": status.retention_budgets,
        "retention": status.retention,
    })
}
