use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ui::template::UiCompiledBindingHandle;
use crate::ui::{
    binding::UiEventKind,
    component::{
        UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiDragMetrics,
        UiValue,
    },
    event_ui::{UiNodeId, UiTreeId},
};

/// 模板绑定产出的动作身份：声明的 action 与操作 route 分别编码，Editor 回调按种类分发到命令或操作。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiTemplateActionInvocation {
    target: UiTemplateActionTarget,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub payload: BTreeMap<String, UiValue>,
}

impl UiTemplateActionInvocation {
    /// Constructs a parameterized local or operation route.
    pub fn route(route: impl Into<String>, payload: BTreeMap<String, UiValue>) -> Self {
        Self {
            target: UiTemplateActionTarget::route(route),
            payload,
        }
    }

    pub fn action(action: impl Into<String>) -> Self {
        Self {
            target: UiTemplateActionTarget::action(action),
            payload: BTreeMap::new(),
        }
    }

    pub fn target_id(&self) -> &str {
        &self.target.id
    }

    pub fn is_action(&self) -> bool {
        self.target.kind == UiTemplateActionTargetKind::Action
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct UiTemplateActionTarget {
    kind: UiTemplateActionTargetKind,
    id: String,
}

impl UiTemplateActionTarget {
    pub fn action(id: impl Into<String>) -> Self {
        Self {
            kind: UiTemplateActionTargetKind::Action,
            id: id.into(),
        }
    }

    pub fn route(id: impl Into<String>) -> Self {
        Self {
            kind: UiTemplateActionTargetKind::Route,
            id: id.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum UiTemplateActionTargetKind {
    Action,
    Route,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_binding: Option<UiCompiledBindingHandle>,
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
            compiled_binding: None,
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

    pub fn with_compiled_binding(mut self, handle: UiCompiledBindingHandle) -> Self {
        self.compiled_binding = Some(handle);
        self
    }

    pub fn with_template_action(mut self, template_action: UiTemplateActionInvocation) -> Self {
        self.template_action = Some(template_action);
        self
    }
}

#[cfg(test)]
#[path = "tests/component_event.rs"]
mod tests;
