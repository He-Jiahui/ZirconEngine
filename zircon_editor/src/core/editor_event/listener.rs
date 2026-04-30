use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::{EditorEventRecord, EditorEventResult, EditorEventSource};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventListenerFilter {
    #[serde(default)]
    pub operation_path_prefixes: Vec<String>,
    #[serde(default)]
    pub sources: Vec<EditorEventSource>,
    #[serde(default = "default_filter_includes_events")]
    pub include_successes: bool,
    #[serde(default = "default_filter_includes_events")]
    pub include_failures: bool,
}

impl Default for EditorEventListenerFilter {
    fn default() -> Self {
        Self {
            operation_path_prefixes: Vec::new(),
            sources: Vec::new(),
            include_successes: true,
            include_failures: true,
        }
    }
}

impl EditorEventListenerFilter {
    pub fn operation_prefix(prefix: impl Into<String>) -> Self {
        Self {
            operation_path_prefixes: vec![prefix.into()],
            ..Self::default()
        }
    }

    pub fn source(source: EditorEventSource) -> Self {
        Self {
            sources: vec![source],
            ..Self::default()
        }
    }

    pub fn with_sources<I>(mut self, sources: I) -> Self
    where
        I: IntoIterator<Item = EditorEventSource>,
    {
        self.sources = sources.into_iter().collect();
        self
    }

    pub fn failures_only(mut self) -> Self {
        self.include_successes = false;
        self.include_failures = true;
        self
    }

    pub fn successes_only(mut self) -> Self {
        self.include_successes = true;
        self.include_failures = false;
        self
    }

    fn accepts(&self, record: &EditorEventRecord) -> bool {
        if !self.operation_path_prefixes.is_empty() {
            let Some(operation_id) = record.operation_id.as_deref() else {
                return false;
            };
            if !self
                .operation_path_prefixes
                .iter()
                .any(|prefix| operation_id.starts_with(prefix))
            {
                return false;
            }
        }

        if !self.sources.is_empty() && !self.sources.contains(&record.source) {
            return false;
        }

        if record.result.error.is_some() {
            return self.include_failures;
        }
        self.include_successes
    }
}

fn default_filter_includes_events() -> bool {
    true
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
    pub source: EditorEventSource,
    pub operation_id: Option<String>,
    pub operation_display_name: Option<String>,
    pub operation_arguments: Option<Value>,
    pub operation_group: Option<String>,
    pub result: EditorEventResult,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorEventListenerStatus {
    pub listener_id: String,
    pub descriptor: EditorEventListenerDescriptor,
    pub pending_delivery_count: usize,
    pub first_pending_sequence: Option<u64>,
    pub last_pending_sequence: Option<u64>,
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

    pub fn status_for(&self, listener_id: &str) -> Result<EditorEventListenerStatus, String> {
        let descriptor = self
            .listeners
            .iter()
            .find(|listener| listener.listener_id == listener_id)
            .cloned()
            .ok_or_else(|| format!("editor event listener {listener_id} is not registered"))?;
        let mut pending_delivery_count = 0usize;
        let mut first_pending_sequence = None;
        let mut last_pending_sequence = None;
        for delivery in self
            .deliveries
            .iter()
            .filter(|delivery| delivery.listener_id == listener_id)
        {
            pending_delivery_count += 1;
            if first_pending_sequence.is_none() {
                first_pending_sequence = Some(delivery.sequence);
            }
            last_pending_sequence = Some(delivery.sequence);
        }

        Ok(EditorEventListenerStatus {
            listener_id: descriptor.listener_id.clone(),
            descriptor,
            pending_delivery_count,
            first_pending_sequence,
            last_pending_sequence,
        })
    }

    pub fn deliveries_for(
        &self,
        listener_id: &str,
    ) -> Result<Vec<EditorEventListenerDelivery>, String> {
        self.ensure_registered(listener_id)?;
        Ok(self
            .deliveries
            .iter()
            .filter(|delivery| delivery.listener_id == listener_id)
            .cloned()
            .collect())
    }

    pub fn deliveries_after(
        &self,
        listener_id: &str,
        after_sequence: u64,
    ) -> Result<Vec<EditorEventListenerDelivery>, String> {
        self.ensure_registered(listener_id)?;
        Ok(self
            .deliveries
            .iter()
            .filter(|delivery| delivery.listener_id == listener_id)
            .filter(|delivery| delivery.sequence > after_sequence)
            .cloned()
            .collect())
    }

    pub fn acknowledge_through(
        &mut self,
        listener_id: &str,
        sequence: u64,
    ) -> Result<usize, String> {
        self.ensure_registered(listener_id)?;
        let before = self.deliveries.len();
        self.deliveries
            .retain(|delivery| delivery.listener_id != listener_id || delivery.sequence > sequence);
        Ok(before - self.deliveries.len())
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
                source: record.source.clone(),
                operation_id: record.operation_id.clone(),
                operation_display_name: record.operation_display_name.clone(),
                operation_arguments: record.operation_arguments.clone(),
                operation_group: record.operation_group.clone(),
                result: record.result.clone(),
            });
        }
    }

    pub fn sync_record(&mut self, record: &EditorEventRecord) {
        for delivery in self
            .deliveries
            .iter_mut()
            .filter(|delivery| delivery.event_id == record.event_id.0)
        {
            delivery.source = record.source.clone();
            delivery.operation_id = record.operation_id.clone();
            delivery.operation_display_name = record.operation_display_name.clone();
            delivery.operation_arguments = record.operation_arguments.clone();
            delivery.operation_group = record.operation_group.clone();
            delivery.result = record.result.clone();
        }
    }

    fn ensure_registered(&self, listener_id: &str) -> Result<(), String> {
        if self
            .listeners
            .iter()
            .any(|listener| listener.listener_id == listener_id)
        {
            return Ok(());
        }
        Err(format!(
            "editor event listener {listener_id} is not registered"
        ))
    }
}

pub(crate) fn listener_descriptors(listeners: &[EditorEventListenerDescriptor]) -> Vec<Value> {
    listeners.iter().map(listener_descriptor).collect()
}

pub(crate) fn listener_status(status: &EditorEventListenerStatus) -> Value {
    json!({
        "listener_id": status.listener_id,
        "descriptor": listener_descriptor(&status.descriptor),
        "pending_delivery_count": status.pending_delivery_count,
        "first_pending_sequence": status.first_pending_sequence,
        "last_pending_sequence": status.last_pending_sequence,
    })
}

fn listener_descriptor(listener: &EditorEventListenerDescriptor) -> Value {
    json!({
        "listener_id": listener.listener_id,
        "display_name": listener.display_name,
        "enabled": listener.enabled,
        "filter": listener.filter,
    })
}

pub(crate) fn listener_deliveries(deliveries: &[EditorEventListenerDelivery]) -> Vec<Value> {
    deliveries
        .iter()
        .map(|delivery| {
            json!({
                "listener_id": delivery.listener_id,
                "event_id": delivery.event_id,
                "sequence": delivery.sequence,
                "source": delivery.source,
                "operation_id": delivery.operation_id,
                "operation_display_name": delivery.operation_display_name,
                "operation_arguments": delivery.operation_arguments,
                "operation_group": delivery.operation_group,
                "result": delivery.result,
            })
        })
        .collect()
}
