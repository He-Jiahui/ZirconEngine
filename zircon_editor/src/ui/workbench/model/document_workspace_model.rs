use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{DocumentWorkspaceSnapshot, ViewTabSnapshot};

#[derive(Clone, Debug)]
pub enum DocumentWorkspaceModel {
    Workbench {
        page_id: MainPageId,
        title: String,
        workspace: DocumentWorkspaceSnapshot,
    },
    Exclusive {
        page_id: MainPageId,
        title: String,
        view: ViewTabSnapshot,
    },
}
