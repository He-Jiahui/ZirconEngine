use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::focus::{UiFocusChangeEvent, UiFocusVisible, UiFocusedInput};

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
