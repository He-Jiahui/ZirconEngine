use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;

impl HostActivityRailPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostActivityRailPointerLayout) -> bool {
        if self.layout == layout {
            zircon_runtime::profile_counter!("editor", "ui.activity_rail.sync_reuse_count", 1);
            return false;
        }

        self.layout = layout;
        zircon_runtime::profile_counter!("editor", "ui.activity_rail.sync_rebuild_count", 1);
        self.rebuild_surface();
        true
    }
}
