use winit::event_loop::ActiveEventLoop;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
};

use super::super::{
    converters::{byte_slice, usize_to_u32},
    RuntimeEntryApp,
};

pub(super) fn forward_ime_preedit(
    app: &mut RuntimeEntryApp,
    event_loop: &dyn ActiveEventLoop,
    value: &str,
    cursor: Option<(usize, usize)>,
) {
    let (cursor_start, cursor_end) = cursor
        .map(|(start, end)| (usize_to_u32(start), usize_to_u32(end)))
        .unwrap_or((
            ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
            ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
        ));
    let event = ZrRuntimeEventV1::ime_preedit(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        app.viewport,
        byte_slice(value),
        cursor_start,
        cursor_end,
    );
    if app.session.handle_event(event).is_err() {
        event_loop.exit();
    }
}

pub(super) fn forward_ime_commit(
    app: &mut RuntimeEntryApp,
    event_loop: &dyn ActiveEventLoop,
    value: &str,
) {
    let event = ZrRuntimeEventV1::ime_commit(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        app.viewport,
        byte_slice(value),
    );
    if app.session.handle_event(event).is_err() {
        event_loop.exit();
    }
}
