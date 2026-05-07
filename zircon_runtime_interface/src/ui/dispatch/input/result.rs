use serde::{Deserialize, Serialize};

use crate::ui::component::UiComponentEvent;
use crate::ui::event_ui::UiNodeId;

use super::{
    UiDispatchEffect, UiDispatchReply, UiInputEvent, UiInputMethodRequest, UiPointerLockPolicy,
    UiPopupEffectKind, UiTooltipEffectKind,
};
use crate::ui::layout::UiPoint;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputDispatchDiagnostics {
    pub routed: bool,
    pub handled_phase: Option<String>,
    pub route_target: Option<UiNodeId>,
    pub blocked_by: Option<UiNodeId>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchAppliedEffect {
    pub effect_index: usize,
    pub effect: UiDispatchEffect,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchRejectedEffect {
    pub effect_index: usize,
    pub effect: UiDispatchEffect,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UiDispatchHostRequestKind {
    InputMethod(UiInputMethodRequest),
    PointerLock {
        target: UiNodeId,
        policy: UiPointerLockPolicy,
    },
    PointerUnlock {
        policy: UiPointerLockPolicy,
    },
    HighPrecisionPointer {
        target: UiNodeId,
        enabled: bool,
    },
    Popup {
        kind: UiPopupEffectKind,
        popup_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        anchor: Option<UiPoint>,
    },
    Tooltip {
        kind: UiTooltipEffectKind,
        tooltip_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiDispatchHostRequest {
    pub effect_index: usize,
    pub request: UiDispatchHostRequestKind,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiComponentEventReport {
    pub target: UiNodeId,
    pub event: UiComponentEvent,
    pub delivered: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiInputDispatchResult {
    pub event: UiInputEvent,
    pub reply: UiDispatchReply,
    #[serde(default)]
    pub diagnostics: UiInputDispatchDiagnostics,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applied_effects: Vec<UiDispatchAppliedEffect>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_effects: Vec<UiDispatchRejectedEffect>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub host_requests: Vec<UiDispatchHostRequest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_events: Vec<UiComponentEventReport>,
}

impl UiInputDispatchResult {
    pub fn new(event: UiInputEvent, reply: UiDispatchReply) -> Self {
        Self {
            event,
            reply,
            diagnostics: UiInputDispatchDiagnostics::default(),
            applied_effects: Vec::new(),
            rejected_effects: Vec::new(),
            host_requests: Vec::new(),
            component_events: Vec::new(),
        }
    }
}
