use crate::layout::ActivityDrawerSlot;
use crate::view::ViewInstanceId;

use super::pane_tab_model::PaneTabModel;

#[derive(Clone, Debug, PartialEq)]
pub struct ToolWindowStackModel {
    pub slot: ActivityDrawerSlot,
    pub mode: crate::ActivityDrawerMode,
    pub visible: bool,
    pub tabs: Vec<PaneTabModel>,
    pub active_tab: Option<ViewInstanceId>,
}
