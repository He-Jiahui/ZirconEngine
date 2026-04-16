use std::collections::BTreeMap;

use zircon_ui::{UiNodeId, UiPointerDispatcher, UiSurface};

use super::welcome_recent_pointer_layout::WelcomeRecentPointerLayout;
use super::welcome_recent_pointer_state::WelcomeRecentPointerState;
use super::welcome_recent_pointer_target::WelcomeRecentPointerTarget;

pub(crate) struct WelcomeRecentPointerBridge {
    pub(in crate::host::slint_host::welcome_recent_pointer) layout: WelcomeRecentPointerLayout,
    pub(in crate::host::slint_host::welcome_recent_pointer) state: WelcomeRecentPointerState,
    pub(in crate::host::slint_host::welcome_recent_pointer) surface: UiSurface,
    pub(in crate::host::slint_host::welcome_recent_pointer) dispatcher: UiPointerDispatcher,
    pub(in crate::host::slint_host::welcome_recent_pointer) targets:
        BTreeMap<UiNodeId, WelcomeRecentPointerTarget>,
}
