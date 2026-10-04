use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;

use super::super::document_tabs::workspace_tabs;
use super::super::floating_window_model::FloatingWindowModel;

/// 保留浮窗身份和原列表顺序；请求frame不是宿主最终确认的几何位置。
pub(super) fn build_floating_windows(chrome: &EditorChromeSnapshot) -> Vec<FloatingWindowModel> {
    let windows = &chrome.workbench.floating_windows;
    let mut output = Vec::with_capacity(windows.len());
    for window in windows {
        output.push(FloatingWindowModel {
            window_id: window.window_id.clone(),
            title: window.title.clone(),
            requested_frame: window.requested_frame,
            focused_view: window.focused_view.clone(),
            tabs: workspace_tabs(
                &window.workspace,
                WorkspaceTarget::FloatingWindow(window.window_id.clone()),
                chrome,
            ),
        });
    }
    output
}

#[cfg(test)]
#[path = "tests/floating_windows_optimization_batch_20260830bt_editor_tests.rs"]
mod optimization_batch_20260830bt_editor_tests;
