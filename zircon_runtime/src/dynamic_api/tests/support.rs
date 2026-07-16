pub(super) use zircon_runtime_interface::{
    ui::{
        accessibility::{
            UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityActionSource,
            UiAccessibilityTreeSnapshot,
        },
        event_ui::UiNodeId,
    },
    ZrByteSlice, ZrHostApiV1, ZrOwnedByteBuffer, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeCursorGrabModeV1,
    ZrRuntimeCursorHostRequestKindV1, ZrRuntimeCursorHostRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeGamepadRumbleRequestKindV1,
    ZrRuntimeGamepadRumbleRequestV1, ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1,
    ZrRuntimeImeCursorAreaV1, ZrRuntimeImeHostRequestKindV1, ZrRuntimeNativeSurfaceTargetV1,
    ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_API_VERSION_V2, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
};

pub(super) use crate::core::framework::input::{
    CursorGrabMode, CursorHostRequest, GamepadId, GamepadRumbleIntensity, GamepadRumbleRequest,
    ImeCursorArea, ImeHostRequest, ImeSurroundingText,
};

pub(super) use super::super::{
    frame::{
        encode_host_request_batch, free_runtime_accessibility_bytes,
        free_runtime_host_request_bytes,
    },
    session::{
        runtime_cursor_host_request, runtime_gamepad_rumble_request, runtime_ime_host_request,
    },
    zircon_runtime_get_api_v2,
};

pub(super) fn runtime_api() -> &'static zircon_runtime_interface::ZrRuntimeApiV2 {
    unsafe { &*zircon_runtime_get_api_v2(core::ptr::null()) }
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
    api: &zircon_runtime_interface::ZrRuntimeApiV2,
) -> ZrRuntimeSessionHandle {
    create_test_session_with_profile(api, b"headless")
}

pub(super) fn create_test_session_with_profile(
    api: &zircon_runtime_interface::ZrRuntimeApiV2,
    profile: &'static [u8],
) -> ZrRuntimeSessionHandle {
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                profile: ZrByteSlice::from_static(profile),
                project_manifest: ZrByteSlice::empty(),
            },
            &mut session,
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    session
}

pub(super) fn destroy_test_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV2,
    session: ZrRuntimeSessionHandle,
) {
    let destroy_session = api.destroy_session.expect("destroy_session");
    let status = unsafe { destroy_session(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

pub(super) fn status_message(status: ZrStatus) -> String {
    String::from_utf8(unsafe { status.diagnostics.as_slice() }.to_vec()).unwrap()
}

pub(super) fn assert_session_status(
    status: ZrStatus,
    expected_code: ZrStatusCode,
    expected_message: &str,
) {
    assert_eq!(status.status_code(), expected_code, "{status:?}");
    assert_eq!(status_message(status), expected_message);
}

pub(super) fn valid_viewport_resize_event() -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::viewport_resized(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        default_viewport(),
        valid_viewport_size(),
    )
}

pub(super) fn valid_frame_request() -> ZrRuntimeFrameRequestV1 {
    ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        default_viewport(),
        valid_viewport_size(),
    )
}

pub(super) fn valid_bind_viewport_surface_request() -> ZrRuntimeBindViewportSurfaceRequestV1 {
    ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        default_viewport(),
        valid_viewport_size(),
        ZrRuntimeNativeSurfaceTargetV1::win32(ZIRCON_RUNTIME_ABI_VERSION_V1, 1, 0),
    )
}

pub(super) fn valid_profile_control_request_bytes() -> Vec<u8> {
    serde_json::to_vec(&zircon_runtime_interface::ProfileControlRequest {
        command: zircon_runtime_interface::ProfileControlCommand::Snapshot,
        config: None,
    })
    .unwrap()
}

pub(super) fn default_viewport() -> ZrRuntimeViewportHandle {
    ZrRuntimeViewportHandle::new(1)
}

pub(super) fn valid_viewport_size() -> ZrRuntimeViewportSizeV1 {
    ZrRuntimeViewportSizeV1::new(64, 48)
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
