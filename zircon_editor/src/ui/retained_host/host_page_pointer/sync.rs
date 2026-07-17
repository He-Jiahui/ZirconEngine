use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_layout::HostPagePointerLayout;

impl HostPagePointerBridge {
    pub(crate) fn sync(&mut self, layout: HostPagePointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        self.layout = layout;
        self.rebuild_surface();
        true
    }
}
