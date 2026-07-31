use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::core::editor_message::{
    EditorMessagePayload, EditorTopic, ModeMessage, ToolMessage, TransactionMessage,
};
use crate::core::jobs::JobEventKind;
use crate::core::tools::ToolLifecycleEvent;

use super::{EditorMessage, EditorMessageProtocol};
use crate::core::editor_message::retention::{
    editor_message_retention, EditorMessageCoalescingKey, EditorMessageRetention,
};

#[derive(Clone, Debug)]
pub struct EditorMessageDelivery {
    payload: Arc<EditorMessageDeliveryPayload>,
    sequence: u64,
    retained_bytes: usize,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct EditorMessageDeliveryPayload {
    protocol: EditorMessageProtocol,
    topic: EditorTopic,
    message: EditorMessage,
}

impl EditorMessageDelivery {
    pub fn new(
        protocol: EditorMessageProtocol,
        topic: EditorTopic,
        message: EditorMessage,
    ) -> Self {
        Self::with_sequence(protocol, topic, message, 0)
    }

    pub(in crate::core::editor_message) fn with_sequence(
        protocol: EditorMessageProtocol,
        topic: EditorTopic,
        message: EditorMessage,
        sequence: u64,
    ) -> Self {
        let retained_bytes = estimate_retained_bytes(&topic, &message);
        Self {
            payload: Arc::new(EditorMessageDeliveryPayload {
                protocol,
                topic,
                message,
            }),
            sequence,
            retained_bytes,
        }
    }

    pub fn protocol(&self) -> EditorMessageProtocol {
        self.payload.protocol
    }

    pub fn topic(&self) -> &EditorTopic {
        &self.payload.topic
    }

    pub fn message(&self) -> &EditorMessage {
        &self.payload.message
    }

    pub(crate) fn shares_payload_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.payload, &other.payload)
    }

    pub(in crate::core::editor_message) fn sequence(&self) -> u64 {
        self.sequence
    }

    pub(in crate::core::editor_message) fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(in crate::core::editor_message) fn retention(&self) -> EditorMessageRetention {
        editor_message_retention(self.protocol(), self.message())
    }

    pub(in crate::core::editor_message) fn coalescing_key(
        &self,
    ) -> Option<EditorMessageCoalescingKey> {
        match self.retention() {
            EditorMessageRetention::Latest(key) => Some(key),
            EditorMessageRetention::Lossless | EditorMessageRetention::Bounded => None,
        }
    }
}

impl PartialEq for EditorMessageDelivery {
    fn eq(&self, other: &Self) -> bool {
        self.payload == other.payload
    }
}

impl Serialize for EditorMessageDelivery {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.payload.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EditorMessageDelivery {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let payload = EditorMessageDeliveryPayload::deserialize(deserializer)?;
        let retained_bytes = estimate_retained_bytes(&payload.topic, &payload.message);
        Ok(Self {
            payload: Arc::new(payload),
            sequence: 0,
            retained_bytes,
        })
    }
}

fn estimate_retained_bytes(topic: &EditorTopic, message: &EditorMessage) -> usize {
    let payload_bytes = match message.payload() {
        EditorMessagePayload::Document(_) | EditorMessagePayload::Focus(_) => 0,
        EditorMessagePayload::Mode(ModeMessage::SceneModeChanged { mode }) => mode.as_str().len(),
        EditorMessagePayload::Mode(ModeMessage::PlayStateChanged { .. }) => 0,
        EditorMessagePayload::Transaction(
            TransactionMessage::Started { label, .. }
            | TransactionMessage::Canceled { label, .. }
            | TransactionMessage::Committed { label, .. }
            | TransactionMessage::Undone { label, .. }
            | TransactionMessage::Redone { label, .. },
        ) => label.len(),
        EditorMessagePayload::Transaction(TransactionMessage::HistoryTrimmed { .. }) => 0,
        EditorMessagePayload::SceneInspection(message) => {
            let entity_bytes = [
                message.added_entities(),
                message.changed_entities(),
                message.removed_entities(),
            ]
            .into_iter()
            .fold(0usize, |bytes, entities| {
                bytes.saturating_add(std::mem::size_of_val(entities))
            });
            let property_bytes = message
                .focused_fields()
                .changed_properties()
                .iter()
                .chain(message.focused_fields().removed_properties())
                .fold(0usize, |bytes, path| {
                    bytes
                        .saturating_add(path.component_type_path().len())
                        .saturating_add(path.field_name().len())
                });
            entity_bytes.saturating_add(property_bytes)
        }
        EditorMessagePayload::Tool(ToolMessage::Lifecycle(event)) => match event {
            ToolLifecycleEvent::Activated { tool, .. }
            | ToolLifecycleEvent::Deactivated { tool, .. }
            | ToolLifecycleEvent::Queued { tool, .. }
            | ToolLifecycleEvent::Withdrawn { tool, .. } => tool.as_str().len(),
            ToolLifecycleEvent::Denied { tool, holder, .. } => {
                tool.as_str().len().saturating_add(holder.as_str().len())
            }
            ToolLifecycleEvent::SetActivated {
                tool, resources, ..
            }
            | ToolLifecycleEvent::SetDeactivated {
                tool, resources, ..
            }
            | ToolLifecycleEvent::SetQueued {
                tool, resources, ..
            }
            | ToolLifecycleEvent::SetWithdrawn {
                tool, resources, ..
            }
            | ToolLifecycleEvent::SetDenied {
                tool, resources, ..
            } => tool.as_str().len().saturating_add(resources.len()),
        },
        EditorMessagePayload::Job(event) => {
            let detail_bytes = match event.kind() {
                JobEventKind::Progress { message, .. } | JobEventKind::Failed { message } => {
                    message.len()
                }
                JobEventKind::Started | JobEventKind::Completed | JobEventKind::Cancelled => 0,
            };
            event.label().len().saturating_add(detail_bytes)
        }
        EditorMessagePayload::Custom { schema_id, payload } => {
            schema_id.len().saturating_add(estimate_json_bytes(payload))
        }
    };
    let dirty_view_bytes = message
        .dirty()
        .map(|dirty| dirty.view().0.len())
        .unwrap_or_default();
    std::mem::size_of::<EditorMessageDeliveryPayload>()
        .saturating_add(topic.as_str().len())
        .saturating_add(payload_bytes)
        .saturating_add(dirty_view_bytes)
}

fn estimate_json_bytes(root: &serde_json::Value) -> usize {
    let mut retained_bytes = 0usize;
    let mut pending = vec![root];
    while let Some(value) = pending.pop() {
        retained_bytes = retained_bytes.saturating_add(std::mem::size_of::<serde_json::Value>());
        match value {
            serde_json::Value::String(value) => {
                retained_bytes = retained_bytes.saturating_add(value.len());
            }
            serde_json::Value::Array(values) => pending.extend(values),
            serde_json::Value::Object(values) => {
                for (key, value) in values {
                    retained_bytes = retained_bytes.saturating_add(key.len());
                    pending.push(value);
                }
            }
            serde_json::Value::Null | serde_json::Value::Bool(_) | serde_json::Value::Number(_) => {
            }
        }
    }
    retained_bytes
}
