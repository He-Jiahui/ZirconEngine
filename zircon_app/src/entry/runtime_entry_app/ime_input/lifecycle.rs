use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::RuntimeEntryApp;

pub(super) fn forward_ime_enabled(app: &mut RuntimeEntryApp, event_loop: &dyn ActiveEventLoop) {
    let event = ZrRuntimeEventV1::ime_enabled(ZIRCON_RUNTIME_ABI_VERSION_V1, app.viewport);
    if app.session.handle_event(event).is_err() {
        event_loop.exit();
    }
}

pub(super) fn forward_ime_disabled(app: &mut RuntimeEntryApp, event_loop: &dyn ActiveEventLoop) {
    let event = ZrRuntimeEventV1::ime_disabled(ZIRCON_RUNTIME_ABI_VERSION_V1, app.viewport);
    if app.session.handle_event(event).is_err() {
        event_loop.exit();
    }
}
