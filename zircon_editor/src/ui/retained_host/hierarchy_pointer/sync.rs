use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;
use super::row_metrics::current_hierarchy_row_metrics;

impl HierarchyPointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: HierarchyPointerLayout,
        state: HierarchyPointerState,
    ) -> bool {
        let row_metrics = current_hierarchy_row_metrics();
        if self.layout == layout && self.state == state && self.row_metrics == row_metrics {
            return false;
        }

        let surface_geometry_changed = self.layout.pane_width != layout.pane_width
            || self.layout.pane_height != layout.pane_height;
        self.layout = layout;
        self.state = state;
        self.row_metrics = row_metrics;
        self.clamp_scroll_offset();
        if surface_geometry_changed {
            self.patch_surface_geometry();
        }
        true
    }

    pub(super) fn refresh_row_metrics(&mut self) {
        let row_metrics = current_hierarchy_row_metrics();
        if self.row_metrics == row_metrics {
            return;
        }

        self.row_metrics = row_metrics;
        self.clamp_scroll_offset();
    }
}
