use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::focus::{UiFocusChangeEvent, UiFocusVisible, UiFocusedInput};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusPath {
    pub focused: Option<UiNodeId>,
    pub root_to_leaf: Vec<UiNodeId>,
    pub bubble_route: Vec<UiNodeId>,
}

impl UiFocusPath {
    pub fn with_route(
        focused: Option<UiNodeId>,
        root_to_leaf: Vec<UiNodeId>,
        bubble_route: Vec<UiNodeId>,
    ) -> Self {
        Self {
            focused,
            root_to_leaf,
            bubble_route,
        }
    }

    pub fn from_bubble_route(focused: Option<UiNodeId>, bubble_route: Vec<UiNodeId>) -> Self {
        let mut root_to_leaf = bubble_route.clone();
        root_to_leaf.reverse();
        Self::with_route(focused, root_to_leaf, bubble_route)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiModalFocusRestoreState {
    pub modal: UiNodeId,
    pub restore: Option<UiNodeId>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiFocusState {
    pub focused: Option<UiNodeId>,
    #[serde(default)]
    pub previous: Option<UiNodeId>,
    #[serde(default)]
    pub pending_autofocus: Option<UiNodeId>,
    #[serde(default)]
    pub focus_visible: UiFocusVisible,
    #[serde(default)]
    pub changes: Vec<UiFocusChangeEvent>,
    #[serde(default)]
    pub focused_inputs: Vec<UiFocusedInput>,
    #[serde(default)]
    pub modal_restore_stack: Vec<UiModalFocusRestoreState>,
    pub captured: Option<UiNodeId>,
    #[serde(default)]
    pub pressed: Option<UiNodeId>,
    pub hovered: Vec<UiNodeId>,
}
