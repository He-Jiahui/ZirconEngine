//! Runtime 宿主请求批次的已知有效 JSON 往返及 ABI 字段测试。
//!
//! 夹具拥有请求、编码字节和回读值；断言覆盖 IME、震动与光标转换，不分配原生输出缓冲或执行宿主设备操作。

use super::support::*;

#[test]
/// 验证 IME 宿主请求批次经过有界 JSON 编码后保留 ABI 版本、变体、视口和组合区间。
///
/// # 使用时机
/// 构造三种 [ZrRuntimeHostRequestV1] IME 请求，调 [encode_host_request_batch] 编码，再用 [host_request_batch_from_bytes] 解码。
///
/// # 返回与限制
/// 测试返回 ()。通过证明请求可回读并保留视口、区域宽度、文本及组合范围；不覆盖超限、畸形 JSON 或释放失败。
///
/// # 参考
/// [ZrRuntimeHostRequestBatchV1]、[ZrRuntimeHostRequestV1]、[ZrRuntimeImeHostRequestKindV1]、[encode_host_request_batch]、[host_request_batch_from_bytes]
fn host_request_batch_encodes_runtime_ime_requests() {
    let viewport = default_viewport();
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(ImeHostRequest::Enable, viewport)),
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(
                ImeHostRequest::SetCursorArea(ImeCursorArea::new(16.0, 24.0, 8.0, 18.0)),
                viewport,
            )),
            ZrRuntimeHostRequestV1::ime(runtime_ime_host_request(
                ImeHostRequest::SetSurroundingText(
                    ImeSurroundingText::new("search", 6, 0)
                        .with_composition_range(Some(ImeCursorRange::new(1, 6))),
                ),
                viewport,
            )),
        ],
    );

    let output = encode_host_request_batch(&batch).unwrap();
    let batch = host_request_batch_from_bytes(&output);

    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 3);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::Enable
                && request.target_viewport == Some(viewport)
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetCursorArea
                && request.cursor_area.as_ref().map(|area| area.width) == Some(8.0)
                && request.target_viewport == Some(viewport)
    ));
    assert!(matches!(
        batch.requests[2],
        ZrRuntimeHostRequestV1::Ime(ref request)
            if request.kind == ZrRuntimeImeHostRequestKindV1::SetSurroundingText
                && request
                    .surrounding_text
                    .as_ref()
                    .map(|text| (text.value.as_str(), text.composition_range))
                    == Some(("search", Some(ZrRuntimeImeTextRangeV1::new(1, 6))))
                && request.target_viewport == Some(viewport)
    ));
}

#[test]
/// 验证手柄震动 Add/Stop 在 Runtime ABI 批次中的强弱马达归一化、持续时间和变体往返。
///
/// # 使用时机
/// 构造同一手柄、已固定入队代际的 Add/Stop，经 [runtime_gamepad_rumble_request] 放入独立 V2 variant，再编码和回读。
///
/// # 返回与限制
/// 测试返回 ()。Add 越界强弱值回读为 1.0/0.0，Stop 携带零强度和零时长；通过不证明宿主执行设备效果。
///
/// # 参考
/// [ZrRuntimeHostRequestBatchV1]、[ZrRuntimeHostRequestV1]、[zircon_runtime_interface::ZrRuntimeGamepadRumbleRequestV2]、[ZrRuntimeGamepadRumbleRequestKindV1]、[runtime_gamepad_rumble_request]、[encode_host_request_batch]、[host_request_batch_from_bytes]
fn host_request_batch_encodes_gamepad_rumble_requests() {
    use std::num::NonZeroU64;
    use crate::core::framework::input::QueuedGamepadRumbleRequest;
    use zircon_runtime_interface::ZrRuntimeGamepadRumbleRequestV2;

    let generation = NonZeroU64::new(3).unwrap();
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![
            ZrRuntimeHostRequestV1::gamepad_rumble_v2(runtime_gamepad_rumble_request(
                QueuedGamepadRumbleRequest::new(
                    GamepadRumbleRequest::add(
                        GamepadId(7),
                        GamepadRumbleIntensity::new(1.25, -0.5),
                        250,
                    ),
                    generation,
                ),
            )),
            ZrRuntimeHostRequestV1::gamepad_rumble_v2(runtime_gamepad_rumble_request(
                QueuedGamepadRumbleRequest::new(GamepadRumbleRequest::stop(GamepadId(7)), generation),
            )),
        ],
    );

    let output = encode_host_request_batch(&batch).unwrap();
    let batch = host_request_batch_from_bytes(&output);

    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 2);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::GamepadRumbleV2(ZrRuntimeGamepadRumbleRequestV2 {
            gamepad_id: 7,
            generation: observed,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Add,
            strong_motor: 1.0,
            weak_motor: 0.0,
            duration_millis: 250,
        }) if observed == generation
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::GamepadRumbleV2(ZrRuntimeGamepadRumbleRequestV2 {
            gamepad_id: 7,
            generation: observed,
            kind: ZrRuntimeGamepadRumbleRequestKindV1::Stop,
            strong_motor: 0.0,
            weak_motor: 0.0,
            duration_millis: 0,
        }) if observed == generation
    ));
}

#[test]
/// 验证光标可见性、抓取模式、命中测试和位置请求在 Runtime ABI 批次中的变体与字段。
///
/// # 使用时机
/// 构造四个 [CursorHostRequest]，经 [runtime_cursor_host_request] 放入 [ZrRuntimeHostRequestBatchV1]，编码后回读。
///
/// # 返回与限制
/// 测试返回 ()。通过证明顺序、版本、变体和位置/布尔字段保持；不覆盖未知枚举、超限输出或宿主应用。
///
/// # 参考
/// [CursorHostRequest]、[ZrRuntimeHostRequestBatchV1]、[ZrRuntimeHostRequestV1]、[ZrRuntimeCursorHostRequestV1]、[ZrRuntimeCursorGrabModeV1]、[runtime_cursor_host_request]、[encode_host_request_batch]、[host_request_batch_from_bytes]
fn host_request_batch_encodes_cursor_requests() {
    let batch = ZrRuntimeHostRequestBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_visible(false),
            )),
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_grab_mode(CursorGrabMode::Locked),
            )),
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_hit_test(false),
            )),
            ZrRuntimeHostRequestV1::cursor(runtime_cursor_host_request(
                CursorHostRequest::set_position(320.0, 180.0),
            )),
        ],
    );

    let output = encode_host_request_batch(&batch).unwrap();
    let batch = host_request_batch_from_bytes(&output);

    assert_eq!(batch.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(batch.requests.len(), 4);
    assert!(matches!(
        batch.requests[0],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetVisible,
            value: false,
            ..
        })
    ));
    assert!(matches!(
        batch.requests[1],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetGrabMode,
            grab_mode: Some(ZrRuntimeCursorGrabModeV1::Locked),
            ..
        })
    ));
    assert!(matches!(
        batch.requests[2],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetHitTest,
            value: false,
            ..
        })
    ));
    assert!(matches!(
        batch.requests[3],
        ZrRuntimeHostRequestV1::Cursor(ZrRuntimeCursorHostRequestV1 {
            kind: ZrRuntimeCursorHostRequestKindV1::SetPosition,
            position: Some(position),
            ..
        }) if position.x == 320.0 && position.y == 180.0
    ));
}
