use zircon_runtime_interface::{
    ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeImeCursorAreaV1, ZrRuntimeImeHostRequestV1,
    ZrRuntimeImeSurroundingTextV1,
};

use crate::core::framework::input::{GamepadRumbleRequest, ImeHostRequest};

pub(in crate::dynamic_api) fn runtime_ime_host_request(
    request: ImeHostRequest,
) -> ZrRuntimeImeHostRequestV1 {
    match request {
        ImeHostRequest::Enable => ZrRuntimeImeHostRequestV1::enable(),
        ImeHostRequest::Disable => ZrRuntimeImeHostRequestV1::disable(),
        ImeHostRequest::SetCursorArea(area) => ZrRuntimeImeHostRequestV1::set_cursor_area(
            ZrRuntimeImeCursorAreaV1::new(area.x, area.y, area.width, area.height),
        ),
        ImeHostRequest::SetSurroundingText(text) => {
            ZrRuntimeImeHostRequestV1::set_surrounding_text(ZrRuntimeImeSurroundingTextV1::new(
                text.value,
                text.cursor,
                text.anchor,
            ))
        }
    }
}

pub(in crate::dynamic_api) fn runtime_gamepad_rumble_request(
    request: GamepadRumbleRequest,
) -> ZrRuntimeGamepadRumbleRequestV1 {
    match request {
        GamepadRumbleRequest::Add {
            gamepad,
            intensity,
            duration_millis,
        } => {
            let intensity = intensity.clamped();
            ZrRuntimeGamepadRumbleRequestV1::add(
                gamepad.0,
                intensity.strong_motor,
                intensity.weak_motor,
                duration_millis,
            )
        }
        GamepadRumbleRequest::Stop { gamepad } => ZrRuntimeGamepadRumbleRequestV1::stop(gamepad.0),
    }
}
