use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;

impl HostActivityRailPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostActivityRailPointerLayout) -> bool {
        if self.layout == layout {
            return false;
        }

        self.layout = layout;
        self.rebuild_surface();
        true
    }
}
