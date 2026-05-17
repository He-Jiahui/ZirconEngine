use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiAxis;
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiWidgetBehavior {
    #[default]
    Auto,
    Passive,
    Button,
    Toggle,
    Disclosure,
    Popup,
    RadioGroup,
    Radio,
    Range,
    Scrollbar,
    ScrollbarThumb,
    TextInput,
    MenuItem,
}

impl UiWidgetBehavior {
    pub fn infer_from_component(component: &str) -> Self {
        match component {
            "Button" | "IconButton" | "MaterialButton" => Self::Button,
            "Toggle" | "Checkbox" | "CheckBox" | "Switch" | "ToggleButton" => Self::Toggle,
            "RadioGroup" | "ButtonGroup" => Self::RadioGroup,
            "Radio" | "RadioButton" => Self::Radio,
            "Group" | "Foldout" | "InspectorSection" | "TreeRow" | "TreeView" => Self::Disclosure,
            "Dropdown" | "ComboBox" | "EnumField" | "FlagsField" | "SearchSelect"
            | "ContextActionMenu" | "Popup" => Self::Popup,
            "RangeField" | "Slider" => Self::Range,
            "Scrollbar" | "ScrollBar" | "ScrollBarTrack" => Self::Scrollbar,
            "ScrollbarThumb" | "ScrollThumb" | "ScrollBarThumb" => Self::ScrollbarThumb,
            "InputField" | "TextField" | "LineEdit" | "TextEdit" | "NumberField" => Self::TextInput,
            "MenuItem" => Self::MenuItem,
            _ => Self::Passive,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWidgetContract {
    pub behavior: UiWidgetBehavior,
    pub disabled: bool,
    pub checked: Option<bool>,
    pub value: Option<UiValue>,
    pub value_property: Option<String>,
    pub checked_property: Option<String>,
    pub open_property: Option<String>,
    pub min_property: Option<String>,
    pub max_property: Option<String>,
    pub step_property: Option<String>,
    pub scroll_target: Option<String>,
    pub scroll_axis: Option<UiAxis>,
    pub min_thumb_extent: Option<f32>,
    pub label_for: Option<String>,
    pub tooltip: Option<String>,
}

impl UiWidgetContract {
    pub fn resolved_behavior(&self, component: &str) -> UiWidgetBehavior {
        match self.behavior {
            UiWidgetBehavior::Auto => UiWidgetBehavior::infer_from_component(component),
            behavior => behavior,
        }
    }
}
