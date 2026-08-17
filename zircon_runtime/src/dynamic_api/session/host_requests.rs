use zircon_runtime_interface::{
    ZrRuntimeCursorGrabModeV1, ZrRuntimeCursorHostRequestV1, ZrRuntimeCursorPositionV1,
    ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeImeCursorAreaV1, ZrRuntimeImeHostRequestV1,
    ZrRuntimeImeSurroundingTextV1, ZrRuntimeViewportHandle,
};

use crate::core::framework::input::{
    CursorGrabMode, CursorHostRequest, GamepadRumbleRequest, ImeHostRequest, ImeSurroundingText,
};

// JSON control characters can expand to six bytes. Keeping the source window at 32 KiB leaves
// ample room for the request envelope under the 256 KiB host-output ceiling.
const RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES: usize = 32 * 1024;

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
            let text = bounded_ime_surrounding_text(text);
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

fn bounded_ime_surrounding_text(mut text: ImeSurroundingText) -> ImeSurroundingText {
    if text.value.len() <= RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES {
        return text;
    }

    let source_len = text.value.len();
    let cursor = floor_char_boundary(&text.value, text.cursor.min(source_len));
    let anchor = floor_char_boundary(&text.value, text.anchor.min(source_len));
    let mut start = floor_char_boundary(
        &text.value,
        cursor.saturating_sub(RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES / 2),
    );
    let mut end = floor_char_boundary(
        &text.value,
        start
            .saturating_add(RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES)
            .min(source_len),
    );
    if end == source_len && end.saturating_sub(start) < RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES {
        start = floor_char_boundary(
            &text.value,
            end.saturating_sub(RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES),
        );
        end = floor_char_boundary(
            &text.value,
            start
                .saturating_add(RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES)
                .min(source_len),
        );
    }

    let remap = |offset: usize| offset.clamp(start, end).saturating_sub(start);
    text.cursor = remap(cursor);
    text.anchor = remap(anchor);
    text.composition_range = text.composition_range.map(|range| {
        crate::core::framework::input::ImeCursorRange::new(
            remap(floor_char_boundary(
                &text.value,
                range.start.min(source_len),
            )),
            remap(floor_char_boundary(&text.value, range.end.min(source_len))),
        )
    });
    text.value = text.value[start..end].to_owned();
    text
}

fn floor_char_boundary(value: &str, mut offset: usize) -> usize {
    while offset > 0 && !value.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
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
    use zircon_runtime_interface::{
        ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZrRuntimeImeHostRequestKindV1,
        ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1,
    };

    use super::{runtime_ime_host_request, RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES};
    use crate::core::framework::input::{ImeCursorRange, ImeHostRequest, ImeSurroundingText};
    use crate::dynamic_api::frame::encode_host_request_batch;

    #[test]
    fn runtime_ime_host_request_preserves_its_viewport_target() {
        let viewport = ZrRuntimeViewportHandle::new(7);
        let request = runtime_ime_host_request(ImeHostRequest::Enable, viewport);

        assert_eq!(request.kind, ZrRuntimeImeHostRequestKindV1::Enable);
        assert_eq!(request.target_viewport, Some(viewport));
    }

    #[test]
    fn runtime_ime_host_request_bounds_and_remaps_its_utf8_context_window() {
        let prefix = "\0".repeat(RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES);
        let value = format!("{prefix}界tail");
        let cursor = value.len() - "tail".len();
        let request = runtime_ime_host_request(
            ImeHostRequest::SetSurroundingText(
                ImeSurroundingText::new(value, cursor, 0)
                    .with_composition_range(Some(ImeCursorRange::new(cursor, cursor + 3))),
            ),
            ZrRuntimeViewportHandle::new(7),
        );
        let text = request
            .surrounding_text
            .as_ref()
            .expect("bounded surrounding-text request");

        assert!(text.value.len() <= RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES);
        assert!(text.value.is_char_boundary(text.cursor));
        assert!(text.cursor <= text.value.len());
        assert!(text.anchor <= text.value.len());
        assert!(text
            .composition_range
            .is_some_and(|range| range.start <= range.end && range.end <= text.value.len()));

        let bytes = encode_host_request_batch(&ZrRuntimeHostRequestBatchV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            vec![ZrRuntimeHostRequestV1::ime(request)],
        ))
        .expect("producer-bounded IME request must fit one host-output page");
        assert!(bytes.len() <= ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1.max_encoded_bytes);
    }
}
