use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

use super::UiNavigationEventKind;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiNavigationRoute {
    pub kind: UiNavigationEventKind,
    pub target: Option<UiNodeId>,
    pub bubbled: Vec<UiNodeId>,
    pub fallback_to_root: bool,
    pub root_targets: Vec<UiNodeId>,
}
