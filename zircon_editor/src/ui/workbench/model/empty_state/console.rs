use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::pane_empty_state_model::PaneEmptyStateModel;

pub(super) fn console_empty_state(chrome: &EditorChromeSnapshot) -> PaneEmptyStateModel {
    if chrome.status_line.trim().is_empty() {
        PaneEmptyStateModel {
            title: "No output yet".to_string(),
            body: "Recent task output will appear here.".to_string(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }
    } else {
        PaneEmptyStateModel {
            title: "Last task status".to_string(),
            body: chrome.status_line.clone(),
            primary_action: None,
            secondary_action: None,
            secondary_hint: None,
        }
    }
}
