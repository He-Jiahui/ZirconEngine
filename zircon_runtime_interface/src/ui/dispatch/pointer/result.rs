use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiPointerRoute;

use super::{UiPointerComponentEvent, UiPointerDispatchInvocation};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiPointerDispatchDiagnostics {
    pub pointer_routed: bool,
    pub ignored_same_target_hover: bool,
    pub hover_entered: usize,
    pub hover_left: usize,
    pub focus_changed: bool,
    pub capture_released: bool,
    pub click_target_resolved: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchResult {
    pub route: UiPointerRoute,
    pub invocations: Vec<UiPointerDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub blocked_by: Option<UiNodeId>,
    pub passthrough: Vec<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
    #[serde(default)]
    pub diagnostics: UiPointerDispatchDiagnostics,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_events: Vec<UiPointerComponentEvent>,
}

impl UiPointerDispatchResult {
    pub fn new(route: UiPointerRoute) -> Self {
        let diagnostics = UiPointerDispatchDiagnostics {
            pointer_routed: route.target.is_some() || !route.root_targets.is_empty(),
            ignored_same_target_hover: route.activation_phase
                == crate::ui::surface::UiPointerActivationPhase::Hover
                && route.entered.is_empty()
                && route.left.is_empty(),
            hover_entered: route.entered.len(),
            hover_left: route.left.len(),
            click_target_resolved: route.click_target.is_some(),
            ..UiPointerDispatchDiagnostics::default()
        };
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            blocked_by: None,
            passthrough: Vec::new(),
            captured_by: None,
            diagnostics,
            component_events: Vec::new(),
        }
    }
}
