use crate::layout::WorkspaceTarget;
use crate::snapshot::{DocumentWorkspaceSnapshot, EditorChromeSnapshot, MainPageSnapshot};

use super::document_tab_model::DocumentTabModel;
use super::empty_state::empty_state_for_tab;
use super::pane_tab::is_closeable_content_kind;

pub(super) fn document_tabs_for_page(
    page: &MainPageSnapshot,
    chrome: &EditorChromeSnapshot,
) -> Vec<DocumentTabModel> {
    match page {
        MainPageSnapshot::Workbench { id, workspace, .. } => {
            workspace_tabs(workspace, WorkspaceTarget::MainPage(id.clone()), chrome)
        }
        MainPageSnapshot::Exclusive { id, view, .. } => vec![DocumentTabModel {
            workspace: WorkspaceTarget::MainPage(id.clone()),
            workspace_path: Vec::new(),
            instance_id: view.instance_id.clone(),
            descriptor_id: view.descriptor_id.clone(),
            title: view.title.clone(),
            icon_key: view.icon_key.clone(),
            content_kind: view.content_kind,
            active: true,
            closeable: is_closeable_content_kind(view.content_kind),
            empty_state: empty_state_for_tab(view, chrome),
        }],
    }
}

pub(super) fn workspace_tabs(
    workspace: &DocumentWorkspaceSnapshot,
    target: WorkspaceTarget,
    chrome: &EditorChromeSnapshot,
) -> Vec<DocumentTabModel> {
    let mut tabs = Vec::new();
    collect_document_tabs(workspace, &target, &mut Vec::new(), chrome, &mut tabs);
    tabs
}

fn collect_document_tabs(
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
