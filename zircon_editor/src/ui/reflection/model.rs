use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::event_ui::{UiRouteId, UiTreeId};

use super::activity::EditorActivityReflection;
use crate::ui::binding::EditorUiBinding;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorMenuItemReflectionModel {
    pub menu_id: String,
    pub control_id: String,
    pub label: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
    pub binding: EditorUiBinding,
    pub route_id: Option<UiRouteId>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorHostPageReflectionModel {
    pub page_id: String,
    pub title: String,
    pub active: bool,
    pub exclusive: bool,
    pub activities: Vec<EditorActivityReflection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorFloatingWindowReflectionModel {
    pub window_id: String,
    pub title: String,
    pub activities: Vec<EditorActivityReflection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorDrawerReflectionModel {
    pub drawer_id: String,
    pub title: String,
    pub visible: bool,
    pub activities: Vec<EditorActivityReflection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditorWorkbenchReflectionModel {
    pub tree_id: UiTreeId,
    pub status_line: String,
    pub menu_items: Vec<EditorMenuItemReflectionModel>,
    pub pages: Vec<EditorHostPageReflectionModel>,
    pub drawers: Vec<EditorDrawerReflectionModel>,
    pub floating_windows: Vec<EditorFloatingWindowReflectionModel>,
}

impl EditorWorkbenchReflectionModel {
    pub fn new(tree_id: UiTreeId) -> Self {
        Self {
            tree_id,
            status_line: String::new(),
            menu_items: Vec::new(),
            pages: Vec::new(),
            drawers: Vec::new(),
            floating_windows: Vec::new(),
        }
    }
}
