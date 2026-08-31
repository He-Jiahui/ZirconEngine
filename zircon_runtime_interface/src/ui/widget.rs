use serde::{Deserialize, Serialize};

use crate::ui::component::UiValue;
use crate::ui::event_ui::UiNodeId;
use crate::ui::layout::UiAxis;
use crate::ui::text::UiTextEditReceipt;

pub const UI_WIDGET_COMPONENT_ROLE_ATTRIBUTE: &str = "component_role";

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
        receipt: Box<UiTextEditReceipt>,
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

/// Declares where a popup obtains its runtime placement anchor.
///
/// Control anchors intentionally carry only a stable UI control identity. The runtime resolves
/// that identity against the current arranged tree so authored templates never persist window
/// coordinates or editor-owned geometry snapshots.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum UiPopupAnchor {
    #[default]
    None,
    Control {
        control_id: String,
    },
    /// Uses the current arranged surface root as the placement frame.
    ///
    /// Surface overlays such as a command palette remain responsive to window and DPI changes
    /// without persisting a host-computed rectangle in template properties.
    Surface,
    /// Uses a transient surface-space point captured for this popup at open time.
    ///
    /// `owner_property` names a string property on the popup whose value identifies the control
    /// that owns dismissal and focus restoration. Pointer coordinates never enter the template.
    Pointer {
        owner_property: String,
    },
}

impl UiPopupAnchor {
    pub fn control_id(&self) -> Option<&str> {
        match self {
            Self::Control { control_id } => Some(control_id.as_str()),
            Self::None | Self::Surface | Self::Pointer { .. } => None,
        }
    }
}

impl UiWidgetBehavior {
    pub fn infer_from_component_role(role: &str) -> Self {
        match role {
            "button" | "icon-button" | "material-button" => Self::Button,
            "toggle" | "checkbox" | "check-box" | "switch" | "toggle-button" => Self::Toggle,
            "radio-group" | "button-group" => Self::RadioGroup,
            "radio" | "radio-button" => Self::Radio,
            "group" | "foldout" | "inspector-section" | "tree-row" | "tree-view" => {
                Self::Disclosure
            }
            "dropdown"
            | "combo-box"
            | "enum-field"
            | "flags-field"
            | "search-select"
            | "menu"
            | "popup-menu"
            | "menu-popup"
            | "context-menu"
            | "context-action-menu"
            | "dropdown-popup"
            | "popup" => Self::Popup,
            "range-field" | "slider" | "range-slider" => Self::Range,
            "scrollbar" | "scroll-bar" | "scroll-bar-track" => Self::Scrollbar,
            "scrollbar-thumb" | "scroll-thumb" | "scroll-bar-thumb" => Self::ScrollbarThumb,
            "input-field" | "text-field" | "line-edit" | "text-edit" | "number-field"
            | "search-field" | "search-input" | "input" | "input-base" | "filled-input"
            | "outlined-input" | "textarea-autosize" | "field-editor" | "source-editor" => {
                Self::TextInput
            }
            "menu-item" => Self::MenuItem,
            _ => Self::Passive,
        }
    }

    pub fn infer_from_component(component: &str) -> Self {
        match component {
            "Button" | "IconButton" | "MaterialButton" => Self::Button,
            "Toggle" | "Checkbox" | "CheckBox" | "Switch" | "ToggleButton" => Self::Toggle,
            "RadioGroup" | "ButtonGroup" => Self::RadioGroup,
            "Radio" | "RadioButton" => Self::Radio,
            "Group" | "Foldout" | "InspectorSection" | "TreeRow" | "TreeView" => Self::Disclosure,
            "Dropdown" | "ComboBox" | "EnumField" | "FlagsField" | "SearchSelect" | "Menu"
            | "PopupMenu" | "MenuPopup" | "ContextMenu" | "ContextActionMenu" | "DropdownPopup"
            | "Popup" => Self::Popup,
            "RangeField" | "Slider" | "RangeSlider" => Self::Range,
            "Scrollbar" | "ScrollBar" | "ScrollBarTrack" => Self::Scrollbar,
            "ScrollbarThumb" | "ScrollThumb" | "ScrollBarThumb" => Self::ScrollbarThumb,
            "InputField" | "TextField" | "LineEdit" | "TextEdit" | "NumberField"
            | "SearchField" | "SearchInput" | "Input" | "InputBase" | "FilledInput"
            | "OutlinedInput" | "TextareaAutosize" | "FieldEditor" | "SourceEditor" => {
                Self::TextInput
            }
            "MenuItem" => Self::MenuItem,
            _ => Self::Passive,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiWidgetContract {
    pub behavior: UiWidgetBehavior,
    #[serde(default)]
    pub popup_anchor: UiPopupAnchor,
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

#[cfg(test)]
mod tests {
    use super::UiWidgetBehavior;

    #[test]
    fn editable_text_component_roles_share_one_behavior_classification() {
        for role in [
            "input-field",
            "text-field",
            "line-edit",
            "text-edit",
            "number-field",
            "search-field",
            "search-input",
            "input",
            "input-base",
            "filled-input",
            "outlined-input",
            "textarea-autosize",
            "field-editor",
            "source-editor",
        ] {
            assert_eq!(
                UiWidgetBehavior::infer_from_component_role(role),
                UiWidgetBehavior::TextInput,
                "role {role} must route through the surface text-input pipeline"
            );
        }
    }

    #[test]
    fn editable_text_component_aliases_share_one_behavior_classification() {
        for component in [
            "InputField",
            "TextField",
            "LineEdit",
            "TextEdit",
            "NumberField",
            "SearchField",
            "SearchInput",
            "Input",
            "InputBase",
            "FilledInput",
            "OutlinedInput",
            "TextareaAutosize",
            "FieldEditor",
            "SourceEditor",
        ] {
            assert_eq!(
                UiWidgetBehavior::infer_from_component(component),
                UiWidgetBehavior::TextInput,
                "component {component} must route through the surface text-input pipeline"
            );
        }
    }
}
