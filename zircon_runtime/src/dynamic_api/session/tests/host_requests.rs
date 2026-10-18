use zircon_runtime_interface::{
    ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZrRuntimeImeHostRequestKindV1,
    ZrRuntimeViewportHandle, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZR_RUNTIME_HOST_REQUEST_OUTPUT_LIMIT_V1,
};

use super::{runtime_ime_host_request, RUNTIME_IME_SURROUNDING_TEXT_MAX_BYTES};
use crate::core::framework::input::{ImeCursorRange, ImeHostRequest, ImeSurroundingText};
use crate::dynamic_api::frame::{encode_host_request_batch, encode_host_request_page};

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

#[test]
fn borrowed_host_request_page_matches_the_owned_v1_batch_encoding() {
    let requests = vec![ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(
        ImeHostRequest::Enable,
        ZrRuntimeViewportHandle::new(7),
    ))];
    let owned = encode_host_request_batch(&ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        requests.clone(),
    ))
    .expect("owned host request batch");
    let borrowed = encode_host_request_page(ZIRCON_RUNTIME_ABI_VERSION_V1, &requests)
        .expect("borrowed host request page");

    assert_eq!(borrowed, owned);
}

#[test]
fn encoded_rumble_page_keeps_enqueue_generation_after_reconnect() {
    use std::num::NonZeroU64;

    use crate::core::framework::input::{
        GamepadConnectionInfo, GamepadId, GamepadRumbleIntensity, GamepadRumbleRequest, InputEvent,
        InputManager,
    };
    use crate::input::DefaultInputManager;
    use zircon_runtime_interface::ZrRuntimeGamepadRumbleRequestKindV1;

    let input = DefaultInputManager::default();
    let gamepad = GamepadId(61);
    let first = NonZeroU64::new(1).unwrap();
    let second = NonZeroU64::new(2).unwrap();
    let connection = |generation, connected| {
        InputEvent::GamepadConnection(GamepadConnectionInfo {
            gamepad,
            generation,
            connected,
            name: None,
            vendor_id: None,
            product_id: None,
        })
    };
    let add = GamepadRumbleRequest::add(gamepad, GamepadRumbleIntensity::new(1.25, -0.5), 250);
    let stop = GamepadRumbleRequest::stop(gamepad);
    input.submit_event(connection(first, true));
    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    input.submit_event(InputEvent::GamepadRumbleRequest(stop));
    let requests = input
        .drain_gamepad_rumble_requests()
        .into_iter()
        .map(super::runtime_gamepad_rumble_request)
        .map(ZrRuntimeHostRequestV1::gamepad_rumble_v2)
        .collect::<Vec<_>>();
    let owned = ZrRuntimeHostRequestBatchV1::new(ZIRCON_RUNTIME_ABI_VERSION_V1, requests);
    let encoded = encode_host_request_batch(&owned).expect("generation-bound owned page");
    assert_eq!(
        encode_host_request_page(owned.abi_version, &owned.requests).unwrap(),
        encoded,
    );

    input.submit_event(connection(first, false));
    input.submit_event(connection(second, true));
    input.submit_event(InputEvent::GamepadRumbleRequest(add));
    // An already prepared/encoded owned page has no dependency on the new connection.
    assert_eq!(
        encode_host_request_page(owned.abi_version, &owned.requests).unwrap(),
        encoded,
    );
    let decoded: ZrRuntimeHostRequestBatchV1 = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded.requests.len(), 2);
    let mut kinds = Vec::new();
    for request in decoded.requests {
        let ZrRuntimeHostRequestV1::GamepadRumbleV2(request) = request else {
            panic!("rumble page must use the distinct V2 wire variant");
        };
        assert_eq!(request.gamepad_id, gamepad.0);
        assert_eq!(request.generation, first);
        kinds.push(request.kind);
    }
    assert_eq!(
        kinds,
        vec![
            ZrRuntimeGamepadRumbleRequestKindV1::Add,
            ZrRuntimeGamepadRumbleRequestKindV1::Stop
        ],
    );
    let next = input.drain_gamepad_rumble_requests();
    assert_eq!(next.len(), 1);
    assert_eq!(
        super::runtime_gamepad_rumble_request(next[0]).generation,
        second
    );
    assert!(input.drain_gamepad_rumble_requests().is_empty());
}
