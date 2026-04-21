use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::status_bar_model::StatusBarModel;

pub(super) fn build_status_bar(chrome: &EditorChromeSnapshot) -> StatusBarModel {
    StatusBarModel {
        primary_text: chrome.status_line.clone(),
        secondary_text: chrome
            .inspector
            .as_ref()
            .map(|inspector| format!("Selection {}", inspector.name)),
        viewport_label: format!("{} x {}", chrome.viewport_size.x, chrome.viewport_size.y),
    }
}
