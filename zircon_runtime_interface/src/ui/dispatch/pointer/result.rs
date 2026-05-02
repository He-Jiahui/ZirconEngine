use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiPointerRoute;

use super::UiPointerDispatchInvocation;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPointerDispatchResult {
    pub route: UiPointerRoute,
    pub invocations: Vec<UiPointerDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub blocked_by: Option<UiNodeId>,
    pub passthrough: Vec<UiNodeId>,
    pub captured_by: Option<UiNodeId>,
}

impl UiPointerDispatchResult {
    pub fn new(route: UiPointerRoute) -> Self {
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            blocked_by: None,
            passthrough: Vec::new(),
            captured_by: None,
        }
    }
}
