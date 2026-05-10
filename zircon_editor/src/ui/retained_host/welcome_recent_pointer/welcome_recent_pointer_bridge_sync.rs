use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;

impl WelcomeRecentPointerBridge {
    pub(crate) fn sync(
        &mut self,
        layout: WelcomeRecentPointerLayout,
        state: WelcomeRecentPointerState,
    ) {
        self.layout = layout;
        self.state = state;
        self.clamp_scroll_offset();
        self.rebuild_surface();
    }
}
