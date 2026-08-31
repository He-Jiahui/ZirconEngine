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

    pub(in crate::core::editor_message) fn coalesce_latest_from(self, previous: &Self) -> Self {
        let composes_scene_inspection = matches!(
            (self.message().payload(), previous.message().payload()),
            (
                EditorMessagePayload::SceneInspection(_),
                EditorMessagePayload::SceneInspection(_)
            )
        );
        if !composes_scene_inspection {
            return self;
        }
        let message = self
            .message()
            .clone()
            .coalesce_latest_from(previous.message());
        Self::with_sequence(
            self.protocol(),
            self.topic().clone(),
            message,
            self.sequence(),
        )
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
            let hierarchy_bytes = std::mem::size_of_val(message.added_anchors())
                .saturating_add(std::mem::size_of_val(message.changed_anchors()))
                .saturating_add(std::mem::size_of_val(message.removed_entities()));
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
            let selection_bytes = std::mem::size_of_val(message.selection().added_entities())
                .saturating_add(std::mem::size_of_val(
                    message.selection().removed_entities(),
                ));
            hierarchy_bytes
                .saturating_add(property_bytes)
                .saturating_add(selection_bytes)
        }
        EditorMessagePayload::Tool(ToolMessage::Transition(batch)) => batch
            .events()
            .iter()
            .fold(std::mem::size_of_val(&batch.revision()), |bytes, event| {
                bytes.saturating_add(estimate_tool_lifecycle_event(event))
            }),
        EditorMessagePayload::Job(event) => {
            let detail_bytes = match event.kind() {
                JobEventKind::Progress { message, .. } | JobEventKind::Failed { message } => {
                    message.len()
                }
                JobEventKind::Started | JobEventKind::Completed | JobEventKind::Cancelled => 0,
            };
            event.label().len().saturating_add(detail_bytes)
        }
        EditorMessagePayload::JobJournalGap(gap) => std::mem::size_of_val(gap),
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

fn estimate_tool_lifecycle_event(event: &ToolLifecycleEvent) -> usize {
    match event {
        ToolLifecycleEvent::AuthorityStateChanged { .. } => {
            std::mem::size_of::<crate::core::tools::ToolAuthorityState>().saturating_mul(2)
        }
        ToolLifecycleEvent::OwnerGenerationRegistered { .. }
        | ToolLifecycleEvent::OwnerGenerationRevoked { .. } => {
            std::mem::size_of::<crate::core::tools::ToolOwnerGeneration>()
        }
        ToolLifecycleEvent::ResourceKindRegistered { registration } => {
            estimate_tool_resource_kind_registration(registration)
        }
        ToolLifecycleEvent::ResourceKindsRevoked { kinds, .. } => {
            std::mem::size_of::<crate::core::tools::ToolOwnerGeneration>().saturating_add(
                kinds
                    .iter()
                    .map(|kind| {
                        std::mem::size_of::<crate::core::tools::ToolResourceKindId>()
                            .saturating_add(kind.as_str().len())
                    })
                    .fold(0usize, usize::saturating_add),
            )
        }
        ToolLifecycleEvent::InputCapture { event } => estimate_tool_input_capture_event(event),
        ToolLifecycleEvent::Activated { lease } | ToolLifecycleEvent::Deactivated { lease } => {
            estimate_tool_lease(lease)
        }
        ToolLifecycleEvent::Queued { request, .. }
        | ToolLifecycleEvent::Withdrawn { request, .. } => estimate_tool_request(request),
        ToolLifecycleEvent::Denied {
            instance,
            resources,
            holder,
            ..
        } => instance
            .as_str()
            .len()
            .saturating_add(estimate_tool_resources(resources))
            .saturating_add(holder.as_ref().map(estimate_tool_lease).unwrap_or_default()),
    }
}

fn estimate_tool_resource_kind_registration(
    registration: &crate::core::tools::ToolResourceKindRegistration,
) -> usize {
    std::mem::size_of::<crate::core::tools::ToolResourceKindRegistration>()
        .saturating_add(registration.kind().as_str().len())
        .saturating_add(
            registration
                .supported_scopes()
                .len()
                .saturating_mul(std::mem::size_of::<crate::core::tools::ToolScopeKind>()),
        )
}

fn estimate_tool_input_capture_event(event: &crate::core::tools::ToolInputCaptureEvent) -> usize {
    use crate::core::tools::ToolInputCaptureEvent;

    match event {
        ToolInputCaptureEvent::Started { handle } | ToolInputCaptureEvent::Ended { handle, .. } => {
            estimate_tool_input_capture(handle)
        }
        ToolInputCaptureEvent::Denied {
            request, holder, ..
        } => estimate_tool_input_capture_request(request).saturating_add(
            holder
                .as_ref()
                .map(estimate_tool_input_capture)
                .unwrap_or_default(),
        ),
    }
}

fn estimate_tool_input_capture_request(
    request: &crate::core::tools::ToolInputCaptureRequest,
) -> usize {
    estimate_tool_input_capture_owner(request.owner())
        .saturating_add(estimate_tool_input_source(request.source()))
        .saturating_add(request.resource().estimated_retained_bytes())
        .saturating_add(std::mem::size_of::<
            crate::core::tools::ToolInputCapturePriority,
        >())
}

fn estimate_tool_input_capture(capture: &crate::core::tools::ToolInputCaptureHandle) -> usize {
    std::mem::size_of::<crate::core::tools::ToolInputCaptureId>()
        .saturating_add(estimate_tool_input_capture_owner(capture.owner()))
        .saturating_add(estimate_tool_input_source(capture.source()))
        .saturating_add(capture.resource().estimated_retained_bytes())
        .saturating_add(std::mem::size_of::<
            crate::core::tools::ToolInputCapturePriority,
        >())
}

fn estimate_tool_input_capture_owner(owner: &crate::core::tools::ToolInputCaptureOwner) -> usize {
    std::mem::size_of::<crate::core::tools::ToolLeaseId>()
        .saturating_add(std::mem::size_of::<crate::core::tools::ToolOwnerGeneration>())
        .saturating_add(owner.instance().as_str().len())
}

fn estimate_tool_input_source(source: &crate::core::tools::ToolInputSource) -> usize {
    let scope = source.scope();
    scope
        .window_id()
        .0
        .len()
        .saturating_add(scope.surface_id().0.len())
        .saturating_add(std::mem::size_of_val(source))
}

fn estimate_tool_request(request: &crate::core::tools::ToolRequestHandle) -> usize {
    std::mem::size_of::<crate::core::tools::ToolRequestId>()
        .saturating_add(std::mem::size_of::<crate::core::tools::ToolLeaseId>())
        .saturating_add(request.instance().as_str().len())
        .saturating_add(estimate_tool_resources(request.resources()))
}

fn estimate_tool_lease(lease: &crate::core::tools::ToolLeaseHandle) -> usize {
    std::mem::size_of::<crate::core::tools::ToolLeaseId>()
        .saturating_add(std::mem::size_of::<crate::core::tools::ToolRequestId>())
        .saturating_add(lease.instance().as_str().len())
        .saturating_add(estimate_tool_resources(lease.resources()))
}

fn estimate_tool_resources(resources: &crate::core::tools::ToolResourceSet) -> usize {
    resources
        .as_slice()
        .iter()
        .map(crate::core::tools::ToolResourceKey::estimated_retained_bytes)
        .fold(0usize, usize::saturating_add)
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
