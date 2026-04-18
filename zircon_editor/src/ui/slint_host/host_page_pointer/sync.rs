use super::workbench_host_page_pointer_bridge::WorkbenchHostPagePointerBridge;
use super::workbench_host_page_pointer_layout::WorkbenchHostPagePointerLayout;

impl WorkbenchHostPagePointerBridge {
    pub(crate) fn sync(&mut self, layout: WorkbenchHostPagePointerLayout) {
        self.layout = layout;
        self.measured_frames.resize(self.layout.items.len(), None);
        if self.measured_frames.len() > self.layout.items.len() {
            self.measured_frames.truncate(self.layout.items.len());
        }
        self.rebuild_surface();
    }
}
