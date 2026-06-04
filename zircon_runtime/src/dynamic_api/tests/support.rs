pub(super) use zircon_runtime_interface::{
    ui::{
        accessibility::{
            UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityActionSource,
            UiAccessibilityTreeSnapshot,
        },
        event_ui::UiNodeId,
    },
    ZrByteSlice, ZrHostApiV1, ZrOwnedByteBuffer, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeFrameV1, ZrRuntimeGamepadRumbleRequestKindV1, ZrRuntimeGamepadRumbleRequestV1,
    ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZrRuntimeImeHostRequestKindV1,
    ZrRuntimeNativeSurfaceTargetV1, ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
};

pub(super) use crate::core::framework::input::{
    GamepadId, GamepadRumbleIntensity, GamepadRumbleRequest, ImeCursorArea, ImeHostRequest,
    ImeSurroundingText,
};

pub(super) use super::super::{
    frame::{
        encode_host_request_batch, free_runtime_accessibility_bytes,
        free_runtime_host_request_bytes,
    },
    session::{runtime_gamepad_rumble_request, runtime_ime_host_request},
    zircon_runtime_get_api_v1,
};

pub(super) fn runtime_api() -> &'static zircon_runtime_interface::ZrRuntimeApiV1 {
    unsafe { &*zircon_runtime_get_api_v1(core::ptr::null()) }
}

pub(super) fn accessibility_tree_request(
    abi_version: u32,
    viewport: u64,
) -> ZrRuntimeAccessibilityTreeRequestV1 {
    ZrRuntimeAccessibilityTreeRequestV1::new(
        abi_version,
        ZrRuntimeViewportHandle::new(viewport),
        ZrRuntimeViewportSizeV1::new(64, 48),
        7,
    )
}

pub(super) fn create_test_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV1,
) -> ZrRuntimeSessionHandle {
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1),
            &mut session,
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    session
}

pub(super) fn destroy_test_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV1,
    session: ZrRuntimeSessionHandle,
) {
    let destroy_session = api.destroy_session.expect("destroy_session");
    let status = unsafe { destroy_session(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

pub(super) fn status_message(status: ZrStatus) -> String {
    String::from_utf8(unsafe { status.diagnostics.as_slice() }.to_vec()).unwrap()
}

pub(super) fn host_request_batch_from_output(
    output: ZrOwnedByteBuffer,
) -> ZrRuntimeHostRequestBatchV1 {
    let bytes = unsafe { core::slice::from_raw_parts(output.data as *const u8, output.len) };
    let batch = serde_json::from_slice(bytes).unwrap();
    free_host_request_output(output);
    batch
}

pub(super) fn free_output(output: ZrOwnedByteBuffer) {
    let free = output.free.expect("free accessibility output");
    let status = unsafe { free(output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

pub(super) fn free_profile_output(output: ZrOwnedByteBuffer) {
    let free = output.free.expect("free profile output");
    let status = unsafe { free(output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

fn free_host_request_output(output: ZrOwnedByteBuffer) {
    let free = output.free.expect("free host request output");
    let status = unsafe { free(output) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}
