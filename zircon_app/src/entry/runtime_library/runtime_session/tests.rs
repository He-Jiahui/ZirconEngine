use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};

use super::{
    ensure_status_releasing_output_on_error, profile_control_response_item_count,
    project_root_for_abi, release_owned_buffer, release_owned_buffer_after_result,
    validate_owned_buffer_releasing_on_error, validate_plugin_event_batch,
    validate_plugin_event_encoded_len, validate_runtime_frame,
    validate_runtime_frame_releasing_on_error, RuntimeFrame, RuntimeLibraryError,
    RuntimeSessionTeardownFailureState,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeFrameV1, ZrRuntimePluginEventDeliveryBatchV1,
    ZrRuntimePluginEventDeliveryV1, ZrRuntimePluginEventSubscriptionHandle, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1,
    ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1,
};

const FRAME_RELEASE_DIAGNOSTIC: &[u8] = b"frame allocation still in use";
const CAPTURE_DIAGNOSTIC: &[u8] = b"capture submission rejected";
static EMPTY_BUFFER_RELEASED: AtomicBool = AtomicBool::new(false);
static BOX_BUFFER_RELEASED: AtomicBool = AtomicBool::new(false);
static FRAME_PIXELS_RELEASED: AtomicBool = AtomicBool::new(false);

#[test]
fn profile_item_budget_counts_diagnostic_series_history_and_tags() {
    let mut response = zircon_runtime_interface::ProfileControlResponse::ok("profile");
    let mut diagnostics = zircon_runtime_interface::RuntimeDiagnosticsSnapshot::default();
    diagnostics
        .diagnostic_series
        .push(zircon_runtime_interface::RuntimeDiagnosticSeriesSnapshot {
            subsystem_tags: vec!["render".to_string(), "frame".to_string()],
            history: vec![
                zircon_runtime_interface::RuntimeDiagnosticMeasurement::default(),
                zircon_runtime_interface::RuntimeDiagnosticMeasurement::default(),
                zircon_runtime_interface::RuntimeDiagnosticMeasurement::default(),
            ],
            ..Default::default()
        });
    response.runtime_diagnostics = Some(diagnostics);

    assert_eq!(profile_control_response_item_count(&response), 7);
}

unsafe extern "C" fn record_empty_buffer_release(buffer: ZrOwnedByteBuffer) -> ZrStatus {
    if !buffer.data.is_null() {
        unsafe {
            drop(Box::from_raw(buffer.data));
        }
    }
    EMPTY_BUFFER_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_box_buffer(buffer: ZrOwnedByteBuffer) -> ZrStatus {
    if !buffer.data.is_null() {
        unsafe {
            drop(Box::from_raw(buffer.data));
        }
    }
    BOX_BUFFER_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn release_frame_pixels(buffer: ZrOwnedByteBuffer) -> ZrStatus {
    if !buffer.data.is_null() {
        unsafe {
            drop(Box::from_raw(buffer.data));
        }
    }
    FRAME_PIXELS_RELEASED.store(true, Ordering::Release);
    ZrStatus::ok()
}

unsafe extern "C" fn reject_frame_buffer_release(buffer: ZrOwnedByteBuffer) -> ZrStatus {
    if !buffer.data.is_null() {
        unsafe {
            drop(Box::from_raw(buffer.data));
        }
    }
    ZrStatus::new(
        ZrStatusCode::Error,
        ZrByteSlice::from_static(FRAME_RELEASE_DIAGNOSTIC),
    )
}

#[test]
fn runtime_frame_release_failure_is_recorded_for_terminal_teardown() {
    let teardown_failure_state = RuntimeSessionTeardownFailureState::default();
    let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    frame.width = 1;
    frame.height = 1;
    frame.rgba = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 1,
        capacity: 1,
        owner_token: 1,
        free: Some(reject_frame_buffer_release),
    };

    drop(RuntimeFrame {
        frame,
        teardown_failure_state: teardown_failure_state.clone(),
        _session: PhantomData,
    });

    assert_eq!(
        teardown_failure_state.take().unwrap().to_string(),
        "failed to free runtime frame buffer: error: frame allocation still in use"
    );
}

#[test]
fn runtime_frame_type_retains_the_session_lifetime() {
    let source = include_str!("../runtime_session.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("runtime session production source should precede its tests");

    assert!(production.contains(") -> Result<RuntimeFrame<'_>, RuntimeLibraryError> {"));
    assert!(production.contains("_session: PhantomData<&'session RuntimeSession>"));
}

#[test]
fn runtime_project_root_abi_preserves_unicode_and_absence() {
    assert_eq!(project_root_for_abi(None).unwrap(), None);
    assert_eq!(
        project_root_for_abi(Some(std::path::Path::new("E:/projects/\u{9879}\u{76ee}"))).unwrap(),
        Some("E:/projects/\u{9879}\u{76ee}")
    );
}

#[test]
#[cfg(any(windows, unix))]
fn runtime_project_root_abi_rejects_unrepresentable_os_paths() {
    #[cfg(windows)]
    let path = {
        use std::os::windows::ffi::OsStringExt;

        std::path::PathBuf::from(std::ffi::OsString::from_wide(&[0xd800]))
    };
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::OsStringExt;

        std::path::PathBuf::from(std::ffi::OsString::from_vec(vec![0xff]))
    };

    let error = project_root_for_abi(Some(&path))
        .expect_err("lossy project roots must not cross the runtime ABI");

    assert!(error
        .to_string()
        .contains("runtime project root cannot cross the UTF-8 ABI boundary"));
}

#[test]
fn empty_owned_buffer_still_invokes_its_release_callback() {
    EMPTY_BUFFER_RELEASED.store(false, Ordering::Release);
    let output = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 0,
        capacity: 1,
        owner_token: 7,
        free: Some(record_empty_buffer_release),
    };
    assert_eq!(output.len, 0);
    assert!(!output.data.is_null());

    release_owned_buffer(output, "free empty runtime output").unwrap();

    assert!(EMPTY_BUFFER_RELEASED.load(Ordering::Acquire));
}

#[test]
fn failed_output_call_retains_call_and_release_diagnostics() {
    let output = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 1,
        capacity: 1,
        owner_token: 2,
        free: Some(reject_frame_buffer_release),
    };

    let error = ensure_status_releasing_output_on_error(
        ZrStatus::new(
            ZrStatusCode::Error,
            ZrByteSlice::from_static(CAPTURE_DIAGNOSTIC),
        ),
        "capture runtime frame",
        output,
        "free runtime frame output after failed capture",
    )
    .expect_err("call and release failures must reject the runtime operation");

    assert_eq!(
        error.to_string(),
        "failed to capture runtime frame: error: capture submission rejected; cleanup also failed: failed to free runtime frame output after failed capture: error: frame allocation still in use"
    );
}

#[test]
fn malformed_owned_buffer_is_released_and_rejected_before_decode() {
    BOX_BUFFER_RELEASED.store(false, Ordering::Release);
    let output = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 2,
        capacity: 1,
        owner_token: 3,
        free: Some(release_box_buffer),
    };

    let error = validate_owned_buffer_releasing_on_error(
        output,
        "decode runtime host requests",
        "free runtime host requests",
    )
    .expect_err("malformed runtime-owned storage must be rejected before slicing");

    assert_eq!(
        error.to_string(),
        "decode runtime host requests returned malformed storage: len 2 exceeds capacity 1"
    );
    assert!(BOX_BUFFER_RELEASED.load(Ordering::Acquire));
}

#[test]
fn decode_and_release_failures_preserve_both_diagnostics() {
    let output = ZrOwnedByteBuffer {
        data: Box::into_raw(Box::new(0_u8)),
        len: 1,
        capacity: 1,
        owner_token: 4,
        free: Some(reject_frame_buffer_release),
    };

    let result: Result<(), RuntimeLibraryError> = release_owned_buffer_after_result(
        output,
        Err(RuntimeLibraryError::new(
            "decode runtime host requests: expected value",
        )),
        "free runtime host requests",
    );
    let error = result.expect_err("decode and cleanup failures must both remain visible");

    assert_eq!(
        error.to_string(),
        "decode runtime host requests: expected value; cleanup also failed: failed to free runtime host requests: error: frame allocation still in use"
    );
}

#[test]
fn runtime_frame_validation_rejects_foreign_abi() {
    let frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1);

    let error = validate_runtime_frame(&frame)
        .expect_err("a successful capture must use the negotiated frame ABI");

    assert_eq!(
        error.to_string(),
        "runtime frame used unsupported ABI version 2"
    );
}

#[test]
fn runtime_frame_validation_rejects_zero_dimensions_and_length_overflow() {
    let zero_width = ZrRuntimeFrameV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        width: 0,
        height: 1,
        generation: 1,
        rgba: ZrOwnedByteBuffer::empty(),
    };
    assert_eq!(
        validate_runtime_frame(&zero_width)
            .expect_err("a successful capture must have non-zero dimensions")
            .to_string(),
        "runtime frame returned invalid dimensions 0x1"
    );

    let overflow = ZrRuntimeFrameV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        width: u32::MAX,
        height: u32::MAX,
        generation: 1,
        rgba: ZrOwnedByteBuffer::empty(),
    };
    assert_eq!(
        validate_runtime_frame(&overflow)
            .expect_err("unrepresentable RGBA lengths must reject capture")
            .to_string(),
        "runtime frame pixel length overflowed usize"
    );
}

#[test]
fn frame_protocol_and_release_failures_preserve_both_diagnostics() {
    let frame = ZrRuntimeFrameV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1 + 1,
        width: 1,
        height: 1,
        generation: 1,
        rgba: ZrOwnedByteBuffer {
            data: Box::into_raw(Box::new(0_u8)),
            len: 1,
            capacity: 1,
            owner_token: 6,
            free: Some(reject_frame_buffer_release),
        },
    };

    let error = validate_runtime_frame_releasing_on_error(&frame)
        .expect_err("frame protocol and cleanup failures must both remain visible");

    assert_eq!(
        error.to_string(),
        "runtime frame used unsupported ABI version 2; cleanup also failed: failed to free runtime frame output after invalid capture: error: frame allocation still in use"
    );
}

#[test]
fn plugin_event_batch_rejects_crossed_subscriptions_and_oversized_pages() {
    assert_eq!(
        validate_plugin_event_encoded_len(ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_ENCODED_BYTES_V1 + 1)
            .expect_err("encoded pages above the ABI bound must be rejected")
            .to_string(),
        "runtime plugin event page returned 262145 encoded bytes; maximum is 262144"
    );

    let requested = ZrRuntimePluginEventSubscriptionHandle::new(7);
    let crossed = ZrRuntimePluginEventDeliveryBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![ZrRuntimePluginEventDeliveryV1::new(
            1,
            ZrRuntimePluginEventSubscriptionHandle::new(8),
            "zircon.test.event",
            "zircon.test.v1",
            1,
            serde_json::Value::Null,
        )],
    );
    assert_eq!(
        validate_plugin_event_batch(&crossed, requested)
            .expect_err("deliveries from another subscription must be rejected")
            .to_string(),
        "runtime plugin event delivery subscription 8 did not match requested subscription 7"
    );

    let delivery = ZrRuntimePluginEventDeliveryV1::new(
        1,
        requested,
        "zircon.test.event",
        "zircon.test.v1",
        1,
        serde_json::Value::Null,
    );
    let oversized = ZrRuntimePluginEventDeliveryBatchV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        vec![delivery; ZR_RUNTIME_PLUGIN_EVENT_PAGE_MAX_DELIVERIES_V1 + 1],
    );
    assert_eq!(
        validate_plugin_event_batch(&oversized, requested)
            .expect_err("delivery pages above the ABI bound must be rejected")
            .to_string(),
        "runtime plugin event batch returned 65 deliveries; maximum is 64"
    );
}

#[test]
fn runtime_frame_validation_releases_truncated_pixels() {
    FRAME_PIXELS_RELEASED.store(false, Ordering::Release);
    let frame = ZrRuntimeFrameV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        width: 1,
        height: 1,
        generation: 1,
        rgba: ZrOwnedByteBuffer {
            data: Box::into_raw(Box::new(0_u8)),
            len: 1,
            capacity: 1,
            owner_token: 5,
            free: Some(release_frame_pixels),
        },
    };

    let error = validate_runtime_frame_releasing_on_error(&frame)
        .expect_err("truncated runtime frame pixels must reject capture");

    assert_eq!(
        error.to_string(),
        "runtime frame returned 1 RGBA bytes for 1x1 pixels; expected 4"
    );
    assert!(FRAME_PIXELS_RELEASED.load(Ordering::Acquire));
}
