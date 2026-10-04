use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::view::ViewInstanceId;

use super::pane_tab_model::PaneTabModel;

#[derive(Clone, Debug, PartialEq)]
/// 单抽屉的局部标签栈；保留tabs与当前展开/选择状态分别供布局和呈现使用。
pub struct ToolWindowStackModel {
    pub slot: ActivityDrawerSlot,
    pub mode: crate::ui::workbench::layout::ActivityDrawerMode,
    pub visible: bool,
    pub tabs: Vec<PaneTabModel>,
    pub active_tab: Option<ViewInstanceId>,
}
