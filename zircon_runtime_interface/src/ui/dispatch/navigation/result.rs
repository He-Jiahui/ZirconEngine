use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::UiNavigationRoute;

use super::UiNavigationDispatchInvocation;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationDispatchResult {
    pub route: UiNavigationRoute,
    pub invocations: Vec<UiNavigationDispatchInvocation>,
    pub handled_by: Option<UiNodeId>,
    pub focus_changed_to: Option<UiNodeId>,
}

impl UiNavigationDispatchResult {
    pub fn new(route: UiNavigationRoute) -> Self {
        Self {
            route,
            invocations: Vec::new(),
            handled_by: None,
            focus_changed_to: None,
        }
    }
}
