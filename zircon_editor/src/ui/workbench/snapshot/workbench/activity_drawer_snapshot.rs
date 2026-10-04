use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::view::ViewInstanceId;

use super::ViewTabSnapshot;

#[derive(Clone, Debug)]
/// 当前activity window的抽屉slot；tab选择、焦点、展开模式与可见性分别保留。
pub struct ActivityDrawerSnapshot {
    pub slot: ActivityDrawerSlot,
    pub tabs: Vec<ViewTabSnapshot>,
    pub active_tab: Option<ViewInstanceId>,
    pub active_view: Option<ViewInstanceId>,
    pub mode: ActivityDrawerMode,
    pub extent: f32,
    pub visible: bool,
}
