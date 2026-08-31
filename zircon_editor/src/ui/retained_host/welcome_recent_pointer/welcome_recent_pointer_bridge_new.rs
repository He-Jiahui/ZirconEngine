use crate::ui::retained_host::welcome_recent_geometry::current_welcome_recent_layout_metrics;

use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;

impl WelcomeRecentPointerBridge {
    pub(crate) fn new() -> Self {
        Self {
            layout: WelcomeRecentPointerLayout::default(),
            state: WelcomeRecentPointerState::default(),
            layout_metrics: current_welcome_recent_layout_metrics(),
        }
    }
}
