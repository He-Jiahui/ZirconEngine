use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;
use crate::ui::event_ui::UiNodeId;
use crate::ui::text::UiTextEdit;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidgetEventSource {
    #[default]
    Programmatic,
    Pointer,
    Keyboard,
    Navigation,
    TextInput,
    Accessibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidgetEventKind {
    Activate,
    ValueChange,
    TextEditChange,
    OpenChanged,
    SelectionChanged,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum UiWidgetEvent {
    Activate {
        target: UiNodeId,
        source: UiWidgetEventSource,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action_id: Option<String>,
    },
    ValueChange {
        target: UiNodeId,
        property: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        previous: Option<UiValue>,
        value: UiValue,
        source: UiWidgetEventSource,
    },
    TextEditChange {
        edit: UiTextEdit,
    },
    OpenChanged {
        target: UiNodeId,
        open: bool,
        source: UiWidgetEventSource,
    },
    SelectionChanged {
        target: UiNodeId,
        selection: Vec<UiValue>,
        source: UiWidgetEventSource,
    },
}

impl UiWidgetEvent {
    pub const fn kind(&self) -> UiWidgetEventKind {
        match self {
            Self::Activate { .. } => UiWidgetEventKind::Activate,
            Self::ValueChange { .. } => UiWidgetEventKind::ValueChange,
            Self::TextEditChange { .. } => UiWidgetEventKind::TextEditChange,
            Self::OpenChanged { .. } => UiWidgetEventKind::OpenChanged,
            Self::SelectionChanged { .. } => UiWidgetEventKind::SelectionChanged,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWidgetContract {
    pub disabled: bool,
    pub checked: Option<bool>,
    pub value: Option<UiValue>,
    pub label_for: Option<String>,
    pub tooltip: Option<String>,
}
