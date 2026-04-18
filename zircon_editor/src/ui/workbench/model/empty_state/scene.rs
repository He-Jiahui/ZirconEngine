use crate::snapshot::EditorChromeSnapshot;

use super::super::pane_empty_state_model::PaneEmptyStateModel;
use super::action_factory::{create_scene_action, open_project_action, open_scene_action};

pub(super) fn scene_empty_state(chrome: &EditorChromeSnapshot) -> Option<PaneEmptyStateModel> {
    if !chrome.project_open {
        Some(PaneEmptyStateModel {
            title: "No project open".to_string(),
            body: "Open a project to enter the editor workspace.".to_string(),
            primary_action: Some(open_project_action()),
            secondary_action: None,
            secondary_hint: None,
        })
    } else if chrome.scene_entries.is_empty() {
        Some(PaneEmptyStateModel {
            title: "No active scene".to_string(),
            body: "Open Scene or Create Scene to begin editing.".to_string(),
            primary_action: Some(open_scene_action()),
            secondary_action: Some(create_scene_action()),
            secondary_hint: None,
        })
    } else {
        None
    }
}
