use crate::snapshot::EditorChromeSnapshot;

use super::super::pane_empty_state_model::PaneEmptyStateModel;

pub(super) fn hierarchy_empty_state(chrome: &EditorChromeSnapshot) -> Option<PaneEmptyStateModel> {
    if !chrome.project_open {
        Some(PaneEmptyStateModel {
            title: "No scene loaded".to_string(),
            body: "Open a project to inspect the active scene hierarchy.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        })
    } else if chrome.scene_entries.is_empty() {
        Some(PaneEmptyStateModel {
            title: "No nodes in scene".to_string(),
            body: "Create or open a scene to populate the hierarchy.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        })
    } else {
        None
    }
}
