//! Backend-neutral workbench host events and stable editor UI bindings.

use thiserror::Error;
use zircon_editor_ui::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use zircon_scene::NodeKind;

use crate::view::ViewDescriptorId;

use super::model::MenuAction;

const WORKBENCH_MENU_VIEW_ID: &str = "WorkbenchMenuBar";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkbenchHostEvent {
    Menu(MenuAction),
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum WorkbenchHostEventError {
    #[error("unsupported workbench binding payload")]
    UnsupportedPayload,
    #[error("unknown menu action id {0}")]
    UnknownMenuAction(String),
}

pub fn menu_action_binding(action: &MenuAction) -> EditorUiBinding {
    let action_id = menu_action_id(action);
    EditorUiBinding::new(
        WORKBENCH_MENU_VIEW_ID,
        action_id.clone(),
        EditorUiEventKind::Click,
        EditorUiBindingPayload::menu_action(action_id),
    )
}

pub fn dispatch_workbench_binding(
    binding: &EditorUiBinding,
) -> Result<WorkbenchHostEvent, WorkbenchHostEventError> {
    match binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => menu_action_from_id(action_id)
            .map(WorkbenchHostEvent::Menu)
            .ok_or_else(|| WorkbenchHostEventError::UnknownMenuAction(action_id.clone())),
        _ => Err(WorkbenchHostEventError::UnsupportedPayload),
    }
}

fn menu_action_id(action: &MenuAction) -> String {
    match action {
        MenuAction::OpenProject => "OpenProject".to_string(),
        MenuAction::SaveProject => "SaveProject".to_string(),
        MenuAction::SaveLayout => "SaveLayout".to_string(),
        MenuAction::ResetLayout => "ResetLayout".to_string(),
        MenuAction::Undo => "Undo".to_string(),
        MenuAction::Redo => "Redo".to_string(),
        MenuAction::CreateNode(kind) => format!("CreateNode.{}", node_kind_id(kind)),
        MenuAction::DeleteSelected => "DeleteSelected".to_string(),
        MenuAction::OpenView(descriptor_id) => format!("OpenView.{}", descriptor_id.0),
    }
}

fn menu_action_from_id(action_id: &str) -> Option<MenuAction> {
    match action_id {
        "OpenProject" => Some(MenuAction::OpenProject),
        "SaveProject" => Some(MenuAction::SaveProject),
        "SaveLayout" => Some(MenuAction::SaveLayout),
        "ResetLayout" => Some(MenuAction::ResetLayout),
        "Undo" => Some(MenuAction::Undo),
        "Redo" => Some(MenuAction::Redo),
        "DeleteSelected" => Some(MenuAction::DeleteSelected),
        _ => {
            if let Some(kind) = action_id.strip_prefix("CreateNode.") {
                return node_kind_from_id(kind).map(MenuAction::CreateNode);
            }
            if let Some(descriptor_id) = action_id.strip_prefix("OpenView.") {
                return Some(MenuAction::OpenView(ViewDescriptorId::new(descriptor_id)));
            }
            None
        }
    }
}

fn node_kind_id(kind: &NodeKind) -> &'static str {
    match kind {
        NodeKind::Camera => "Camera",
        NodeKind::Cube => "Cube",
        NodeKind::Mesh => "Mesh",
        NodeKind::DirectionalLight => "DirectionalLight",
    }
}

fn node_kind_from_id(value: &str) -> Option<NodeKind> {
    match value {
        "Camera" => Some(NodeKind::Camera),
        "Cube" => Some(NodeKind::Cube),
        "Mesh" => Some(NodeKind::Mesh),
        "DirectionalLight" => Some(NodeKind::DirectionalLight),
        _ => None,
    }
}
