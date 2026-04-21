use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::pane_empty_state_model::PaneEmptyStateModel;
use super::action_factory::open_project_action;

pub(super) fn project_or_assets_empty_state(
    chrome: &EditorChromeSnapshot,
) -> Option<PaneEmptyStateModel> {
    if chrome.project_open {
        None
    } else {
        Some(PaneEmptyStateModel {
            title: "No project open".to_string(),
            body: "Open a project to browse files, assets, and content roots.".to_string(),
            primary_action: Some(open_project_action()),
            secondary_action: None,
            secondary_hint: Some("Recent Projects is available from the File menu.".to_string()),
        })
    }
}
