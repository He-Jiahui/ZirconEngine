use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusVisibleReason {
    #[default]
    Initial,
    KeyboardNavigation,
    PointerInteraction,
    Programmatic,
    DisabledOrHidden,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusVisible {
    pub visible: bool,
    pub reason: UiFocusVisibleReason,
}

impl UiFocusVisible {
    pub const fn visible(reason: UiFocusVisibleReason) -> Self {
        Self {
            visible: true,
            reason,
        }
    }

    pub const fn hidden(reason: UiFocusVisibleReason) -> Self {
        Self {
            visible: false,
            reason,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiInputFocus {
    pub focused: Option<UiNodeId>,
    pub previous: Option<UiNodeId>,
    pub pending_autofocus: Option<UiNodeId>,
    pub focus_visible: UiFocusVisible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusChangeReason {
    #[default]
    Input,
    Navigation,
    Programmatic,
    Autofocus,
    Clear,
    Disabled,
    Hidden,
    Despawned,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusChangeEvent {
    pub previous: Option<UiNodeId>,
    pub current: Option<UiNodeId>,
    pub reason: UiFocusChangeReason,
    pub visible: UiFocusVisible,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiFocusedInputKind {
    #[default]
    Keyboard,
    Text,
    Ime,
    Navigation,
    Pointer,
    AccessibilityAction,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusedInput {
    pub focused: UiNodeId,
    pub kind: UiFocusedInputKind,
    pub route: Vec<UiNodeId>,
    pub handled_by: Option<UiNodeId>,
    pub accepted: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiFocusContract {
    pub focusable: bool,
    pub autofocus: bool,
    pub restore_on_close: bool,
    pub focus_visible: Option<UiFocusVisible>,
}
