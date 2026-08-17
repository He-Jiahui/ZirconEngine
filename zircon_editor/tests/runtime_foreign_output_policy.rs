use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use zircon_editor::core::gateway::{
    EditorRuntimeGateway, GatewayError, RuntimeCapabilities, SessionGateway,
};
use zircon_runtime_host::foreign_output::{RuntimeForeignOutputKind, RuntimeForeignOutputState};
use zircon_runtime_interface::{
    ZrRuntimeApiV6, ZrRuntimeFrameRequestV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

static PRESENT_CALLS: AtomicUsize = AtomicUsize::new(0);

unsafe extern "C" fn record_present(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    PRESENT_CALLS.fetch_add(1, Ordering::SeqCst);
    ZrStatus::ok()
}

#[test]
fn injected_foreign_output_fuse_blocks_gateway_before_runtime_dispatch() {
    PRESENT_CALLS.store(0, Ordering::SeqCst);
    let foreign_output = Arc::new(RuntimeForeignOutputState::default());
    let mut api = ZrRuntimeApiV6::empty();
    api.present_viewport = Some(record_present);
    let gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(27),
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
