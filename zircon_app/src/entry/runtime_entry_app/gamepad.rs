use gilrs::{Axis, Button, EventType, GilrsBuilder};
use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_warn;
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_C_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1, ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1, ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1, ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1, ZR_RUNTIME_GAMEPAD_BUTTON_START_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1,
};

use super::RuntimeEntryApp;

pub(super) fn create_gilrs() -> Option<gilrs::Gilrs> {
    match GilrsBuilder::new()
        .with_default_filters(false)
        .set_update_state(false)
        .build()
    {
        Ok(gilrs) => Some(gilrs),
        Err(error) => {
            write_warn(
                "runtime_gamepad",
                format!("runtime_gamepad_gilrs_unavailable: {error}"),
            );
            None
        }
    }
}

impl RuntimeEntryApp {
    pub(super) fn poll_gamepads(&mut self, event_loop: &dyn ActiveEventLoop) {
        if self.gamepads.is_none() {
            return;
        }
        if !self.gamepad_connections_announced {
            self.gamepad_connections_announced = true;
            if !self.announce_connected_gamepads(event_loop) {
                return;
            }
        }

        let session = &self.session;
        let viewport = self.viewport;
        let Some(gamepads) = self.gamepads.as_mut() else {
            return;
        };
        while let Some(event) = gamepads.next_event() {
            gamepads.update(&event);
            let result = match event.event {
                EventType::Connected => {
                    let pad = gamepads.gamepad(event.id);
                    send_connection(
                        session,
                        viewport,
                        gamepad_id(event.id),
                        true,
                        pad.name(),
                        pad.vendor_id(),
                        pad.product_id(),
                    )
                }
                EventType::Disconnected => send_connection(
                    session,
                    viewport,
                    gamepad_id(event.id),
                    false,
                    "",
                    None,
                    None,
                ),
                EventType::ButtonPressed(button, _) | EventType::ButtonRepeated(button, _) => {
                    send_button(session, viewport, gamepad_id(event.id), button, 1.0, true)
                }
                EventType::ButtonReleased(button, _) => {
                    send_button(session, viewport, gamepad_id(event.id), button, 0.0, false)
                }
                EventType::ButtonChanged(button, value, _) => send_button(
                    session,
                    viewport,
                    gamepad_id(event.id),
                    button,
                    value,
                    value >= 0.5,
                ),
                EventType::AxisChanged(axis, value, _) => {
                    send_axis(session, viewport, gamepad_id(event.id), axis, value)
                }
                EventType::Dropped | EventType::ForceFeedbackEffectCompleted => Ok(()),
                _ => Ok(()),
            };
            if result.is_err() {
                event_loop.exit();
                return;
            }
        }
        gamepads.inc();
    }

    fn announce_connected_gamepads(&mut self, event_loop: &dyn ActiveEventLoop) -> bool {
        let session = &self.session;
        let viewport = self.viewport;
        let Some(gamepads) = self.gamepads.as_mut() else {
            return true;
        };
        for (id, pad) in gamepads.gamepads() {
            if send_connection(
                session,
                viewport,
                gamepad_id(id),
                true,
                pad.name(),
                pad.vendor_id(),
                pad.product_id(),
            )
            .is_err()
            {
                event_loop.exit();
                return false;
            }
        }
        true
    }
}

fn send_connection(
    session: &crate::entry::runtime_library::RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    gamepad: u64,
    connected: bool,
    name: &str,
    vendor_id: Option<u16>,
    product_id: Option<u16>,
) -> Result<(), crate::entry::runtime_library::RuntimeLibraryError> {
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

fn send_button(
    session: &crate::entry::runtime_library::RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    gamepad: u64,
    button: Button,
    value: f32,
    pressed: bool,
) -> Result<(), crate::entry::runtime_library::RuntimeLibraryError> {
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

fn send_axis(
    session: &crate::entry::runtime_library::RuntimeSession,
    viewport: ZrRuntimeViewportHandle,
    gamepad: u64,
    axis: Axis,
    value: f32,
) -> Result<(), crate::entry::runtime_library::RuntimeLibraryError> {
    let event = ZrRuntimeEventV1::gamepad_axis(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        viewport,
        gamepad,
        axis_code(axis),
        value,
    );
    session.handle_event(event)
}

fn byte_slice(value: &str) -> ZrByteSlice {
    ZrByteSlice {
        data: value.as_bytes().as_ptr(),
        len: value.len(),
    }
}

fn gamepad_id(id: gilrs::GamepadId) -> u64 {
    let id: usize = id.into();
    id as u64
}

fn button_code(button: Button) -> u32 {
    match button {
        Button::South => ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1,
        Button::East => ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1,
        Button::North => ZR_RUNTIME_GAMEPAD_BUTTON_NORTH_V1,
        Button::West => ZR_RUNTIME_GAMEPAD_BUTTON_WEST_V1,
        Button::C => ZR_RUNTIME_GAMEPAD_BUTTON_C_V1,
        Button::Z => ZR_RUNTIME_GAMEPAD_BUTTON_Z_V1,
        Button::LeftTrigger => ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER_V1,
        Button::LeftTrigger2 => ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_TRIGGER2_V1,
        Button::RightTrigger => ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER_V1,
        Button::RightTrigger2 => ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_TRIGGER2_V1,
        Button::Select => ZR_RUNTIME_GAMEPAD_BUTTON_SELECT_V1,
        Button::Start => ZR_RUNTIME_GAMEPAD_BUTTON_START_V1,
        Button::Mode => ZR_RUNTIME_GAMEPAD_BUTTON_MODE_V1,
        Button::LeftThumb => ZR_RUNTIME_GAMEPAD_BUTTON_LEFT_THUMB_V1,
        Button::RightThumb => ZR_RUNTIME_GAMEPAD_BUTTON_RIGHT_THUMB_V1,
        Button::DPadUp => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
        Button::DPadDown => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1,
        Button::DPadLeft => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
        Button::DPadRight => ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
        Button::Unknown => ZR_RUNTIME_GAMEPAD_BUTTON_UNKNOWN_V1,
    }
}

fn axis_code(axis: Axis) -> u32 {
    match axis {
        Axis::LeftStickX => ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
        Axis::LeftStickY => ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
        Axis::LeftZ => ZR_RUNTIME_GAMEPAD_AXIS_LEFT_Z_V1,
        Axis::RightStickX => ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_X_V1,
        Axis::RightStickY => ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_STICK_Y_V1,
        Axis::RightZ => ZR_RUNTIME_GAMEPAD_AXIS_RIGHT_Z_V1,
        Axis::DPadX => ZR_RUNTIME_GAMEPAD_AXIS_DPAD_X_V1,
        Axis::DPadY => ZR_RUNTIME_GAMEPAD_AXIS_DPAD_Y_V1,
        Axis::Unknown => ZR_RUNTIME_GAMEPAD_AXIS_UNKNOWN_V1,
    }
}
