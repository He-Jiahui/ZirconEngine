use super::workbench_menu_pointer_bridge::WorkbenchMenuPointerBridge;
use super::workbench_menu_pointer_layout::WorkbenchMenuPointerLayout;
use super::workbench_menu_pointer_state::WorkbenchMenuPointerState;

impl WorkbenchMenuPointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: WorkbenchMenuPointerLayout,
        state: WorkbenchMenuPointerState,
    ) {
        self.layout = layout;
        self.state = state;
        self.clamp_popup_scroll_offset();
        self.rebuild_surface();
    }
}
