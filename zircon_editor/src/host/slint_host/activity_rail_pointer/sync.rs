use super::workbench_activity_rail_pointer_bridge::WorkbenchActivityRailPointerBridge;
use super::workbench_activity_rail_pointer_layout::WorkbenchActivityRailPointerLayout;

impl WorkbenchActivityRailPointerBridge {
    pub(crate) fn sync(&mut self, layout: WorkbenchActivityRailPointerLayout) {
        self.layout = layout;
        self.rebuild_surface();
    }
}
