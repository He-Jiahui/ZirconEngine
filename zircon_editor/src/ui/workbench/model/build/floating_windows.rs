use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::document_tabs::workspace_tabs;
use super::super::floating_window_model::FloatingWindowModel;

pub(super) fn build_floating_windows(chrome: &EditorChromeSnapshot) -> Vec<FloatingWindowModel> {
    chrome
        .workbench
        .floating_windows
        .iter()
        .map(|window| FloatingWindowModel {
            window_id: window.window_id.clone(),
            title: window.title.clone(),
            requested_frame: window.requested_frame,
            focused_view: window.focused_view.clone(),
            tabs: workspace_tabs(
                &window.workspace,
                WorkspaceTarget::FloatingWindow(window.window_id.clone()),
                chrome,
            ),
        })
        .collect()
}
