use zircon_runtime_interface::{
    ZrRuntimeCursorGrabModeV1, ZrRuntimeCursorHostRequestV1, ZrRuntimeCursorPositionV1,
    ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeImeCursorAreaV1, ZrRuntimeImeHostRequestV1,
    ZrRuntimeImeSurroundingTextV1, ZrRuntimeViewportHandle,
};

use crate::core::framework::input::{
    CursorGrabMode, CursorHostRequest, GamepadRumbleRequest, ImeHostRequest,
};

pub(in crate::dynamic_api) fn runtime_ime_host_request(
    request: ImeHostRequest,
    target_viewport: ZrRuntimeViewportHandle,
) -> ZrRuntimeImeHostRequestV1 {
    let request = match request {
        ImeHostRequest::Enable => ZrRuntimeImeHostRequestV1::enable(),
        ImeHostRequest::Disable => ZrRuntimeImeHostRequestV1::disable(),
        ImeHostRequest::SetCursorArea(area) => ZrRuntimeImeHostRequestV1::set_cursor_area(
            ZrRuntimeImeCursorAreaV1::new(area.x, area.y, area.width, area.height),
        ),
        ImeHostRequest::SetSurroundingText(text) => {
            ZrRuntimeImeHostRequestV1::set_surrounding_text(
                ZrRuntimeImeSurroundingTextV1::new(text.value, text.cursor, text.anchor)
                    .with_composition_range(text.composition_range.map(|range| {
                        zircon_runtime_interface::ZrRuntimeImeTextRangeV1::new(
                            range.start,
                            range.end,
                        )
                    })),
            )
        }
    };
    request.with_target_viewport(target_viewport)
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

pub(in crate::dynamic_api) fn runtime_cursor_host_request(
    request: CursorHostRequest,
) -> ZrRuntimeCursorHostRequestV1 {
    match request {
        CursorHostRequest::SetVisible(visible) => {
            ZrRuntimeCursorHostRequestV1::set_visible(visible)
        }
        CursorHostRequest::SetGrabMode(grab_mode) => {
            ZrRuntimeCursorHostRequestV1::set_grab_mode(runtime_cursor_grab_mode(grab_mode))
        }
        CursorHostRequest::SetHitTest(hit_test) => {
            ZrRuntimeCursorHostRequestV1::set_hit_test(hit_test)
        }
        CursorHostRequest::SetPosition(position) => ZrRuntimeCursorHostRequestV1::set_position(
            ZrRuntimeCursorPositionV1::new(position.x, position.y),
        ),
    }
}

fn runtime_cursor_grab_mode(grab_mode: CursorGrabMode) -> ZrRuntimeCursorGrabModeV1 {
    match grab_mode {
        CursorGrabMode::None => ZrRuntimeCursorGrabModeV1::None,
        CursorGrabMode::Confined => ZrRuntimeCursorGrabModeV1::Confined,
        CursorGrabMode::Locked => ZrRuntimeCursorGrabModeV1::Locked,
    }
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::{ZrRuntimeImeHostRequestKindV1, ZrRuntimeViewportHandle};

    use super::runtime_ime_host_request;
    use crate::core::framework::input::ImeHostRequest;

    #[test]
    fn runtime_ime_host_request_preserves_its_viewport_target() {
        let viewport = ZrRuntimeViewportHandle::new(7);
        let request = runtime_ime_host_request(ImeHostRequest::Enable, viewport);

        assert_eq!(request.kind, ZrRuntimeImeHostRequestKindV1::Enable);
        assert_eq!(request.target_viewport, Some(viewport));
    }
}
