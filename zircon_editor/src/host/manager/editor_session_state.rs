use std::collections::BTreeMap;

use crate::layout::{ActivityDrawerSlot, WorkbenchLayout};
use crate::view::{ViewInstance, ViewInstanceId};

#[derive(Clone, Debug)]
pub(super) struct EditorSessionState {
    pub(super) layout: WorkbenchLayout,
    pub(super) open_view_instances: BTreeMap<ViewInstanceId, ViewInstance>,
    pub(super) active_center_tab: Option<ViewInstanceId>,
    pub(super) active_drawers: Vec<ActivityDrawerSlot>,
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
