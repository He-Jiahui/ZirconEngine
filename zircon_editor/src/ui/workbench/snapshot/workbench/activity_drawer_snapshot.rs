use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::view::ViewInstanceId;

use super::ViewTabSnapshot;

#[derive(Clone, Debug)]
pub struct ActivityDrawerSnapshot {
    pub slot: ActivityDrawerSlot,
    pub tabs: Vec<ViewTabSnapshot>,
    pub active_tab: Option<ViewInstanceId>,
    pub active_view: Option<ViewInstanceId>,
    pub mode: ActivityDrawerMode,
    pub extent: f32,
    pub visible: bool,
}
