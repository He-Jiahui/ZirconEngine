use serde::{Deserialize, Serialize};

use crate::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::view::{ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectEditorWorkspace {
    pub layout_version: u32,
    pub workbench: WorkbenchLayout,
    pub open_view_instances: Vec<ViewInstance>,
    pub active_center_tab: Option<ViewInstanceId>,
    pub active_drawers: Vec<ActivityDrawerSlot>,
}
