use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::{
    binding::UiEventKind,
    component::{
        UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiDragMetrics,
        UiValue,
    },
    event_ui::{UiNodeId, UiTreeId},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateActionInvocation {
    pub route: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub payload: BTreeMap<String, UiValue>,
}

impl UiTemplateActionInvocation {
    pub fn new(route: impl Into<String>, payload: BTreeMap<String, UiValue>) -> Self {
        Self {
            route: route.into(),
            payload,
        }
    }
}

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drag: Option<UiDragMetrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template_action: Option<UiTemplateActionInvocation>,
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
            drag: None,
            template_action: None,
        }
    }

    pub fn with_drag_metrics(mut self, drag: UiDragMetrics) -> Self {
        self.drag = Some(drag);
        self
    }

    pub fn with_template_action(mut self, template_action: UiTemplateActionInvocation) -> Self {
        self.template_action = Some(template_action);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_action_round_trips_with_typed_object_payload() {
        let event = UiPointerComponentEvent::new(
            &UiTreeId::new("test.template.action"),
            UiNodeId::new(7),
            "BakeSelected",
            "BakeSelected/Click",
            UiEventKind::Click,
            UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            },
            UiPointerComponentEventReason::DefaultClick,
        )
        .with_template_action(UiTemplateActionInvocation::new(
            "navigation.bake.surface",
            BTreeMap::from([
                ("surface_entity".to_string(), UiValue::Int(73)),
                ("force_full_rebuild".to_string(), UiValue::Bool(true)),
            ]),
        ));

        let encoded = serde_json::to_value(&event).expect("pointer event should serialize");
        assert_eq!(
            encoded["template_action"]["payload"]["surface_entity"],
            serde_json::json!({ "Int": 73 })
        );
        assert_eq!(
            encoded["template_action"]["payload"]["force_full_rebuild"],
            serde_json::json!({ "Bool": true })
        );

        let restored: UiPointerComponentEvent =
            serde_json::from_value(encoded).expect("pointer event should deserialize");
        assert_eq!(restored.template_action, event.template_action);
    }

    #[test]
    fn template_action_is_backward_compatible_when_absent() {
        let event = UiPointerComponentEvent::new(
            &UiTreeId::new("test.template.action.legacy"),
            UiNodeId::new(8),
            "LegacyButton",
            "LegacyButton/Click",
            UiEventKind::Click,
            UiComponentEvent::Commit {
                property: "activated".to_string(),
                value: UiValue::Bool(true),
            },
            UiPointerComponentEventReason::DefaultClick,
        );

        let encoded = serde_json::to_value(&event).expect("legacy pointer event should serialize");
        assert!(encoded.get("template_action").is_none());
        let restored: UiPointerComponentEvent =
            serde_json::from_value(encoded).expect("legacy pointer event should deserialize");
        assert_eq!(restored.template_action, None);
    }
}
