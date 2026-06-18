use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::status_bar_model::StatusBarModel;

pub(super) fn build_status_bar(chrome: &EditorChromeSnapshot) -> StatusBarModel {
    StatusBarModel::from_chrome(chrome)
}
