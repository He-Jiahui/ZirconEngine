use std::collections::BTreeMap;

use crate::ui::workbench::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::ui::workbench::view::{ViewInstance, ViewInstanceId};

#[derive(Clone, Debug)]
pub(crate) struct EditorSessionState {
    pub(crate) layout: WorkbenchLayout,
    pub(crate) open_view_instances: BTreeMap<ViewInstanceId, ViewInstance>,
    pub(crate) active_center_tab: Option<ViewInstanceId>,
    pub(crate) active_drawers: Vec<ActivityDrawerSlot>,
}

impl Default for EditorSessionState {
    fn default() -> Self {
        Self {
            layout: WorkbenchLayout::default(),
            open_view_instances: BTreeMap::new(),
            active_center_tab: None,
            active_drawers: Vec::new(),
        }
    }
}
