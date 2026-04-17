use std::collections::BTreeMap;

use zircon_ui::{UiNodeId, UiPointerDispatcher, UiSurface};

use super::workbench_activity_rail_pointer_layout::WorkbenchActivityRailPointerLayout;
use super::workbench_activity_rail_pointer_target::WorkbenchActivityRailPointerTarget;

#[derive(Default)]
pub(crate) struct WorkbenchActivityRailPointerBridge {
    pub(super) layout: WorkbenchActivityRailPointerLayout,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, WorkbenchActivityRailPointerTarget>,
}
