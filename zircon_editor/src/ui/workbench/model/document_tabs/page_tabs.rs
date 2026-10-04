use crate::ui::workbench::layout::WorkspaceTarget;
use crate::ui::workbench::snapshot::{EditorChromeSnapshot, MainPageSnapshot};

use super::super::document_tab_model::DocumentTabModel;
use super::super::empty_state::empty_state_for_tab;
use super::super::pane_tab::is_closeable_content_kind;
use super::workspace_tabs::workspace_tabs;

/// 独占页面投影单标签，普通页面保留每个文档leaf的路径和选择态。
pub(crate) fn document_tabs_for_page(
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
