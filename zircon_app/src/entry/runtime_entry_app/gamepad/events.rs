use gilrs::{Axis, Button};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1,
};

use super::codes::{axis_code, button_code};
use crate::entry::runtime_library::{RuntimeLibraryError, RuntimeSession};

pub(super) fn send_connection(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    gamepad: u64,
    connected: bool,
    name: &str,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
) -> Result<(), RuntimeLibraryError> {
    let state = if connected {
        ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1
    } else {
        ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1
    };
    let event = ZrRuntimeEventV1::gamepad_connection_with_ids(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        gamepad,
        state,
        vendor_id.map(u32::from).unwrap_or_default(),
        product_id.map(u32::from).unwrap_or_default(),
        byte_slice(name),
    );
    session.handle_event(event)
}

pub(super) fn send_button(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    gamepad: u64,
    button: Button,
    value: f32,
    pressed: bool,
) -> Result<(), RuntimeLibraryError> {
    let state = if pressed {
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1
    } else {
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1
    };
    let event = ZrRuntimeEventV1::gamepad_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        gamepad,
        button_code(button),
        state,
        value,
    );
    session.handle_event(event)
}

pub(super) fn send_raw_button(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    gamepad: u64,
    button: Button,
    value: f32,
) -> Result<(), RuntimeLibraryError> {
    let event = ZrRuntimeEventV1::gamepad_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        gamepad,
        button_code(button),
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        value,
    );
    session.handle_event(event)
}

pub(super) fn send_axis(
    session: &RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    gamepad: u64,
    axis: Axis,
    value: f32,
) -> Result<(), RuntimeLibraryError> {
    let event = ZrRuntimeEventV1::gamepad_axis(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        gamepad,
        axis_code(axis),
        value,
    );
    session.handle_event(event)
}

pub(super) fn gamepad_id(id: gilrs::GamepadId) -> u64 {
    let id: usize = id.into();
    id as u64
}

fn byte_slice(value: &str) -> ZrByteSlice {
    ZrByteSlice {
        data: value.as_bytes().as_ptr(),
        len: value.len(),
    }
}
