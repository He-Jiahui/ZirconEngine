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
mod tests {
    use super::*;

    #[test]
    fn sync_viewport_preserves_recent_project_paths() {
        let mut bridge = WelcomeRecentPointerBridge::new();
        let project_paths = vec![String::from("E:/ProjectA"), String::from("E:/ProjectB")];

        assert!(!bridge.sync(WelcomeRecentPointerLayout {
            viewport: UiFrame::new(8.0, 12.0, 120.0, 80.0),
            recent_project_paths: project_paths.clone(),
        }));

        assert!(!bridge.sync_viewport(UiFrame::new(8.0, 12.0, 120.0, 80.0)));
        assert_eq!(bridge.layout.recent_project_paths, project_paths);

        assert!(!bridge.sync_viewport(UiFrame::new(8.0, 12.0, 180.0, 80.0)));
        assert_eq!(
            bridge.layout.recent_project_paths,
            vec![String::from("E:/ProjectA"), String::from("E:/ProjectB")]
        );
    }
}
