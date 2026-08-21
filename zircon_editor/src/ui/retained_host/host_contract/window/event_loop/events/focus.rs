use crate::ui::retained_host::host_contract::globals::UiHostContext;

use super::super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(super) fn handle_native_window_focus_lost(&mut self) {
        self.host
            .global::<UiHostContext>()
            .invoke_native_window_focus_lost();
    }
}
