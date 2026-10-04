use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use crate::ui::retained_host::welcome_recent_geometry::current_welcome_recent_layout_metrics;
use zircon_runtime_interface::ui::layout::UiFrame;

impl WelcomeRecentPointerBridge {
    pub(crate) fn sync(&mut self, layout: WelcomeRecentPointerLayout) -> bool {
        let layout_metrics = current_welcome_recent_layout_metrics();
        if self.layout == layout && self.layout_metrics == layout_metrics {
            return false;
        }

        let previous_state = self.state;
        self.layout = layout;
        self.layout_metrics = layout_metrics;
        self.clamp_scroll_offset();
        self.clamp_hovered_item();
        self.state != previous_state
    }

    pub(crate) fn sync_viewport(&mut self, viewport: UiFrame) -> bool {
        let layout_metrics = current_welcome_recent_layout_metrics();
        if self.layout.viewport == viewport && self.layout_metrics == layout_metrics {
            return false;
        }

        let previous_state = self.state;
        self.layout.viewport = viewport;
        self.layout_metrics = layout_metrics;
        self.clamp_scroll_offset();
        self.clamp_hovered_item();
        self.state != previous_state
    }

    pub(in crate::ui::retained_host::welcome_recent_pointer) fn refresh_layout_metrics(&mut self) {
        let layout_metrics = current_welcome_recent_layout_metrics();
        if self.layout_metrics == layout_metrics {
            return;
        }

        self.layout_metrics = layout_metrics;
        self.clamp_scroll_offset();
    }

    fn clamp_hovered_item(&mut self) {
        if self
            .state
            .hovered_item_index
            .is_some_and(|index| index >= self.layout.recent_project_paths.len())
        {
            self.state.hovered_item_index = None;
            self.state.hovered_action = None;
        }
    }
}

#[cfg(test)]
#[path = "tests/welcome_recent_pointer_bridge_sync.rs"]
mod tests;
