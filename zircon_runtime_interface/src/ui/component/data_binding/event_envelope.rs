use serde::{Deserialize, Serialize};

use super::UiComponentBindingTarget;
use crate::ui::component::{UiComponentEvent, UiComponentEventKind, UiDragSourceMetadata};

/// Wraps a typed component event with enough target metadata for host adapters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "UiComponentEventEnvelopeWire")]
pub struct UiComponentEventEnvelope {
    pub document_id: String,
    pub control_id: String,
    pub component_id: Option<String>,
    pub target: UiComponentBindingTarget,
    pub event_kind: UiComponentEventKind,
    pub event: UiComponentEvent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<UiDragSourceMetadata>,
}

#[derive(Deserialize)]
struct UiComponentEventEnvelopeWire {
    document_id: String,
    control_id: String,
    component_id: Option<String>,
    target: UiComponentBindingTarget,
    event_kind: UiComponentEventKind,
    event: UiComponentEvent,
    source: Option<UiDragSourceMetadata>,
}

impl TryFrom<UiComponentEventEnvelopeWire> for UiComponentEventEnvelope {
    type Error = String;

    fn try_from(value: UiComponentEventEnvelopeWire) -> Result<Self, Self::Error> {
        let actual_event_kind = value.event.kind();
        if value.event_kind != actual_event_kind {
            return Err(format!(
                "component event kind {:?} does not match typed event {:?}",
                value.event_kind, actual_event_kind
            ));
        }

        Ok(Self {
            document_id: value.document_id,
            control_id: value.control_id,
            component_id: value.component_id,
            target: value.target,
            event_kind: value.event_kind,
            event: value.event,
            source: value.source,
        })
    }
}

impl UiComponentEventEnvelope {
    pub fn new(
        document_id: impl Into<String>,
        control_id: impl Into<String>,
        target: UiComponentBindingTarget,
        event: UiComponentEvent,
    ) -> Self {
        let event_kind = event.kind();
        Self {
            document_id: document_id.into(),
            control_id: control_id.into(),
            component_id: None,
            target,
            event_kind,
            event,
            source: None,
        }
    }

    pub fn with_component_id(mut self, component_id: impl Into<String>) -> Self {
        self.component_id = Some(component_id.into());
        self
    }

    pub fn with_source(mut self, source: UiDragSourceMetadata) -> Self {
        self.source = Some(source);
        self
    }
}
