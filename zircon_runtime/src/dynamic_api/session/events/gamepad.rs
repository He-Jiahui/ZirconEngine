use zircon_runtime_interface::ui::dispatch::{
    UiAnalogInputEvent, UiInputEvent, UiNavigationInputEvent,
};
use zircon_runtime_interface::ui::surface::UiNavigationEventKind;
use zircon_runtime_interface::{
    ZR_RUNTIME_BUTTON_STATE_PRESSED_V1, ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1, ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1,
    ZrRuntimeEventV1, ZrStatus,
};

use crate::core::framework::input::{GamepadConnectionInfo, GamepadId, InputEvent};

use super::super::RuntimeDynamicSession;
use super::super::input_events::{gamepad_axis, gamepad_button, nonzero_u16};
use super::super::status::invalid_argument;
use super::event_payload;

impl RuntimeDynamicSession {
    pub(super) fn handle_gamepad_connection(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let connected = match event.state {
            ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1 => true,
            ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1 => false,
            _ => return invalid_argument(b"unknown runtime gamepad connection state"),
        };
        let payload = match event_payload(event) {
            Ok(payload) => payload,
            Err(status) => return status,
        };
        let name = if payload.is_empty() {
            None
        } else {
            String::from_utf8(payload.to_vec()).ok()
        };
        self.submit_input_event(InputEvent::GamepadConnection(GamepadConnectionInfo {
            gamepad: GamepadId(event.pointer_id),
            connected,
            name,
            vendor_id: nonzero_u16(event.key_code),
            product_id: nonzero_u16(event.scan_code),
        }));
        ZrStatus::ok()
    }

    pub(super) fn handle_gamepad_button(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let pressed = match event.state {
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => true,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => false,
            _ => return invalid_argument(b"unknown runtime gamepad button state"),
        };
        self.submit_input_event(InputEvent::GamepadButton {
            gamepad: GamepadId(event.pointer_id),
            button: gamepad_button(event.button),
            value: event.delta,
            pressed,
        });
        if pressed {
            if let Some(kind) = ui_gamepad_navigation(event.button) {
                match self.dispatch_runtime_ui_event(|metadata| {
                    UiInputEvent::Navigation(UiNavigationInputEvent { metadata, kind })
                }) {
                    Ok(true) => return ZrStatus::ok(),
                    Ok(false) => {}
                    Err(status) => return status,
                }
            }
        }
        ZrStatus::ok()
    }

    pub(super) fn handle_gamepad_axis(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        self.submit_input_event(InputEvent::GamepadAxis {
            gamepad: GamepadId(event.pointer_id),
            axis: gamepad_axis(event.button),
            value: event.delta,
        });
        if let Some(control) = ui_gamepad_analog_control(event.button) {
            match self.dispatch_runtime_ui_event(|metadata| {
                UiInputEvent::Analog(UiAnalogInputEvent {
                    metadata,
                    control: control.to_string(),
                    value: event.delta,
                })
            }) {
                Ok(true) => return ZrStatus::ok(),
                Ok(false) => {}
                Err(status) => return status,
            }
        }
        ZrStatus::ok()
    }
}

pub(super) fn ui_gamepad_navigation(button: u32) -> Option<UiNavigationEventKind> {
    match button {
        ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1 => Some(UiNavigationEventKind::Activate),
        ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1 => Some(UiNavigationEventKind::Cancel),
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1 => Some(UiNavigationEventKind::Up),
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1 => Some(UiNavigationEventKind::Down),
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1 => Some(UiNavigationEventKind::Left),
        ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1 => Some(UiNavigationEventKind::Right),
        _ => None,
    }
}

pub(super) fn ui_gamepad_analog_control(axis: u32) -> Option<&'static str> {
    match axis {
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1 => Some("gamepad_left_stick_x"),
        ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1 => Some("gamepad_left_stick_y"),
        _ => None,
    }
}
