use std::mem::size_of;
use std::sync::{Arc, Mutex};

use super::super::{
    EditorRuntimeGateway, EditorRuntimeGatewayHandle, EditorRuntimeHighlightSet, GatewayError,
    GatewaySessionIdentity, RuntimeCapabilities, SessionProfileKind,
};
use super::gateway::SessionGateway;
use super::protocol::frame_demand_from_runtime;
use zircon_runtime_host::{
    foreign_output::{RuntimeForeignOutputKind, RuntimeForeignOutputState},
    viewport_surface::ViewportSurfaceBindings,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeAllocationId, ZrRuntimeApiV8, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeEditorTransformPhaseV1, ZrRuntimeEditorTransformWriteV1, ZrRuntimeEventV1,
    ZrRuntimeFrameDemandV1, ZrRuntimeFrameRequestV1, ZrRuntimeHighlightSetV1,
    ZrRuntimeNativeSurfaceTargetV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle,
    ZrRuntimeViewportPickDispositionV1, ZrRuntimeViewportPickPurposeV1,
    ZrRuntimeViewportPickRequestV1, ZrRuntimeViewportPickResultV1, ZrRuntimeViewportPickTicket,
    ZrRuntimeViewportPixelV1, ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1,
};

#[test]
fn session_gateway_exposes_the_app_validated_module_composition_receipt() {
    use zircon_runtime_interface::runtime_build_set::{
        ZrRuntimeDigestV1, ZrRuntimeModuleCompositionReceiptV1, ZrRuntimeModuleCompositionTargetV1,
        ZrRuntimeSessionProfileV1,
    };

    let receipt = ZrRuntimeModuleCompositionReceiptV1::new(
        3,
        11,
        ZrRuntimeModuleCompositionTargetV1::EditorHost,
        None,
        ZrRuntimeSessionProfileV1::Editor,
        ZrRuntimeDigestV1::parse(
            "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff",
        )
        .unwrap(),
    );
    let session = ZrRuntimeSessionHandle::new(9);
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            test_api(),
            session,
            test_identity(session),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_module_composition_receipt(receipt.clone())
    .unwrap();

    assert_eq!(
        EditorRuntimeGateway::module_composition_receipt(&gateway).as_deref(),
        Some(&receipt)
    );
    let handle = EditorRuntimeGatewayHandle::new(Arc::new(gateway));
    assert_eq!(
        handle.module_composition_receipt().as_deref(),
        Some(&receipt)
    );
}

static RECORDED_HIGHLIGHT_SETS: Mutex<Vec<(u64, u64, Vec<u64>, bool, [u32; 4])>> =
    Mutex::new(Vec::new());
static RECORDED_VIEWPORT_SURFACE_BINDS: Mutex<Vec<(u64, u32, u32, u32, u32, u64, u64)>> =
    Mutex::new(Vec::new());
static RECORDED_VIEWPORT_SURFACE_UNBINDS: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static RECORDED_VIEWPORT_PRESENTS: Mutex<Vec<(u64, u32, u32)>> = Mutex::new(Vec::new());
static RECORDED_VIEWPORT_PICK_REQUEST: Mutex<Option<ZrRuntimeViewportPickRequestV1>> =
    Mutex::new(None);
static RECORDED_EDITOR_TRANSFORM_WRITE: Mutex<Option<ZrRuntimeEditorTransformWriteV1>> =
    Mutex::new(None);

unsafe extern "C" fn release_test_allocation(
    _session: ZrRuntimeSessionHandle,
    _allocation: ZrRuntimeAllocationId,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn request_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    request: ZrRuntimeViewportPickRequestV1,
    out_ticket: *mut ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    *RECORDED_VIEWPORT_PICK_REQUEST.lock().unwrap() = Some(request);
    unsafe { out_ticket.write(ZrRuntimeViewportPickTicket::new(1)) };
    ZrStatus::ok()
}

unsafe extern "C" fn poll_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    ticket: ZrRuntimeViewportPickTicket,
    out_result: *mut ZrRuntimeViewportPickResultV1,
) -> ZrStatus {
    let request = RECORDED_VIEWPORT_PICK_REQUEST
        .lock()
        .unwrap()
        .expect("viewport-pick request must precede poll");
    unsafe {
        out_result.write(ZrRuntimeViewportPickResultV1::empty(
            ZrRuntimeViewportPickDispositionV1::Unavailable,
            ticket,
            request,
            0,
        ))
    };
    ZrStatus::ok()
}

unsafe extern "C" fn cancel_test_viewport_pick(
    _session: ZrRuntimeSessionHandle,
    _ticket: ZrRuntimeViewportPickTicket,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn record_runtime_event(
    _session: ZrRuntimeSessionHandle,
    event: ZrRuntimeEventV1,
) -> ZrStatus {
    let request = unsafe { ZrRuntimeEditorTransformWriteV1::from_payload(event.payload) }
        .expect("test editor-transform event payload");
    *RECORDED_EDITOR_TRANSFORM_WRITE.lock().unwrap() = Some(request);
    ZrStatus::ok()
}

fn test_api() -> ZrRuntimeApiV8 {
    let mut api = ZrRuntimeApiV8::empty();
    api.release_allocation = Some(release_test_allocation);
    api.request_viewport_pick = Some(request_test_viewport_pick);
    api.poll_viewport_pick = Some(poll_test_viewport_pick);
    api.cancel_viewport_pick = Some(cancel_test_viewport_pick);
    api.handle_event = Some(record_runtime_event);
    api
}

fn test_identity(session: ZrRuntimeSessionHandle) -> GatewaySessionIdentity {
    GatewaySessionIdentity::new(1, session, 1, None)
}

#[test]
fn session_gateway_preserves_exact_viewport_pick_request_identity() {
    *RECORDED_VIEWPORT_PICK_REQUEST.lock().unwrap() = None;
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            test_api(),
            ZrRuntimeSessionHandle::new(31),
            test_identity(ZrRuntimeSessionHandle::new(31)),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap();
    let request = ZrRuntimeViewportPickRequestV1::new(
        ZrRuntimeViewportHandle::new(3),
        ZrRuntimeViewportSizeV1::new(1280, 720),
        ZrRuntimeViewportPixelV1::new(640, 360),
        19,
        23,
        ZrRuntimeViewportPickPurposeV1::Press,
        0,
    );

    let ticket = EditorRuntimeGateway::request_viewport_pick(&gateway, request).unwrap();
    let result = EditorRuntimeGateway::poll_viewport_pick(&gateway, ticket).unwrap();

    assert_eq!(ticket, ZrRuntimeViewportPickTicket::new(1));
    assert!(result.matches_request(request));
    assert_eq!(
        result.disposition(),
        Some(ZrRuntimeViewportPickDispositionV1::Unavailable)
    );
    EditorRuntimeGateway::cancel_viewport_pick(&gateway, ticket).unwrap();
}

#[test]
fn session_gateway_dispatches_fixed_editor_transform_payload_synchronously() {
    use zircon_runtime_interface::math::{Transform, Vec3};

    *RECORDED_EDITOR_TRANSFORM_WRITE.lock().unwrap() = None;
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            test_api(),
            ZrRuntimeSessionHandle::new(41),
            test_identity(ZrRuntimeSessionHandle::new(41)),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap();
    let before = Transform::identity();
    let after = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    let request = ZrRuntimeEditorTransformWriteV1::new(
        7,
        11,
        1,
        13,
        ZrRuntimeEditorTransformPhaseV1::Apply,
        before,
        after,
    );

    EditorRuntimeGateway::handle_event(
        &gateway,
        ZrRuntimeEventV1::editor_transform_write(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            &request,
        ),
    )
    .unwrap();

    assert_eq!(
        *RECORDED_EDITOR_TRANSFORM_WRITE.lock().unwrap(),
        Some(request)
    );
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
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(15),
            test_identity(ZrRuntimeSessionHandle::new(15)),
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
fn session_gateway_rejects_a_v8_table_with_an_incorrect_exact_size() {
    let mut api = test_api();
    api.size_bytes = size_of::<ZrRuntimeApiV8>() - 1;

    assert_eq!(
        unsafe {
            SessionGateway::new_with_identity(
                Arc::new(()),
                api,
                ZrRuntimeSessionHandle::new(21),
                test_identity(ZrRuntimeSessionHandle::new(21)),
                RuntimeCapabilities::editor_default(),
                Arc::new(RuntimeForeignOutputState::default()),
            )
        }
        .unwrap_err(),
        GatewayError::Protocol {
            message: format!(
                "runtime API V8 requires table size {}, received {}",
                size_of::<ZrRuntimeApiV8>(),
                size_of::<ZrRuntimeApiV8>() - 1
            ),
        }
    );
}

#[test]
fn session_gateway_maps_a_shared_v8_family_rejection_to_a_protocol_error() {
    let mut api = test_api();
    api.abi_version = 0;

    assert_eq!(
        unsafe {
            SessionGateway::new_with_identity(
                Arc::new(()),
                api,
                ZrRuntimeSessionHandle::new(22),
                test_identity(ZrRuntimeSessionHandle::new(22)),
                RuntimeCapabilities::editor_default(),
                Arc::new(RuntimeForeignOutputState::default()),
            )
        }
        .unwrap_err(),
        GatewayError::Protocol {
            message: "runtime API V8 requires version 8, received version 0".to_string(),
        }
    );
}

#[test]
fn session_gateway_publishes_viewport_surface_bindings_to_the_session_owner() {
    let bindings = Arc::new(ViewportSurfaceBindings::default());
    let mut api = test_api();
    api.bind_viewport_surface = Some(record_viewport_surface_bind);
    api.unbind_viewport_surface = Some(record_viewport_surface_unbind);
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(17),
            test_identity(ZrRuntimeSessionHandle::new(17)),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_bindings(Arc::clone(&bindings));
    let viewport = ZrRuntimeViewportHandle::new(3);

    gateway
        .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZrRuntimeViewportSizeV1::new(96, 54),
            ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
        ))
        .unwrap();
    assert_eq!(bindings.bound_viewports(), vec![viewport]);

    gateway.unbind_viewport_surface(viewport).unwrap();
    assert!(bindings.bound_viewports().is_empty());
}

#[test]
fn session_gateway_restores_published_viewport_surface_bindings_when_runtime_calls_fail() {
    let viewport = ZrRuntimeViewportHandle::new(3);
    let bind_bindings = Arc::new(ViewportSurfaceBindings::default());
    let mut bind_api = test_api();
    bind_api.bind_viewport_surface = Some(reject_viewport_surface_bind);
    let bind_gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            bind_api,
            ZrRuntimeSessionHandle::new(18),
            test_identity(ZrRuntimeSessionHandle::new(18)),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_bindings(Arc::clone(&bind_bindings));

    assert!(bind_gateway
        .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZrRuntimeViewportSizeV1::new(96, 54),
            ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
        ))
        .is_err());
    assert!(bind_bindings.bound_viewports().is_empty());

    let unbind_bindings = Arc::new(ViewportSurfaceBindings::default());
    let mut unbind_api = test_api();
    unbind_api.bind_viewport_surface = Some(record_viewport_surface_bind);
    unbind_api.unbind_viewport_surface = Some(reject_viewport_surface_unbind);
    let unbind_gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            unbind_api,
            ZrRuntimeSessionHandle::new(19),
            test_identity(ZrRuntimeSessionHandle::new(19)),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_bindings(Arc::clone(&unbind_bindings));

    unbind_gateway
        .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZrRuntimeViewportSizeV1::new(96, 54),
            ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
        ))
        .unwrap();

    assert!(unbind_gateway.unbind_viewport_surface(viewport).is_err());
    assert_eq!(unbind_bindings.bound_viewports(), vec![viewport]);
}

#[test]
fn session_gateway_rejects_a_viewport_surface_transition_owned_by_the_session() {
    let viewport = ZrRuntimeViewportHandle::new(23);
    let bindings = Arc::new(ViewportSurfaceBindings::default());
    let transition = bindings
        .begin_binding(viewport)
        .expect("session transition reservation begins");
    let mut api = test_api();
    api.bind_viewport_surface = Some(record_viewport_surface_bind);
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(20),
            test_identity(ZrRuntimeSessionHandle::new(20)),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_bindings(Arc::clone(&bindings));

    assert_eq!(
        gateway
            .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                viewport,
                ZrRuntimeViewportSizeV1::new(96, 54),
                ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
            ))
            .unwrap_err(),
        GatewayError::ViewportSurfaceTransitionInFlight { viewport: 23 }
    );
    drop(transition);
}

#[test]
fn session_gateway_reports_missing_viewport_surface_entries_without_requiring_empty_unbind() {
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            test_api(),
            ZrRuntimeSessionHandle::new(16),
            test_identity(ZrRuntimeSessionHandle::new(16)),
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
    assert!(gateway.unbind_viewport_surface(viewport).is_ok());
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
fn session_gateway_preserves_a_bound_surface_when_unbind_capability_is_missing() {
    let viewport = ZrRuntimeViewportHandle::new(30);
    let bindings = Arc::new(ViewportSurfaceBindings::default());
    let mut api = test_api();
    api.bind_viewport_surface = Some(record_viewport_surface_bind);
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(23),
            test_identity(ZrRuntimeSessionHandle::new(23)),
            RuntimeCapabilities::editor_default(),
            Arc::new(RuntimeForeignOutputState::default()),
        )
    }
    .unwrap()
    .with_viewport_surface_bindings(Arc::clone(&bindings));

    gateway
        .bind_viewport_surface(ZrRuntimeBindViewportSurfaceRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            ZrRuntimeViewportSizeV1::new(64, 36),
            ZrRuntimeNativeSurfaceTargetV1::none(ZIRCON_RUNTIME_ABI_VERSION_V1),
        ))
        .unwrap();

    assert_eq!(
        gateway.unbind_viewport_surface(viewport).unwrap_err(),
        GatewayError::CapabilityMissing {
            capability: "runtime.viewport.surface.unbind",
        }
    );
    assert_eq!(bindings.bound_viewports(), vec![viewport]);
}

#[test]
fn session_gateway_submits_the_canonical_abi_value() {
    RECORDED_HIGHLIGHT_SETS.lock().unwrap().clear();
    let mut api = test_api();
    api.submit_highlight_set = Some(record_highlight_set);
    let gateway = unsafe {
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(9),
            test_identity(ZrRuntimeSessionHandle::new(9)),
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
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(10),
            test_identity(ZrRuntimeSessionHandle::new(10)),
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
        SessionGateway::new_with_identity(
            Arc::new(()),
            api,
            ZrRuntimeSessionHandle::new(21),
            test_identity(ZrRuntimeSessionHandle::new(21)),
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
