use serde::{Deserialize, Serialize};

use crate::ui::workbench::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::ui::workbench::view::{ViewInstance, ViewInstanceId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectEditorWorkspace {
    pub workbench: WorkbenchLayout,
    pub open_view_instances: Vec<ViewInstance>,
    pub focused_view: Option<ViewInstanceId>,
    pub active_drawers: Vec<ActivityDrawerSlot>,
}
