use std::collections::BTreeMap;

use zircon_ui::{UiFrame, UiNodeId, UiPointerDispatcher, UiSurface};

use super::workbench_drawer_header_pointer_layout::WorkbenchDrawerHeaderPointerLayout;
use super::workbench_drawer_header_pointer_target::WorkbenchDrawerHeaderPointerTarget;

#[derive(Default)]
pub(crate) struct WorkbenchDrawerHeaderPointerBridge {
    pub(super) layout: WorkbenchDrawerHeaderPointerLayout,
    pub(super) measured_frames: BTreeMap<String, Vec<Option<UiFrame>>>,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, WorkbenchDrawerHeaderPointerTarget>,
}
