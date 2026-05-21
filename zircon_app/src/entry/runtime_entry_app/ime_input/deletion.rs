use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

use super::super::{converters::usize_to_u32, RuntimeEntryApp};

pub(super) fn forward_ime_delete_surrounding(
    app: &mut RuntimeEntryApp,
    event_loop: &dyn ActiveEventLoop,
    before_bytes: usize,
    after_bytes: usize,
) {
    let event = ZrRuntimeEventV1::ime_delete_surrounding(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        app.viewport,
        usize_to_u32(before_bytes),
        usize_to_u32(after_bytes),
    );
    if app.session.handle_event(event).is_err() {
        event_loop.exit();
    }
}
