use super::host_activity_rail_pointer_bridge::HostActivityRailPointerBridge;
use super::host_activity_rail_pointer_layout::HostActivityRailPointerLayout;

impl HostActivityRailPointerBridge {
    pub(crate) fn sync(&mut self, layout: HostActivityRailPointerLayout) -> bool {
        if self.layout == layout {
            zircon_runtime::profile_counter!("editor", "ui.activity_rail.sync_reuse_count", 1);
            return false;
        }

        let previous_layout = std::mem::replace(&mut self.layout, layout);
        match super::surface_delta::surface_delta_for_layout(&previous_layout, &self.layout) {
            super::surface_delta::ActivityRailSurfaceDelta::NoChange => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.activity_rail.sync_geometry_no_change_count",
                    1
                );
            }
            super::surface_delta::ActivityRailSurfaceDelta::Geometry(changes) => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.activity_rail.sync_geometry_patch_count",
                    1
                );
                self.apply_geometry_delta(changes);
            }
            super::surface_delta::ActivityRailSurfaceDelta::Topology => {
                zircon_runtime::profile_counter!(
                    "editor",
                    "ui.activity_rail.sync_rebuild_count",
                    1
                );
                self.rebuild_surface();
            }
        }
        true
    }
}
