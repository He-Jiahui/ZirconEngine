use serde::{Deserialize, Serialize};

use crate::ui::event_ui::{UiNodeId, UiNodePath, UiTreeId};
use crate::ui::layout::UiFrame;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiA11yRole {
    #[default]
    Generic,
    Button,
    Checkbox,
    Radio,
    Slider,
    Text,
    TextInput,
    Image,
    List,
    ListItem,
    Menu,
    MenuItem,
    Tab,
    TabList,
    Panel,
    Dialog,
    Tooltip,
    Scrollbar,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiA11yCheckedState {
    False,
    True,
    Mixed,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiA11yState {
    pub disabled: bool,
    pub hidden: bool,
    pub focused: bool,
    pub selected: bool,
    pub expanded: Option<bool>,
    pub checked: Option<UiA11yCheckedState>,
    pub pressed: Option<bool>,
    pub value: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAccessibilityAction {
    Activate,
    Focus,
    Increment,
    Decrement,
    SetValue,
    ScrollTo,
    Dismiss,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiAccessibilityNode {
    pub node_id: UiNodeId,
    pub node_path: Option<UiNodePath>,
    pub role: UiA11yRole,
    pub name: Option<String>,
    pub description: Option<String>,
    pub bounds: Option<UiFrame>,
    pub state: UiA11yState,
    pub actions: Vec<UiAccessibilityAction>,
    pub children: Vec<UiNodeId>,
    pub labelled_by: Option<UiNodeId>,
    pub label_for: Option<UiNodeId>,
    pub tooltip: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiAccessibilityTreeSnapshot {
    pub tree_id: UiTreeId,
    pub roots: Vec<UiNodeId>,
    pub nodes: Vec<UiAccessibilityNode>,
    pub focused: Option<UiNodeId>,
    pub diagnostics: Vec<UiAccessibilityDiagnostic>,
}

impl UiAccessibilityTreeSnapshot {
    pub fn node(&self, node_id: UiNodeId) -> Option<&UiAccessibilityNode> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiAccessibilityDiagnostic {
    pub severity: UiAccessibilityDiagnosticSeverity,
    pub code: UiAccessibilityDiagnosticCode,
    pub node_id: Option<UiNodeId>,
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAccessibilityDiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UiAccessibilityDiagnosticCode {
    MissingName,
    MissingRole,
    InvalidLabelReference,
    HiddenFocusable,
    DisabledAction,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UiAccessibilityContract {
    pub role: UiA11yRole,
    pub name: Option<String>,
    pub description: Option<String>,
    pub label_for: Option<String>,
    pub labelled_by: Option<String>,
    pub tooltip: Option<String>,
    pub actions: Vec<UiAccessibilityAction>,
}
