use crate::ui::retained_host::welcome_recent_geometry::WelcomeRecentLayoutMetrics;

use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;

pub(crate) struct WelcomeRecentPointerBridge {
    pub(in crate::ui::retained_host::welcome_recent_pointer) layout: WelcomeRecentPointerLayout,
    pub(in crate::ui::retained_host::welcome_recent_pointer) state: WelcomeRecentPointerState,
    pub(in crate::ui::retained_host::welcome_recent_pointer) layout_metrics:
        WelcomeRecentLayoutMetrics,
}

impl WelcomeRecentPointerBridge {
    pub(crate) const fn state(&self) -> WelcomeRecentPointerState {
        self.state
    }
}
