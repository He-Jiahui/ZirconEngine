use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationState {
    pub focus_visible: bool,
    pub navigation_root: Option<UiNodeId>,
}
