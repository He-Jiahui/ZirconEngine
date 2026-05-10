use super::host_drawer_header_pointer_bridge::HostDrawerHeaderPointerBridge;
use super::host_drawer_header_pointer_layout::HostDrawerHeaderPointerLayout;

impl HostDrawerHeaderPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostDrawerHeaderPointerLayout) {
        self.layout = layout;
        self.measured_frames = self
            .layout
            .surfaces
            .iter()
            .map(|surface| (surface.key.clone(), vec![None; surface.items.len()]))
            .collect();
        self.rebuild_surface();
    }
}
