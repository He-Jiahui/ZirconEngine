use crate::ui::retained_host::host_contract::data::{
    HostDockOverflowMenuStateData, HostMenuStateData, HostPageOverflowMenuStateData,
};
use crate::ui::retained_host::host_contract::globals::UiHostContext;
use crate::ui::retained_host::host_contract::native_pointer::HOST_POINTER_CANCEL;

use super::super::UiHostWindowEventLoop;

impl UiHostWindowEventLoop {
    pub(super) fn handle_native_window_focused(&mut self) {
        self.host.notify_native_window_focused();
    }

    pub(super) fn handle_native_window_focus_lost(&mut self) {
        let capture_canceled = self.cancel_native_pointer_capture();
        let host = self.host.global::<UiHostContext>();
        host.set_menu_state(HostMenuStateData::default());
        host.set_host_page_overflow_menu_state(HostPageOverflowMenuStateData::default());
        host.set_host_dock_overflow_menu_state(HostDockOverflowMenuStateData::default());
        host.invoke_native_window_focus_lost();
        self.dispatch_pointer_result(self.host.clear_template_button_press());
        if capture_canceled {
            self.host.request_frame_update();
        }
    }

    pub(super) fn cancel_native_pointer_capture(&mut self) -> bool {
        let host = self.host.global::<UiHostContext>();
        let (drag_canceled, resize_canceled) = host.cancel_native_primary_capture();
        if drag_canceled {
            host.invoke_host_drag_pointer_event(HOST_POINTER_CANCEL, 0.0, 0.0);
        }
        if resize_canceled {
            host.invoke_host_resize_pointer_event(HOST_POINTER_CANCEL, 0.0, 0.0);
        }
        drag_canceled || resize_canceled
    }
}
