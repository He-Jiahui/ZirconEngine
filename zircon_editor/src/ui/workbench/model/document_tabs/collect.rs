use crate::layout::WorkspaceTarget;
use crate::snapshot::{DocumentWorkspaceSnapshot, EditorChromeSnapshot};

use super::super::document_tab_model::DocumentTabModel;
use super::super::empty_state::empty_state_for_tab;
use super::super::pane_tab::is_closeable_content_kind;

pub(super) fn collect_document_tabs(
    workspace: &DocumentWorkspaceSnapshot,
    target: &WorkspaceTarget,
    path: &mut Vec<usize>,
    chrome: &EditorChromeSnapshot,
    output: &mut Vec<DocumentTabModel>,
) {
    match workspace {
        DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            path.push(0);
            collect_document_tabs(first, target, path, chrome, output);
            path.pop();
            path.push(1);
            collect_document_tabs(second, target, path, chrome, output);
            path.pop();
        }
        DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => {
            for tab in tabs {
                output.push(DocumentTabModel {
                    workspace: target.clone(),
                    workspace_path: path.clone(),
                    instance_id: tab.instance_id.clone(),
                    descriptor_id: tab.descriptor_id.clone(),
                    title: tab.title.clone(),
                    icon_key: tab.icon_key.clone(),
                    content_kind: tab.content_kind,
                    active: active_tab.as_ref() == Some(&tab.instance_id),
                    closeable: is_closeable_content_kind(tab.content_kind),
                    empty_state: empty_state_for_tab(tab, chrome),
                });
            }
        }
    }
}
