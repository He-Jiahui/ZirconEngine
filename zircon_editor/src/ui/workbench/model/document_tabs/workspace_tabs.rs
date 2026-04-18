use crate::layout::WorkspaceTarget;
use crate::snapshot::{DocumentWorkspaceSnapshot, EditorChromeSnapshot};

use super::super::document_tab_model::DocumentTabModel;
use super::collect::collect_document_tabs;

pub(crate) fn workspace_tabs(
    workspace: &DocumentWorkspaceSnapshot,
    target: WorkspaceTarget,
    chrome: &EditorChromeSnapshot,
) -> Vec<DocumentTabModel> {
    let mut tabs = Vec::new();
    collect_document_tabs(workspace, &target, &mut Vec::new(), chrome, &mut tabs);
    tabs
}
