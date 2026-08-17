use super::support::*;

#[test]
fn capture_accessibility_tree_requires_output_pointer() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let session = create_test_session(api);

    let status = unsafe {
        capture_accessibility_tree(
            session,
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1, 1),
            core::ptr::null_mut::<ZrOwnedResultV2>(),
        )
    };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "missing accessibility tree output");
}

#[test]
fn capture_accessibility_tree_rejects_wrong_abi_after_session_action_admission() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let mut output = ZrOwnedResultV2::empty();
    let session = create_test_session(api);

    let status = unsafe {
        capture_accessibility_tree(
            session,
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1 + 1, 1),
            &mut output,
        )
    };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::UnsupportedVersion);
    assert!(output.is_empty());
}

#[test]
fn capture_accessibility_tree_rejects_unknown_viewport() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let mut output = ZrOwnedResultV2::empty();
    let session = create_test_session(api);

    let status = unsafe {
        capture_accessibility_tree(
            session,
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1, 44),
            &mut output,
        )
    };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime viewport not found");
    assert!(output.is_empty());
}

#[test]
fn capture_accessibility_tree_returns_serialized_preview_snapshot() {
    let api = runtime_api();
    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let session = create_test_session(api);
    let mut output = ZrOwnedResultV2::empty();

    let status = unsafe {
        capture_accessibility_tree(
            session,
            accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1, 1),
            &mut output,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::Ok);
    assert!(!output.is_empty());
    assert!(output.allocation.is_valid());

    let bytes = output_bytes(&output);
    let snapshot: UiAccessibilityTreeSnapshot = serde_json::from_slice(bytes).unwrap();
    assert_eq!(snapshot.roots, vec![UiNodeId::new(1)]);
    assert_eq!(snapshot.nodes.len(), 1);
    assert_eq!(
        snapshot.nodes[0].name.as_deref(),
        Some("Zircon Runtime Preview")
    );
    assert!(snapshot.diagnostics.iter().any(|diagnostic| {
        diagnostic.message
            == "runtime UI surface accessibility extraction unavailable in dynamic preview"
    }));

    release_output(session, output);
    destroy_test_session(api, session);
}

#[test]
fn accessibility_action_event_rejects_invalid_json_payload() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let session = create_test_session(api);
    let payload = b"not-json";
    let event = ZrRuntimeEventV1::accessibility_action(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrByteSlice {
            data: payload.as_ptr(),
            len: payload.len(),
        },
    );

    let status = unsafe { handle_event(session, event) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "invalid accessibility action payload"
    );
}

#[test]
fn accessibility_action_event_rejects_dynamic_preview_without_surface() {
    let api = runtime_api();
    let handle_event = api.handle_event.expect("handle_event");
    let session = create_test_session(api);
    let request = UiAccessibilityActionRequest {
        target: UiNodeId::new(1),
        action: UiAccessibilityAction::Focus,
        source: UiAccessibilityActionSource::AssistiveTechnology,
        value: None,
        numeric_value: None,
        text_selection: None,
        scroll_offset: None,
    };
    let bytes = serde_json::to_vec(&request).unwrap();
    let event = ZrRuntimeEventV1::accessibility_action(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZrByteSlice {
            data: bytes.as_ptr(),
            len: bytes.len(),
        },
    );

    let status = unsafe { handle_event(session, event) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(
        status_message(status),
        "runtime UI surface accessibility action dispatch unavailable in dynamic preview"
    );
}
