use crate::{
    runtime_api::ZrRuntimeCaptureFrameFnV1,
    ui::{
        accessibility::{
            UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityActionResult,
            UiAccessibilityActionSource, UiAccessibilityActionStatus, UiAccessibilityDiagnostic,
            UiAccessibilityDiagnosticCode, UiAccessibilityDiagnosticSeverity,
        },
        dispatch::{
            UiAccessibilityInputEvent, UiInputEvent, UiInputEventMetadata, UiInputSequence,
            UiInputTimestamp,
        },
        event_ui::UiNodeId,
    },
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeAccessibilityTreeRequestV1, ZrRuntimeApiV1,
    ZrRuntimeCaptureAccessibilityTreeFnV1, ZrRuntimeEventV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus, ZrStatusCode,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1,
};

fn round_trip<T>(value: &T) -> T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    serde_json::from_str(&serde_json::to_string(value).unwrap()).unwrap()
}

#[test]
fn accessibility_action_dtos_round_trip_with_defaults() {
    let request = UiAccessibilityActionRequest {
        target: UiNodeId::new(42),
        action: UiAccessibilityAction::SetValue,
        source: UiAccessibilityActionSource::AssistiveTechnology,
        value: Some("42%".to_string()),
        numeric_value: Some(0.42),
    };
    let result = UiAccessibilityActionResult {
        target: request.target,
        action: request.action,
        status: UiAccessibilityActionStatus::Unsupported,
        reason: Some("slider action not available".to_string()),
    };

    assert_eq!(round_trip(&request), request);
    assert_eq!(round_trip(&result), result);

    let default_request: UiAccessibilityActionRequest = serde_json::from_str("{}").unwrap();
    let default_result: UiAccessibilityActionResult = serde_json::from_str("{}").unwrap();
    assert_eq!(default_request.action, UiAccessibilityAction::Activate);
    assert_eq!(
        default_request.source,
        UiAccessibilityActionSource::AssistiveTechnology
    );
    assert_eq!(default_result.status, UiAccessibilityActionStatus::Accepted);
}

#[test]
fn accessibility_diagnostic_codes_serialize_as_snake_case() {
    let diagnostics = vec![
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Error,
            code: UiAccessibilityDiagnosticCode::DuplicateNodeId,
            node_id: Some(UiNodeId::new(1)),
            message: "duplicate node id".to_string(),
        },
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Error,
            code: UiAccessibilityDiagnosticCode::MissingBounds,
            node_id: Some(UiNodeId::new(2)),
            message: "interactive node has no bounds".to_string(),
        },
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Error,
            code: UiAccessibilityDiagnosticCode::InvalidFocus,
            node_id: Some(UiNodeId::new(3)),
            message: "focused node is invalid".to_string(),
        },
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Warning,
            code: UiAccessibilityDiagnosticCode::DanglingLabel,
            node_id: Some(UiNodeId::new(4)),
            message: "label target missing".to_string(),
        },
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Warning,
            code: UiAccessibilityDiagnosticCode::DanglingDescription,
            node_id: Some(UiNodeId::new(5)),
            message: "description target missing".to_string(),
        },
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Error,
            code: UiAccessibilityDiagnosticCode::RelationCycle,
            node_id: Some(UiNodeId::new(6)),
            message: "label relation cycle".to_string(),
        },
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Warning,
            code: UiAccessibilityDiagnosticCode::UnsupportedRoleAction,
            node_id: Some(UiNodeId::new(7)),
            message: "action is not supported by role".to_string(),
        },
        UiAccessibilityDiagnostic {
            severity: UiAccessibilityDiagnosticSeverity::Error,
            code: UiAccessibilityDiagnosticCode::ExcludedFocusedNode,
            node_id: Some(UiNodeId::new(8)),
            message: "focused node is excluded".to_string(),
        },
    ];

    let serialized = serde_json::to_string(&diagnostics).unwrap();

    assert!(serialized.contains("duplicate_node_id"));
    assert!(serialized.contains("missing_bounds"));
    assert!(serialized.contains("invalid_focus"));
    assert!(serialized.contains("dangling_label"));
    assert!(serialized.contains("dangling_description"));
    assert!(serialized.contains("relation_cycle"));
    assert!(serialized.contains("unsupported_role_action"));
    assert!(serialized.contains("excluded_focused_node"));
    assert_eq!(round_trip(&diagnostics), diagnostics);
}

#[test]
fn accessibility_input_event_payload_round_trips() {
    let metadata =
        UiInputEventMetadata::new(UiInputTimestamp::from_micros(123), UiInputSequence::new(7));
    let event = UiInputEvent::Accessibility(UiAccessibilityInputEvent {
        metadata,
        request: UiAccessibilityActionRequest {
            target: UiNodeId::new(9),
            action: UiAccessibilityAction::Focus,
            source: UiAccessibilityActionSource::Keyboard,
            value: None,
            numeric_value: None,
        },
    });

    let round_tripped = round_trip(&event);

    let UiInputEvent::Accessibility(payload) = round_tripped else {
        panic!("accessibility input event changed family");
    };
    assert_eq!(payload.request.target, UiNodeId::new(9));
    assert_eq!(payload.request.action, UiAccessibilityAction::Focus);
    assert_eq!(
        payload.request.source,
        UiAccessibilityActionSource::Keyboard
    );
}

#[test]
fn runtime_accessibility_tree_request_constructor_preserves_generation_hint() {
    let request = ZrRuntimeAccessibilityTreeRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(3),
        ZrRuntimeViewportSizeV1::new(640, 360),
        99,
    );

    assert_eq!(request.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(request.viewport, ZrRuntimeViewportHandle::new(3));
    assert_eq!(request.size, ZrRuntimeViewportSizeV1::new(640, 360));
    assert_eq!(request.generation_hint, 99);
}

#[test]
fn runtime_accessibility_capture_function_type_matches_abi_shape() {
    unsafe extern "C" fn capture_stub(
        session: ZrRuntimeSessionHandle,
        request: ZrRuntimeAccessibilityTreeRequestV1,
        output: *mut ZrOwnedByteBuffer,
    ) -> ZrStatus {
        assert_eq!(session, ZrRuntimeSessionHandle::new(11));
        assert_eq!(request.generation_hint, 4);
        assert!(!output.is_null());
        unsafe {
            *output = ZrOwnedByteBuffer::empty();
        }
        ZrStatus::ok()
    }

    let capture: ZrRuntimeCaptureAccessibilityTreeFnV1 = capture_stub;
    let mut output = ZrOwnedByteBuffer::empty();
    let status = unsafe {
        capture(
            ZrRuntimeSessionHandle::new(11),
            ZrRuntimeAccessibilityTreeRequestV1::new(
                ZIRCON_RUNTIME_ABI_VERSION_V1,
                ZrRuntimeViewportHandle::new(1),
                ZrRuntimeViewportSizeV1::new(1, 1),
                4,
            ),
            &mut output,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::Ok);
    assert!(output.is_empty());
}

#[test]
fn runtime_api_default_leaves_accessibility_capture_optional() {
    let api = ZrRuntimeApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);

    assert_eq!(api.abi_version, ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_eq!(api.size_bytes, core::mem::size_of::<ZrRuntimeApiV1>());
    assert!(api.capture_frame.is_none());
    assert!(api.capture_accessibility_tree.is_none());
    assert!(api.profile_control.is_none());
    assert_eq!(
        core::mem::offset_of!(ZrRuntimeApiV1, capture_accessibility_tree),
        core::mem::offset_of!(ZrRuntimeApiV1, capture_frame)
            + core::mem::size_of::<Option<ZrRuntimeCaptureFrameFnV1>>()
    );
}

#[test]
fn runtime_event_accessibility_action_carries_serialized_payload_bytes() {
    let request = UiAccessibilityActionRequest {
        target: UiNodeId::new(15),
        action: UiAccessibilityAction::Activate,
        source: UiAccessibilityActionSource::Pointer,
        value: None,
        numeric_value: None,
    };
    let bytes = serde_json::to_vec(&request).unwrap();
    let event = ZrRuntimeEventV1::accessibility_action(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(22),
        ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        },
    );

    assert_eq!(event.kind, ZR_RUNTIME_EVENT_KIND_ACCESSIBILITY_ACTION_V1);
    assert_eq!(event.viewport, ZrRuntimeViewportHandle::new(22));
    assert_eq!(unsafe { event.payload.as_slice() }, bytes.as_slice());
}
