use super::hierarchy_pointer_bridge::HierarchyPointerBridge;
use super::hierarchy_pointer_layout::HierarchyPointerLayout;
use super::hierarchy_pointer_state::HierarchyPointerState;
use super::row_metrics::current_hierarchy_row_metrics;

#[cfg(test)]
#[path = "tests/sync.rs"]
mod tests;

impl HierarchyPointerBridge {
    pub(in crate::ui::retained_host) fn set_authored_row_metrics(
        &mut self,
        metrics: Option<super::row_metrics::HierarchyRowMetrics>,
    ) -> bool {
        let previous = self.row_metrics;
        self.authored_row_metrics = metrics;
        self.refresh_row_metrics();
        previous != self.row_metrics
    }

    pub(in crate::ui::retained_host) fn committed_state(&self) -> HierarchyPointerState {
        self.state
    }

    pub(crate) fn sync(
        &mut self,
        layout: HierarchyPointerLayout,
        state: HierarchyPointerState,
    ) -> bool {
        let row_metrics = self
            .authored_row_metrics
            .unwrap_or_else(current_hierarchy_row_metrics);
        if self.layout == layout && self.state == state && self.row_metrics == row_metrics {
            return false;
        }

        if self.layout != layout || self.row_metrics != row_metrics {
            self.cancel_reparent_drag();
        }
        self.layout = layout;
        self.state = state;
        self.row_metrics = row_metrics;
        self.clamp_scroll_offset();
        true
    }

    pub(super) fn refresh_row_metrics(&mut self) {
        let row_metrics = self
            .authored_row_metrics
            .unwrap_or_else(current_hierarchy_row_metrics);
        if self.row_metrics == row_metrics {
            return;
        }

        self.cancel_reparent_drag();
        self.row_metrics = row_metrics;
        self.clamp_scroll_offset();
    }
}
