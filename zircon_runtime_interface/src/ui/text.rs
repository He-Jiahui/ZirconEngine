use serde::{Deserialize, Serialize};

use crate::ui::event_ui::UiNodeId;
use crate::ui::surface::{UiEditableTextState, UiTextEditAction};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiTextEditSource {
    #[default]
    Keyboard,
    Pointer,
    Ime,
    Clipboard,
    Accessibility,
    Programmatic,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiTextEdit {
    pub node_id: UiNodeId,
    pub source: UiTextEditSource,
    pub action: UiTextEditAction,
    pub before: UiEditableTextState,
    pub after: UiEditableTextState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiTextCursorStyle {
    pub width: f32,
    pub color: Option<String>,
    pub blink_period_millis: Option<u64>,
    pub visible: bool,
}

impl Default for UiTextCursorStyle {
    fn default() -> Self {
        Self {
            width: 1.0,
            color: None,
            blink_period_millis: None,
            visible: true,
        }
    }
}
