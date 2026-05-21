use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1,
    ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1,
};

use super::super::RuntimeEntryApp;

impl RuntimeEntryApp {
    pub(in crate::entry::runtime_entry_app) fn handle_window_focus_changed(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        focused: bool,
    ) {
        let state = if focused {
            ZR_RUNTIME_LIFECYCLE_STATE_FOREGROUND_V1
        } else {
            ZR_RUNTIME_LIFECYCLE_STATE_BACKGROUND_V1
        };
        let event =
            ZrRuntimeEventV1::lifecycle(ZIRCON_RUNTIME_ABI_VERSION_V1, self.viewport, state);
        if self.session.handle_event(event).is_err() {
            event_loop.exit();
        }
    }
}
