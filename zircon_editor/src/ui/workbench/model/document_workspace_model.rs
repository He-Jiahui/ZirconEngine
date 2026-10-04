use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::{DocumentWorkspaceSnapshot, ViewTabSnapshot};

#[derive(Clone, Debug)]
/// 当前页面的内容边界；独占页是单视图，不参加普通文档树渲染。
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
