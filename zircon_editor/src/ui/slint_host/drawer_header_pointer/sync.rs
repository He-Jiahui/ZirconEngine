use super::workbench_drawer_header_pointer_bridge::WorkbenchDrawerHeaderPointerBridge;
use super::workbench_drawer_header_pointer_layout::WorkbenchDrawerHeaderPointerLayout;

impl WorkbenchDrawerHeaderPointerBridge {
    pub(crate) fn sync(&mut self, layout: WorkbenchDrawerHeaderPointerLayout) {
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
