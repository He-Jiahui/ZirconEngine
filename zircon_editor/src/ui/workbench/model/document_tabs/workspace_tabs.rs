use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::snapshot::{DocumentWorkspaceSnapshot, EditorChromeSnapshot};

use super::super::document_tab_model::DocumentTabModel;
use super::collect::collect_document_tabs;

#[cfg(test)]
#[path = "workspace_tabs/tests/capacity_tests.rs"]
mod capacity_tests;

/// 将主页面或浮窗的文档树摊平为带路径标签；target必须与输入树的宿主一致。
pub(crate) fn workspace_tabs(
    workspace: &DocumentWorkspaceSnapshot,
    target: WorkspaceTarget,
    chrome: &EditorChromeSnapshot,
) -> Vec<DocumentTabModel> {
    let mut tabs = Vec::with_capacity(workspace_document_tab_count(workspace));
    collect_document_tabs(workspace, &target, &mut Vec::new(), chrome, &mut tabs);
    tabs
}

fn workspace_document_tab_count(workspace: &DocumentWorkspaceSnapshot) -> usize {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            workspace_document_tab_count(first).saturating_add(workspace_document_tab_count(second))
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, .. } => tabs.len(),
    }
}
