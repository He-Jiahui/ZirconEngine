use std::collections::BTreeMap;

use zircon_ui::{UiFrame, UiNodeId, UiPointerDispatcher, UiSurface};

use super::workbench_host_page_pointer_layout::WorkbenchHostPagePointerLayout;
use super::workbench_host_page_pointer_target::WorkbenchHostPagePointerTarget;

#[derive(Default)]
pub(crate) struct WorkbenchHostPagePointerBridge {
    pub(super) layout: WorkbenchHostPagePointerLayout,
    pub(super) measured_frames: Vec<Option<UiFrame>>,
    pub(super) surface: UiSurface,
    pub(super) dispatcher: UiPointerDispatcher,
    pub(super) targets: BTreeMap<UiNodeId, WorkbenchHostPagePointerTarget>,
}
