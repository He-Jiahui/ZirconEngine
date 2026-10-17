use zircon_runtime_interface::ui::dispatch::{
    UiAnalogInputEvent, UiInputEvent, UiNavigationInputEvent,
};
use zircon_runtime_interface::ui::surface::UiNavigationEventKind;
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeGamepadGenerationV2, ZrStatus, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_X_V1,
    ZR_RUNTIME_GAMEPAD_AXIS_LEFT_STICK_Y_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_DOWN_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_LEFT_V1, ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_RIGHT_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_DPAD_UP_V1, ZR_RUNTIME_GAMEPAD_BUTTON_EAST_V1,
    ZR_RUNTIME_GAMEPAD_BUTTON_SOUTH_V1, ZR_RUNTIME_GAMEPAD_CONNECTION_CONNECTED_V1,
    ZR_RUNTIME_GAMEPAD_CONNECTION_DISCONNECTED_V1,
};

use crate::core::framework::input::{GamepadConnectionInfo, GamepadId, InputEvent, InputEventAdmission};

use super::super::input_events::{gamepad_axis, gamepad_button, nonzero_u16};
use super::super::status::{error_status, invalid_argument};
use super::super::RuntimeDynamicSession;
use super::event_payload;

impl RuntimeDynamicSession {
    fn gamepad_event_admitted(&self, event: InputEvent) -> Result<bool, ZrStatus> {
        match self.submit_input_event_with_admission(event) {
            Ok(InputEventAdmission::Accepted) => Ok(true),
            // The common helper diagnoses a delayed valid envelope, which is a safe no-op.
            // It never enters the independent UI branch or becomes a fatal App ABI failure.
            Ok(InputEventAdmission::Rejected(_)) => Ok(false),
            Err(error) => Err(error_status(format!("admit runtime gamepad input: {error}"))),
        }
    }

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
        let decoded = match ZrRuntimeGamepadGenerationV2::decode_connection(payload) {
            Ok(decoded) => decoded,
            Err(_) => return invalid_argument(b"invalid runtime gamepad generation V2 payload"),
        };
        match self.gamepad_event_admitted(InputEvent::GamepadConnection(GamepadConnectionInfo {
            gamepad: GamepadId(event.pointer_id),
            generation: decoded.generation,
            connected,
            name: decoded.name.map(str::to_owned),
            vendor_id: nonzero_u16(event.key_code),
            product_id: nonzero_u16(event.scan_code),
        })) {
            Ok(_) => ZrStatus::ok(),
            Err(status) => status,
        }
    }

    pub(super) fn handle_gamepad_button(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        let pressed = match event.state {
            ZR_RUNTIME_BUTTON_STATE_PRESSED_V1 => true,
            ZR_RUNTIME_BUTTON_STATE_RELEASED_V1 => false,
            _ => return invalid_argument(b"unknown runtime gamepad button state"),
        };
        if !event.delta.is_finite() {
            return invalid_argument(b"invalid runtime gamepad button value");
        }
        let generation = match gamepad_sample_generation(event) {
            Ok(generation) => generation,
            Err(status) => return status,
        };
        match self.gamepad_event_admitted(InputEvent::GamepadButton {
            gamepad: GamepadId(event.pointer_id),
            generation,
            button: gamepad_button(event.button),
            value: event.delta,
            pressed,
        }) {
            Ok(true) => {}
            Ok(false) => return ZrStatus::ok(),
            Err(status) => return status,
        }
        // Generation admission gates UI; UI keeps the existing raw button/navigation policy.
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
        if !event.delta.is_finite() {
            return invalid_argument(b"invalid runtime gamepad axis value");
        }
        let generation = match gamepad_sample_generation(event) {
            Ok(generation) => generation,
            Err(status) => return status,
        };
        match self.gamepad_event_admitted(InputEvent::GamepadAxis {
            gamepad: GamepadId(event.pointer_id),
            generation,
            axis: gamepad_axis(event.button),
            value: event.delta,
        }) {
            Ok(true) => {}
            Ok(false) => return ZrStatus::ok(),
            Err(status) => return status,
        }
        // Preserve existing raw UI analog values, rather than claiming filtered-value parity.
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

fn gamepad_sample_generation(event: ZrRuntimeEventV1) -> Result<std::num::NonZeroU64, ZrStatus> {
    let payload = event_payload(event)?;
    ZrRuntimeGamepadGenerationV2::decode_sample(payload)
        .map_err(|_| invalid_argument(b"invalid runtime gamepad generation V2 payload"))
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
