pub(super) use zircon_runtime_interface::{
    ui::{
        accessibility::{
            UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityActionSource,
            UiAccessibilityTreeSnapshot,
        },
        event_ui::UiNodeId,
    },
    ZrByteSlice, ZrHostApiV1, ZrOwnedResultV2, ZrRuntimeAccessibilityTreeRequestV1,
    ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeCursorGrabModeV1,
    ZrRuntimeCursorHostRequestKindV1, ZrRuntimeCursorHostRequestV1, ZrRuntimeEventV1,
    ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2,
    ZrRuntimeGamepadRumbleRequestKindV1, ZrRuntimeGamepadRumbleRequestV1,
    ZrRuntimeHostRequestBatchV1, ZrRuntimeHostRequestV1, ZrRuntimeImeCursorAreaV1,
    ZrRuntimeImeHostRequestKindV1, ZrRuntimeImeTextRangeV1, ZrRuntimeNativeSurfaceTargetV1,
    ZrRuntimeSessionConfigV3, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrRuntimeWakeSinkV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZIRCON_RUNTIME_ABI_VERSION_V2, ZIRCON_RUNTIME_ABI_VERSION_V3,
    ZIRCON_RUNTIME_API_VERSION_V8, ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1,
};

pub(super) use crate::core::framework::input::{
    CursorGrabMode, CursorHostRequest, GamepadId, GamepadRumbleIntensity, GamepadRumbleRequest,
    ImeCursorArea, ImeCursorRange, ImeHostRequest, ImeSurroundingText,
};

pub(super) use super::super::{
    frame::encode_host_request_batch,
    session::{
        runtime_cursor_host_request, runtime_gamepad_rumble_request, runtime_ime_host_request,
    },
    zircon_runtime_get_api_v8,
};

pub(super) fn runtime_api() -> &'static zircon_runtime_interface::ZrRuntimeApiV8 {
    unsafe { &*zircon_runtime_get_api_v8(core::ptr::null()) }
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
    api: &zircon_runtime_interface::ZrRuntimeApiV8,
) -> ZrRuntimeSessionHandle {
    create_test_session_with_profile(api, b"headless")
}

pub(super) fn create_test_session_with_profile(
    api: &zircon_runtime_interface::ZrRuntimeApiV8,
    profile: &'static [u8],
) -> ZrRuntimeSessionHandle {
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV3 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                profile: ZrByteSlice::from_static(profile),
                project_root: ZrByteSlice::empty(),
                play_scene: ZrByteSlice::empty(),
                play_report_pipe: ZrByteSlice::empty(),
                wake_sink: ZrRuntimeWakeSinkV1::disabled(),
            },
            &mut session,
        )
    };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
    session
}

pub(super) fn destroy_test_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV8,
    session: ZrRuntimeSessionHandle,
) {
    let destroy_session = api.destroy_session.expect("destroy_session");
    let status = unsafe { destroy_session(session) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

pub(super) fn status_message(status: ZrStatus) -> String {
    String::from_utf8(
        unsafe {
            status.diagnostics.checked_slice(
                zircon_runtime_interface::ZR_RUNTIME_STATUS_DIAGNOSTICS_MAX_ENCODED_BYTES_V1,
            )
        }
        .unwrap()
        .to_vec(),
    )
    .unwrap()
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
    session: ZrRuntimeSessionHandle,
    output: ZrOwnedResultV2,
) -> ZrRuntimeHostRequestBatchV1 {
    let len = usize::try_from(output.len).expect("runtime output length fits host address space");
    let bytes = unsafe { core::slice::from_raw_parts(output.data, len) };
    let batch = serde_json::from_slice(bytes).unwrap();
    release_output(session, output);
    batch
}

pub(super) fn host_request_batch_from_bytes(bytes: &[u8]) -> ZrRuntimeHostRequestBatchV1 {
    serde_json::from_slice(bytes).unwrap()
}

pub(super) fn release_output(session: ZrRuntimeSessionHandle, output: ZrOwnedResultV2) {
    let release = runtime_api()
        .release_allocation
        .expect("release runtime allocation");
    let status = unsafe { release(session, output.allocation) };
    assert_eq!(status.status_code(), ZrStatusCode::Ok, "{status:?}");
}

pub(super) fn output_bytes(output: &ZrOwnedResultV2) -> &[u8] {
    let len = usize::try_from(output.len).expect("runtime output length fits host address space");
    unsafe { core::slice::from_raw_parts(output.data, len) }
}
