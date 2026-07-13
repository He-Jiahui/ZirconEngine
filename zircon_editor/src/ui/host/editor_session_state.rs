use std::collections::BTreeMap;

use crate::ui::workbench::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::ui::workbench::view::{ViewInstance, ViewInstanceId};

#[derive(Clone, Debug)]
pub(crate) struct EditorSessionState {
    pub(crate) layout: WorkbenchLayout,
    pub(crate) open_view_instances: BTreeMap<ViewInstanceId, ViewInstance>,
    pub(crate) focused_view: Option<ViewInstanceId>,
    pub(crate) active_drawers: Vec<ActivityDrawerSlot>,
}

impl Default for EditorSessionState {
    fn default() -> Self {
        Self {
            layout: WorkbenchLayout::default(),
            open_view_instances: BTreeMap::new(),
            focused_view: None,
            active_drawers: Vec::new(),
        }
    }
}
