use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_editor::core::gateway::{
    EditorRuntimeGateway, GatewayError, RuntimeCapabilities, SessionGateway,
};
use zircon_runtime_host::foreign_output::{RuntimeForeignOutputKind, RuntimeForeignOutputState};
use zircon_runtime_interface::{
    GatewaySessionIdentity, ZrByteSlice, ZrOwnedResultV2, ZrRuntimeAllocationId, ZrRuntimeApiV8,
    ZrRuntimeFrameRequestV1, ZrRuntimeFrameV2, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
    ZIRCON_RUNTIME_ABI_VERSION_V2,
};

static PRESENT_CALLS: AtomicUsize = AtomicUsize::new(0);
static RELEASE_CALLS: AtomicUsize = AtomicUsize::new(0);
static OWNER_OBSERVED_RELEASE_CALLS: AtomicUsize = AtomicUsize::new(usize::MAX);
static FRAME_OWNER_SESSION: AtomicU64 = AtomicU64::new(0);
static FRAME_RGBA: [u8; 4] = [1, 2, 3, 4];
const FRAME_ALLOCATION_ID: u64 = 41;

struct ReleaseOrderOwner;

impl Drop for ReleaseOrderOwner {
    fn drop(&mut self) {
        OWNER_OBSERVED_RELEASE_CALLS.store(RELEASE_CALLS.load(Ordering::SeqCst), Ordering::SeqCst);
    }
}

unsafe extern "C" fn release_test_allocation(
    session: ZrRuntimeSessionHandle,
    allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    if session.raw() != FRAME_OWNER_SESSION.load(Ordering::SeqCst)
        || allocation.raw() != FRAME_ALLOCATION_ID
    {
        return ZrStatus::new(ZrStatusCode::NotFound, ZrByteSlice::empty());
    }
    RELEASE_CALLS.fetch_add(1, Ordering::SeqCst);
    ZrStatus::ok()
}

unsafe extern "C" fn record_present(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    PRESENT_CALLS.fetch_add(1, Ordering::SeqCst);
    ZrStatus::ok()
}

unsafe extern "C" fn capture_test_frame(
    session: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
    output: *mut ZrRuntimeFrameV2,
) -> ZrStatus {
    if output.is_null() {
        return ZrStatus::new(ZrStatusCode::InvalidArgument, ZrByteSlice::empty());
    }
    FRAME_OWNER_SESSION.store(session.raw(), Ordering::SeqCst);
    unsafe {
        output.write(ZrRuntimeFrameV2 {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V2,
            width: request.size.width,
            height: request.size.height,
            generation: 31,
            rgba: ZrOwnedResultV2 {
                data: FRAME_RGBA.as_ptr(),
                len: FRAME_RGBA.len() as u64,
                allocation: ZrRuntimeAllocationId::new(FRAME_ALLOCATION_ID),
            },
        });
    }
    ZrStatus::ok()
}

unsafe extern "C" fn request_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeViewportPickRequestV1,
    _out_ticket: *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn poll_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
    _out_result: *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn cancel_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

fn test_api() -> ZrRuntimeApiV8 {
    let mut api = ZrRuntimeApiV8::empty();
    api.release_allocation = Some(release_test_allocation);
    api.present_viewport = Some(record_present);
    api.request_viewport_pick = Some(request_test_viewport_pick);
    api.poll_viewport_pick = Some(poll_test_viewport_pick);
    api.cancel_viewport_pick = Some(cancel_test_viewport_pick);
    api
}

#[test]
fn injected_foreign_output_fuse_blocks_gateway_before_runtime_dispatch() {
    PRESENT_CALLS.store(0, Ordering::SeqCst);
    let foreign_output = Arc::new(RuntimeForeignOutputState::default());
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            test_api(),
            ZrRuntimeSessionHandle::new(27),
            GatewaySessionIdentity::new(27, ZrRuntimeSessionHandle::new(27), 1, None),
            RuntimeCapabilities::editor_default(),
            foreign_output.clone(),
        )
    }
    .expect("valid gateway");

    foreign_output
        .reject_protocol::<()>(
            RuntimeForeignOutputKind::HostRequests,
            "host request output exceeded the shared budget",
        )
        .expect_err("protocol rejection must fuse the shared session state");

    let error = gateway
        .present_viewport(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(1280, 720),
        ))
        .expect_err("the gateway must observe the injected shared fuse");

    assert!(matches!(
        error,
        GatewayError::Protocol { message }
            if message.contains("prior foreign-output protocol violation")
    ));
    assert_eq!(PRESENT_CALLS.load(Ordering::SeqCst), 0);
    assert_eq!(foreign_output.metrics().blocked_session_calls, 1);
}

#[test]
fn v8_owned_frame_uses_exactly_once_allocation_release() {
    RELEASE_CALLS.store(0, Ordering::SeqCst);
    OWNER_OBSERVED_RELEASE_CALLS.store(usize::MAX, Ordering::SeqCst);
    let mut api = test_api();
    api.capture_frame = Some(capture_test_frame);
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(ReleaseOrderOwner),
            api,
            ZrRuntimeSessionHandle::new(27),
            GatewaySessionIdentity::new(27, ZrRuntimeSessionHandle::new(27), 1, None),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .expect("valid V8 gateway");

    let frame = gateway
        .capture_frame(
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(1, 1),
        )
        .expect("capture V8 runtime-owned frame");
    assert_eq!(frame.generation(), 31);
    assert_eq!(frame.rgba(), FRAME_RGBA.as_slice());
    assert_eq!(RELEASE_CALLS.load(Ordering::SeqCst), 0);
    drop(gateway);
    assert_eq!(
        OWNER_OBSERVED_RELEASE_CALLS.load(Ordering::SeqCst),
        usize::MAX
    );

    frame.release().expect("release V8 runtime allocation");
    assert_eq!(RELEASE_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(OWNER_OBSERVED_RELEASE_CALLS.load(Ordering::SeqCst), 1);

    RELEASE_CALLS.store(0, Ordering::SeqCst);
    OWNER_OBSERVED_RELEASE_CALLS.store(usize::MAX, Ordering::SeqCst);
    let mut api = test_api();
    api.capture_frame = Some(capture_test_frame);
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(ReleaseOrderOwner),
            api,
            ZrRuntimeSessionHandle::new(28),
            GatewaySessionIdentity::new(28, ZrRuntimeSessionHandle::new(28), 1, None),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .expect("valid V8 gateway");
    let frame = gateway
        .capture_frame(
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(1, 1),
        )
        .expect("capture V8 runtime-owned frame for drop release");
    drop(gateway);
    drop(frame);
    assert_eq!(RELEASE_CALLS.load(Ordering::SeqCst), 1);
    assert_eq!(OWNER_OBSERVED_RELEASE_CALLS.load(Ordering::SeqCst), 1);
}
