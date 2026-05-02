use zircon_runtime::ui::{dispatch::UiPointerDispatcher, surface::UiSurface};
use zircon_runtime_interface::ui::event_ui::UiTreeId;

use super::welcome_recent_pointer_bridge::WelcomeRecentPointerBridge;
use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;

impl WelcomeRecentPointerBridge {
    pub(crate) fn new() -> Self {
        let mut bridge = Self {
            layout: WelcomeRecentPointerLayout::default(),
            state: WelcomeRecentPointerState::default(),
            surface: UiSurface::new(UiTreeId::new("zircon.editor.welcome.recent_pointer")),
            dispatcher: UiPointerDispatcher::default(),
            targets: Default::default(),
        };
        bridge.rebuild_surface();
        bridge
    }
}
