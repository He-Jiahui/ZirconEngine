use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::snapshot::MainPageSnapshot;

use super::active_view::active_view_in_workspace;

pub(super) fn page_id(page: &MainPageSnapshot) -> &MainPageId {
    match page {
        MainPageSnapshot::Workbench { id, .. } | MainPageSnapshot::Exclusive { id, .. } => id,
    }
}

pub(super) fn page_title(page: &MainPageSnapshot) -> &str {
    match page {
        MainPageSnapshot::Workbench { title, .. } | MainPageSnapshot::Exclusive { title, .. } => {
            title
        }
    }
}

// TODO: [CR-EDITOR-WORKBENCH-0007] 确认host页面dirty标记应表示当前标签还是整个页面；普通文档树只检查首个活动标签，未覆盖其他split或非活动标签的dirty状态。
/// host页面目前沿首个活动标签显示dirty；需明确页面级汇总语义再扩展。
pub(super) fn page_dirty(page: &MainPageSnapshot) -> bool {
    match page {
        MainPageSnapshot::Workbench { workspace, .. } => active_view_in_workspace(workspace)
            .map(|view| view.dirty)
            .unwrap_or(false),
        MainPageSnapshot::Exclusive { view, .. } => view.dirty,
    }
}
