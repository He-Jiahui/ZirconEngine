use crate::snapshot::{EditorChromeSnapshot, ViewContentKind, ViewTabSnapshot};
use crate::workbench::event::menu_action_binding;

use super::menu_action::MenuAction;
use super::pane_action_model::PaneActionModel;
use super::pane_empty_state_model::PaneEmptyStateModel;

pub(super) fn empty_state_for_tab(
    tab: &ViewTabSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Option<PaneEmptyStateModel> {
    match tab.content_kind {
        ViewContentKind::Welcome => None,
        ViewContentKind::Project | ViewContentKind::Assets if !chrome.project_open => {
            Some(PaneEmptyStateModel {
                title: "No project open".to_string(),
                body: "Open a project to browse files, assets, and content roots.".to_string(),
                primary_action: Some(open_project_action()),
                secondary_action: None,
                secondary_hint: Some(
                    "Recent Projects is available from the File menu.".to_string(),
                ),
            })
        }
        ViewContentKind::Hierarchy if !chrome.project_open => Some(PaneEmptyStateModel {
            title: "No scene loaded".to_string(),
            body: "Open a project to inspect the active scene hierarchy.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Hierarchy if chrome.scene_entries.is_empty() => {
            Some(PaneEmptyStateModel {
                title: "No nodes in scene".to_string(),
                body: "Create or open a scene to populate the hierarchy.".to_string(),
                primary_action: None,
                secondary_action: None,
                secondary_hint: None,
            })
        }
        ViewContentKind::Scene if !chrome.project_open => Some(PaneEmptyStateModel {
            title: "No project open".to_string(),
            body: "Open a project to enter the editor workspace.".to_string(),
            primary_action: Some(open_project_action()),
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Scene if chrome.scene_entries.is_empty() => Some(PaneEmptyStateModel {
            title: "No active scene".to_string(),
            body: "Open Scene or Create Scene to begin editing.".to_string(),
            primary_action: Some(open_scene_action()),
            secondary_action: Some(create_scene_action()),
            secondary_hint: None,
        }),
        ViewContentKind::Inspector if chrome.inspector.is_none() => Some(PaneEmptyStateModel {
            title: "Nothing selected".to_string(),
            body: "Select an item in Hierarchy or Scene to inspect it.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Console if chrome.status_line.trim().is_empty() => {
            Some(PaneEmptyStateModel {
                title: "No output yet".to_string(),
                body: "Recent task output will appear here.".to_string(),
                primary_action: None,
                secondary_action: None,
                secondary_hint: None,
            })
        }
        ViewContentKind::Console => Some(PaneEmptyStateModel {
            title: "Last task status".to_string(),
            body: chrome.status_line.clone(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        ViewContentKind::Placeholder => Some(PaneEmptyStateModel {
            title: "View unavailable".to_string(),
            body: "This pane was restored from layout state but its descriptor is missing."
                .to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }),
        _ => None,
    }
}

fn open_project_action() -> PaneActionModel {
    PaneActionModel {
        label: "Open Project".to_string(),
        binding: Some(menu_action_binding(&MenuAction::OpenProject)),
        prominent: true,
    }
}

fn open_scene_action() -> PaneActionModel {
    PaneActionModel {
        label: "Open Scene".to_string(),
        binding: Some(menu_action_binding(&MenuAction::OpenScene)),
        prominent: true,
    }
}

fn create_scene_action() -> PaneActionModel {
    PaneActionModel {
        label: "Create Scene".to_string(),
        binding: Some(menu_action_binding(&MenuAction::CreateScene)),
        prominent: false,
    }
}
