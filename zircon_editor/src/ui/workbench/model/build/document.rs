use crate::ui::workbench::snapshot::MainPageSnapshot;

use super::super::document_workspace_model::DocumentWorkspaceModel;

pub(super) fn build_document_workspace(active_page: MainPageSnapshot) -> DocumentWorkspaceModel {
    match active_page {
        MainPageSnapshot::Workbench {
            id,
            title,
            activity_window: _,
            workspace,
        } => DocumentWorkspaceModel::Workbench {
            page_id: id,
            title,
            workspace,
        },
        MainPageSnapshot::Exclusive { id, title, view } => DocumentWorkspaceModel::Exclusive {
            page_id: id,
            title,
            view,
        },
    }
}
