use super::host_page_pointer_bridge::HostPagePointerBridge;
use super::host_page_pointer_layout::HostPagePointerLayout;

impl HostPagePointerBridge {
    pub(crate) fn sync(&mut self, layout: HostPagePointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        self.layout = layout;
        self.measured_frames = vec![None; self.layout.tabs.len()];
        self.tab_positions_by_item = vec![None; self.layout.items.len()];
        for (tab_position, tab) in self.layout.tabs.iter().enumerate() {
            if let Some(position) = self.tab_positions_by_item.get_mut(tab.page_index) {
                *position = Some(tab_position);
            }
        }
        self.rebuild_surface();
        true
    }
}
