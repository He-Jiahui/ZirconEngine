use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiNavigationDispatchEffect {
    Unhandled,
    Handled,
    Focus(UiNodeId),
}

impl UiNavigationDispatchEffect {
    pub const fn handled() -> Self {
        Self::Handled
    }

    pub const fn focus(node_id: UiNodeId) -> Self {
        Self::Focus(node_id)
    }
}
