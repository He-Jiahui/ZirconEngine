use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_layout::HostPagePointerLayout;

impl HostPagePointerBridge {
    pub(crate) fn sync(&mut self, layout: HostPagePointerLayout) {
        self.layout = layout;
        self.measured_frames.resize(self.layout.items.len(), None);
        if self.measured_frames.len() > self.layout.items.len() {
            self.measured_frames.truncate(self.layout.items.len());
        }
        self.rebuild_surface();
    }
}
