use crate::ui::retained_host::host_contract::data::{
    HostDockOverflowMenuStateData, HostMenuStateData, HostPageOverflowMenuStateData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;

use super::super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(super) fn handle_native_window_focused(&mut self) {
        self.host.notify_native_window_focused();
    }

    pub(super) fn handle_native_window_focus_lost(&mut self) {
        let host = self.host.global::<UiHostContext>();
        host.set_menu_state(HostMenuStateData::default());
        host.set_host_page_overflow_menu_state(HostPageOverflowMenuStateData::default());
        host.set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData::default());
        host.invoke_native_window_focus_lost();
    }
}
