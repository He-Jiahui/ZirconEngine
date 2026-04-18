use super::helper::{content_height, viewport_frame};
use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;

impl WelcomeRecentPointerBridge {
    pub(in crate::ui::slint_host::welcome_recent_pointer) fn clamp_scroll_offset(&mut self) {
        let viewport_height = viewport_frame(&self.layout).height;
        let content_height = content_height(self.layout.recent_project_paths.len());
        let max_offset = (content_height - viewport_height).max(0.0);
        self.state.scroll_offset = self.state.scroll_offset.clamp(0.0, max_offset);
    }
}
