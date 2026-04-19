use std::collections::BTreeMap;

use zircon_runtime::ui::{dispatch::UiPointerDispatcher, event_ui::UiNodeId, surface::UiSurface};

use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;
use super::welcome_recent_pointer_target::WelcomeRecentPointerTarget;

pub(crate) struct WelcomeRecentPointerBridge {
    pub(in crate::ui::slint_host::welcome_recent_pointer) layout: WelcomeRecentPointerLayout,
    pub(in crate::ui::slint_host::welcome_recent_pointer) state: WelcomeRecentPointerState,
    pub(in crate::ui::slint_host::welcome_recent_pointer) surface: UiSurface,
    pub(in crate::ui::slint_host::welcome_recent_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::ui::slint_host::welcome_recent_pointer) targets:
        BTreeMap<UiNodeId, WelcomeRecentPointerTarget>,
}
