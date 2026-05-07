use serde::{Deserialize, Serialize};

use crate::ui::{
    binding::UiEventKind,
    component::{UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope},
    event_ui::{UiNodeId, UiTreeId},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UiPointerComponentEventReason {
    #[default]
    DirectBinding,
    DefaultClick,
    DefaultDoubleClick,
    DefaultClickRejected,
    HoverEnter,
    HoverLeave,
    PressBegin,
    PressEnd,
    FocusGained,
    FocusLost,
    ScrollFallback,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerComponentEvent {
    pub node_id: UiNodeId,
    pub binding_id: String,
    pub event_kind: UiEventKind,
    pub reason: UiPointerComponentEventReason,
    pub envelope: UiComponentEventEnvelope,
}

impl UiPointerComponentEvent {
    pub fn new(
        tree_id: &UiTreeId,
        node_id: UiNodeId,
        control_id: impl Into<String>,
        binding_id: impl Into<String>,
        event_kind: UiEventKind,
        event: UiComponentEvent,
        reason: UiPointerComponentEventReason,
    ) -> Self {
        let control_id = control_id.into();
        Self {
            node_id,
            binding_id: binding_id.into(),
            event_kind,
            reason,
            envelope: UiComponentEventEnvelope::new(
                tree_id.0.clone(),
                control_id.as_str(),
                UiComponentBindingTarget::showcase(control_id.as_str()),
                event,
            ),
        }
    }
}
