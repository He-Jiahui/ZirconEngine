use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;

impl HostActivityRailPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostActivityRailPointerLayout) {
        self.layout = layout;
        self.rebuild_surface();
    }
}
