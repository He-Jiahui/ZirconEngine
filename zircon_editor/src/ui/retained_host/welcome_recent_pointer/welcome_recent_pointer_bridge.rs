use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};

use crate::ui::retained_host::route_intent::EditorRouteIntentMap;
use crate::ui::retained_host::welcome_recent_geometry::WelcomeRecentLayoutMetrics;

use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;

pub(crate) struct WelcomeRecentPointerBridge {
    pub(in crate::ui::retained_host::welcome_recent_pointer) layout: WelcomeRecentPointerLayout,
    pub(in crate::ui::retained_host::welcome_recent_pointer) state: WelcomeRecentPointerState,
    pub(in crate::ui::retained_host::welcome_recent_pointer) layout_metrics:
        WelcomeRecentLayoutMetrics,
    pub(in crate::ui::retained_host::welcome_recent_pointer) surface: UiSurface,
    pub(in crate::ui::retained_host::welcome_recent_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::retained_host::welcome_recent_pointer) route_intents: EditorRouteIntentMap,
}
