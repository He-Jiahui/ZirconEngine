use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::EditorEventRecord;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventListenerFilter {
    pub operation_path_prefixes: Vec<String>,
}

impl EditorEventListenerFilter {
    pub fn operation_prefix(prefix: impl Into<String>) -> Self {
        Self {
            operation_path_prefixes: vec![prefix.into()],
        }
    }

    fn accepts(&self, record: &EditorEventRecord) -> bool {
        if self.operation_path_prefixes.is_empty() {
            return true;
        }
        let Some(operation_id) = record.operation_id.as_deref() else {
            return false;
        };
        self.operation_path_prefixes
            .iter()
            .any(|prefix| operation_id.starts_with(prefix))
    }
}

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
    pub operation_id: Option<String>,
    pub operation_display_name: Option<String>,
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

#[derive(Clone, Debug, Default)]
pub struct EditorEventListenerRegistry {
    listeners: Vec<EditorEventListenerDescriptor>,
    deliveries: Vec<EditorEventListenerDelivery>,
}

impl EditorEventListenerRegistry {
    pub fn register(
        &mut self,
        listener_id: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Result<(), String> {
        let listener_id = listener_id.into();
        if self
            .listeners
            .iter()
            .any(|listener| listener.listener_id == listener_id)
        {
            return Err(format!(
                "editor event listener {listener_id} already registered"
            ));
        }
        self.listeners.push(EditorEventListenerDescriptor {
            listener_id,
            display_name: display_name.into(),
            enabled: true,
            filter: None,
        });
        Ok(())
    }

    pub fn unregister(&mut self, listener_id: &str) -> Result<(), String> {
        let Some(index) = self
            .listeners
            .iter()
            .position(|listener| listener.listener_id == listener_id)
        else {
            return Err(format!(
                "editor event listener {listener_id} is not registered"
            ));
        };
        self.listeners.remove(index);
        self.deliveries
            .retain(|delivery| delivery.listener_id != listener_id);
        Ok(())
    }

    pub fn set_enabled(&mut self, listener_id: &str, enabled: bool) -> Result<(), String> {
        let Some(listener) = self
            .listeners
            .iter_mut()
            .find(|listener| listener.listener_id == listener_id)
        else {
            return Err(format!(
                "editor event listener {listener_id} is not registered"
            ));
        };
        listener.enabled = enabled;
        Ok(())
    }

    pub fn set_filter(
        &mut self,
        listener_id: &str,
        filter: EditorEventListenerFilter,
    ) -> Result<(), String> {
        let Some(listener) = self
            .listeners
            .iter_mut()
            .find(|listener| listener.listener_id == listener_id)
        else {
            return Err(format!(
                "editor event listener {listener_id} is not registered"
            ));
        };
        listener.filter = Some(filter);
        Ok(())
    }

    pub fn clear_filter(&mut self, listener_id: &str) -> Result<(), String> {
        let Some(listener) = self
            .listeners
            .iter_mut()
            .find(|listener| listener.listener_id == listener_id)
        else {
            return Err(format!(
                "editor event listener {listener_id} is not registered"
            ));
        };
        listener.filter = None;
        Ok(())
    }

    pub fn listeners(&self) -> &[EditorEventListenerDescriptor] {
        &self.listeners
    }

    pub fn deliveries_for(&self, listener_id: &str) -> Vec<EditorEventListenerDelivery> {
        self.deliveries
            .iter()
            .filter(|delivery| delivery.listener_id == listener_id)
            .cloned()
            .collect()
    }

    pub fn deliveries_after(
        &self,
        listener_id: &str,
        after_sequence: u64,
    ) -> Vec<EditorEventListenerDelivery> {
        self.deliveries
            .iter()
            .filter(|delivery| delivery.listener_id == listener_id)
            .filter(|delivery| delivery.sequence > after_sequence)
            .cloned()
            .collect()
    }

    pub fn acknowledge_through(&mut self, listener_id: &str, sequence: u64) -> usize {
        let before = self.deliveries.len();
        self.deliveries
            .retain(|delivery| delivery.listener_id != listener_id || delivery.sequence > sequence);
        before - self.deliveries.len()
    }

    pub fn notify(&mut self, record: &EditorEventRecord) {
        for listener in self
            .listeners
            .iter()
            .filter(|listener| listener.enabled)
            .filter(|listener| {
                listener
                    .filter
                    .as_ref()
                    .map_or(true, |filter| filter.accepts(record))
            })
        {
            self.deliveries.push(EditorEventListenerDelivery {
                listener_id: listener.listener_id.clone(),
                event_id: record.event_id.0,
                sequence: record.sequence.0,
                operation_id: record.operation_id.clone(),
                operation_display_name: record.operation_display_name.clone(),
            });
        }
    }

    pub fn sync_record(&mut self, record: &EditorEventRecord) {
        for delivery in self
            .deliveries
            .iter_mut()
            .filter(|delivery| delivery.event_id == record.event_id.0)
        {
            delivery.operation_id = record.operation_id.clone();
            delivery.operation_display_name = record.operation_display_name.clone();
        }
    }
}

pub(crate) fn listener_descriptors(listeners: &[EditorEventListenerDescriptor]) -> Vec<Value> {
    listeners
        .iter()
        .map(|listener| {
            json!({
                "listener_id": listener.listener_id,
                "display_name": listener.display_name,
                "enabled": listener.enabled,
                "filter": listener.filter,
            })
        })
        .collect()
}

pub(crate) fn listener_deliveries(deliveries: &[EditorEventListenerDelivery]) -> Vec<Value> {
    deliveries
        .iter()
        .map(|delivery| {
            json!({
                "listener_id": delivery.listener_id,
                "event_id": delivery.event_id,
                "sequence": delivery.sequence,
                "operation_id": delivery.operation_id,
                "operation_display_name": delivery.operation_display_name,
            })
        })
        .collect()
}
