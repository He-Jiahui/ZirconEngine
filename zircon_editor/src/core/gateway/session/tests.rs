use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use super::super::{
    EditorRuntimeGateway, EditorRuntimeHighlightSet, GatewayError, RuntimeCapabilities,
    SessionProfileKind,
};
use super::gateway::SessionGateway;
use super::protocol::frame_demand_from_runtime;
use zircon_runtime_host::foreign_output::{RuntimeForeignOutputKind, RuntimeForeignOutputState};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeAllocationId, ZrRuntimeApiV7, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeHighlightSetV1,
    ZrRuntimeNativeSurfaceTargetV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode, ZIRCON_RUNTIME_ABI_VERSION_V1,
};

static RECORDED_HIGHLIGHT_SETS: Mutex<Vec<(u64, u64, Vec<u64>, bool, [u32; 4])>> =
    Mutex::new(Vec::new());
static RECORDED_VIEWPORT_SURFACE_BINDS: Mutex<Vec<(u64, u32, u32, u32, u32, u64, u64)>> =
    Mutex::new(Vec::new());
static RECORDED_VIEWPORT_SURFACE_UNBINDS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static RECORDED_VIEWPORT_PRESENTS: Mutex<Vec<(u64, u32, u32)>> = Mutex::new(Vec::new());

unsafe extern "C" fn release_test_allocation(
    _session: ZrRuntimeSessionHandle,
    _allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    ZrStatus::ok()
}

fn test_api() -> ZrRuntimeApiV7 {
    let mut api = ZrRuntimeApiV7::empty();
    api.release_allocation = Some(release_test_allocation);
    api
}

unsafe extern "C" fn record_highlight_set(
    _session: ZrRuntimeSessionHandle,
    request: ZrRuntimeHighlightSetV1,
) -> ZrStatus {
    let entities = unsafe { request.entities.as_slice() }.unwrap().to_vec();
    RECORDED_HIGHLIGHT_SETS.lock().unwrap().push((
        request.viewport.raw(),
        request.generation,
        entities,
        request.attributes.outline_enabled != 0,
        request.attributes.tint_rgba.map(f32::to_bits),
    ));
    ZrStatus::ok()
}

unsafe extern "C" fn record_viewport_surface_bind(
    _session: ZrRuntimeSessionHandle,
    request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    RECORDED_VIEWPORT_SURFACE_BINDS.lock().unwrap().push((
        request.viewport.raw(),
        request.size.width,
        request.size.height,
        request.target.kind,
        request.target.abi_version,
        request.target.window_handle,
        request.target.display_handle,
    ));
    ZrStatus::ok()
}

unsafe extern "C" fn record_viewport_surface_unbind(
    _session: ZrRuntimeSessionHandle,
    viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    RECORDED_VIEWPORT_SURFACE_UNBINDS
        .lock()
        .unwrap()
        .push(viewport.raw());
    ZrStatus::ok()
}

unsafe extern "C" fn record_viewport_present(
    _session: ZrRuntimeSessionHandle,
    request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    RECORDED_VIEWPORT_PRESENTS.lock().unwrap().push((
        request.viewport.raw(),
        request.size.width,
        request.size.height,
    ));
    ZrStatus::ok()
}

unsafe extern "C" fn reject_viewport_surface_bind(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::InvalidArgument,
        ZrByteSlice::from_static(b"surface bind rejected"),
    )
}

unsafe extern "C" fn reject_viewport_surface_unbind(
    _session: ZrRuntimeSessionHandle,
    _viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    ZrStatus::new(
        ZrStatusCode::InvalidArgument,
        ZrByteSlice::from_static(b"surface unbind rejected"),
    )
}

#[test]
fn session_gateway_forwards_viewport_surface_lifecycle_without_runtime_ownership() {
    RECORDED_VIEWPORT_SURFACE_BINDS.lock().unwrap().clear();
    RECORDED_VIEWPORT_SURFACE_UNBINDS.lock().unwrap().clear();
    RECORDED_VIEWPORT_PRESENTS.lock().unwrap().clear();

    let mut api = test_api();
    api.bind_viewport_surface = Some(record_viewport_surface_bind);
    api.unbind_viewport_surface = Some(record_viewport_surface_unbind);
    api.present_viewport = Some(record_viewport_present);
    let gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(15),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap();
    let viewport = ZrRuntimeViewportHandle::new(3);
    let size = ZrRuntimeViewportSizeV1::new(1280, 720);

    gateway
        .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            size,
            ZrRuntimeNativeSurfaceTargetV1::win32(ZIRCON_RUNTIME_ABI_VERSION_V1, 0xABCD, 0x1234),
        ))
        .unwrap();
    gateway
        .present_viewport(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            size,
        ))
        .unwrap();
    gateway.unbind_viewport_surface(viewport).unwrap();

    assert_eq!(
        RECORDED_VIEWPORT_SURFACE_BINDS.lock().unwrap().as_slice(),
        &[(
            3,
            1280,
            720,
            1,
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            0xABCD,
            0x1234
        )]
    );
    assert_eq!(
        RECORDED_VIEWPORT_PRESENTS.lock().unwrap().as_slice(),
        &[(3, 1280, 720)]
    );
    assert_eq!(
        RECORDED_VIEWPORT_SURFACE_UNBINDS.lock().unwrap().as_slice(),
        &[3]
    );
}

#[test]
fn session_gateway_updates_the_injected_viewport_surface_lifecycle_state() {
    let surface_bound = Arc::new(AtomicBool::new(false));
    let mut api = test_api();
    api.bind_viewport_surface = Some(record_viewport_surface_bind);
    api.unbind_viewport_surface = Some(record_viewport_surface_unbind);
    let gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(17),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_lifecycle_state(Arc::clone(&surface_bound));
    let viewport = ZrRuntimeViewportHandle::new(3);

    gateway
        .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZrRuntimeViewportSizeV1::new(96, 54),
            ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
        ))
        .unwrap();
    assert!(surface_bound.load(Ordering::Acquire));

    gateway.unbind_viewport_surface(viewport).unwrap();
    assert!(!surface_bound.load(Ordering::Acquire));
}

#[test]
fn session_gateway_preserves_surface_lifecycle_state_when_runtime_calls_fail() {
    let viewport = ZrRuntimeViewportHandle::new(3);
    let bind_state = Arc::new(AtomicBool::new(false));
    let mut bind_api = test_api();
    bind_api.bind_viewport_surface = Some(reject_viewport_surface_bind);
    let bind_gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            bind_api,
            ZrRuntimeSessionHandle::new(18),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_lifecycle_state(Arc::clone(&bind_state));

    assert!(bind_gateway
        .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZrRuntimeViewportSizeV1::new(96, 54),
            ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
        ))
        .is_err());
    assert!(!bind_state.load(Ordering::Acquire));

    let unbind_state = Arc::new(AtomicBool::new(true));
    let mut unbind_api = test_api();
    unbind_api.unbind_viewport_surface = Some(reject_viewport_surface_unbind);
    let unbind_gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            unbind_api,
            ZrRuntimeSessionHandle::new(19),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_lifecycle_state(Arc::clone(&unbind_state));

    assert!(unbind_gateway.unbind_viewport_surface(viewport).is_err());
    assert!(unbind_state.load(Ordering::Acquire));
}

#[test]
fn session_gateway_reports_each_missing_viewport_surface_entry() {
    let gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            test_api(),
            ZrRuntimeSessionHandle::new(16),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap();
    let viewport = ZrRuntimeViewportHandle::new(3);
    let size = ZrRuntimeViewportSizeV1::new(32, 18);

    assert_eq!(
        gateway
            .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport,
                size,
                ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
            ))
            .unwrap_err(),
        GatewayError::CapabilityMissing {
            capability: "runtime.viewport.surface.bind",
        }
    );
    assert_eq!(
        gateway.unbind_viewport_surface(viewport).unwrap_err(),
        GatewayError::CapabilityMissing {
            capability: "runtime.viewport.surface.unbind",
        }
    );
    assert_eq!(
        gateway
            .present_viewport(ZrRuntimeFrameRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport,
                size,
            ))
            .unwrap_err(),
        GatewayError::CapabilityMissing {
            capability: "runtime.viewport.present",
        }
    );
}

#[test]
fn session_gateway_submits_the_canonical_abi_value() {
    RECORDED_HIGHLIGHT_SETS.lock().unwrap().clear();
    let mut api = test_api();
    api.submit_highlight_set = Some(record_highlight_set);
    let gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(9),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap();

    gateway
        .submit_highlight_set(EditorRuntimeHighlightSet::new(
            ZrRuntimeViewportHandle::new(4),
            12,
            [9, 2, 9],
            true,
            [0.1, 0.4, 0.7, 1.0],
        ))
        .unwrap();

    assert_eq!(
        RECORDED_HIGHLIGHT_SETS.lock().unwrap().as_slice(),
        &[(
            4,
            12,
            vec![2, 9],
            true,
            [
                0.1f32.to_bits(),
                0.4f32.to_bits(),
                0.7f32.to_bits(),
                1.0f32.to_bits()
            ]
        )]
    );
}

#[test]
fn session_gateway_rejects_an_unreported_overlay_capability() {
    let mut api = test_api();
    api.submit_highlight_set = Some(record_highlight_set);
    let gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(10),
            RuntimeCapabilities::new(SessionProfileKind::Editor, Vec::<String>::new(), Vec::new()),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap();

    assert_eq!(
        gateway
            .submit_highlight_set(EditorRuntimeHighlightSet::new(
                ZrRuntimeViewportHandle::new(1),
                1,
                [],
                true,
                [0.2, 0.6, 0.9, 1.0],
            ))
            .unwrap_err(),
        GatewayError::CapabilityMissing {
            capability: "runtime.editor_overlay.highlight_set",
        }
    );
}

#[test]
fn session_gateway_has_no_private_unbounded_json_decoder() {
    let session_sources = [
        include_str!("gateway.rs"),
        include_str!("operations.rs"),
        include_str!("plugin_events.rs"),
        include_str!("profile.rs"),
        include_str!("world_sync.rs"),
    ];

    assert!(session_sources
        .iter()
        .all(|source| !source.contains("serde_json::from_slice")));
    assert!(include_str!("gateway.rs").contains(".decode_json("));
}

#[test]
fn unknown_runtime_frame_demand_kind_returns_a_protocol_error() {
    let error = frame_demand_from_runtime(ZrRuntimeFrameDemandV1 {
        abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
        kind: u32::MAX,
        delay_nanoseconds: 0,
    })
    .unwrap_err();

    assert!(matches!(
        error,
        GatewayError::Protocol { message } if message.contains("unknown kind")
    ));
}

#[test]
fn shared_foreign_output_fuse_blocks_later_gateway_calls() {
    RECORDED_VIEWPORT_PRESENTS.lock().unwrap().clear();
    let foreign_output = Arc::new(RuntimeForeignOutputState::default());
    let mut api = test_api();
    api.present_viewport = Some(record_viewport_present);
    let gateway = unsafe {
        SessionGateway::new(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(21),
            RuntimeCapabilities::editor_default(),
            foreign_output.clone(),
        )
    }
    .unwrap();
    let _ = foreign_output.reject_protocol::<()>(
        RuntimeForeignOutputKind::WorldQuery,
        "runtime world query violated its output budget",
    );

    let error = gateway
        .present_viewport(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(3),
            ZrRuntimeViewportSizeV1::new(32, 18),
        ))
        .unwrap_err();

    assert!(matches!(
        error,
        GatewayError::Protocol { message }
            if message.contains("prior foreign-output protocol violation")
    ));
    assert!(RECORDED_VIEWPORT_PRESENTS.lock().unwrap().is_empty());
}
