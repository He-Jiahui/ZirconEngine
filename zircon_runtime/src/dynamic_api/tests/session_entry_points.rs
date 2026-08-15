use super::support::*;

#[test]
fn all_session_entry_points_reject_invalid_handle() {
    let api = runtime_api();
    let invalid = ZrRuntimeSessionHandle::invalid();
    let destroy_session = api.destroy_session.expect("destroy_session");

    assert_session_status(
        unsafe { destroy_session(invalid) },
        ZrStatusCode::InvalidArgument,
        "invalid runtime session handle",
    );
    assert_handle_entry_points_reject_session(
        api,
        invalid,
        ZrStatusCode::InvalidArgument,
        "invalid runtime session handle",
    );
}

#[test]
fn destroyed_headless_session_entry_points_reject_old_handle() {
    let api = runtime_api();
    let session = create_test_session(api);
    destroy_test_session(api, session);

    assert_handle_entry_points_reject_session(
        api,
        session,
        ZrStatusCode::NotFound,
        "runtime session not found",
    );
}

#[test]
fn missing_session_entry_points_reject_nonzero_handle() {
    let api = runtime_api();
    let missing_session = ZrRuntimeSessionHandle::new(99_999);

    assert_handle_entry_points_reject_session(
        api,
        missing_session,
        ZrStatusCode::NotFound,
        "runtime session not found",
    );
}

fn assert_handle_entry_points_reject_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV6,
    session: ZrRuntimeSessionHandle,
    expected_code: ZrStatusCode,
    expected_message: &str,
) {
    let handle_event = api.handle_event.expect("handle_event");
    assert_session_status(
        unsafe { handle_event(session, valid_viewport_resize_event()) },
        expected_code,
        expected_message,
    );

    let capture_frame = api.capture_frame.expect("capture_frame");
    let mut frame = ZrRuntimeFrameV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    assert_session_status(
        unsafe { capture_frame(session, valid_frame_request(), &mut frame) },
        expected_code,
        expected_message,
    );
    assert!(frame.is_empty());

    let capture_accessibility_tree = api
        .capture_accessibility_tree
        .expect("capture_accessibility_tree");
    let mut accessibility_tree = ZrOwnedByteBuffer::empty();
    assert_session_status(
        unsafe {
            capture_accessibility_tree(
                session,
                accessibility_tree_request(ZIRCON_RUNTIME_ABI_VERSION_V1, 1),
                &mut accessibility_tree,
            )
        },
        expected_code,
        expected_message,
    );
    assert!(accessibility_tree.is_empty());

    let bind_viewport_surface = api.bind_viewport_surface.expect("bind_viewport_surface");
    assert_session_status(
        unsafe { bind_viewport_surface(session, valid_bind_viewport_surface_request()) },
        expected_code,
        expected_message,
    );

    let unbind_viewport_surface = api
        .unbind_viewport_surface
        .expect("unbind_viewport_surface");
    assert_session_status(
        unsafe { unbind_viewport_surface(session, default_viewport()) },
        expected_code,
        expected_message,
    );

    let present_viewport = api.present_viewport.expect("present_viewport");
    assert_session_status(
        unsafe { present_viewport(session, valid_frame_request()) },
        expected_code,
        expected_message,
    );

    let profile_control = api.profile_control.expect("profile_control");
    let profile_control_request = valid_profile_control_request_bytes();
    let mut profile_output = ZrOwnedByteBuffer::empty();
    assert_session_status(
        unsafe {
            profile_control(
                session,
                ZrByteSlice {
                    data: profile_control_request.as_ptr(),
                    len: profile_control_request.len(),
                },
                &mut profile_output,
            )
        },
        expected_code,
        expected_message,
    );
    assert!(profile_output.is_empty());

    let tick_frame = api.tick_frame.expect("tick_frame");
    let mut demand = ZrRuntimeFrameDemandV1::idle();
    assert_session_status(
        unsafe { tick_frame(session, &mut demand) },
        expected_code,
        expected_message,
    );

    let drain_host_requests = api.drain_host_requests.expect("drain_host_requests");
    let mut host_requests = ZrOwnedByteBuffer::empty();
    assert_session_status(
        unsafe { drain_host_requests(session, &mut host_requests) },
        expected_code,
        expected_message,
    );
    assert!(host_requests.is_empty());

    let query_world = api.query_world.expect("query_world");
    let query_request =
        serde_json::to_vec(&zircon_runtime_interface::world_sync::WorldQuery::default())
            .expect("serialize world query");
    let mut query_result = ZrOwnedByteBuffer::empty();
    assert_session_status(
        unsafe {
            query_world(
                session,
                ZrByteSlice {
                    data: query_request.as_ptr(),
                    len: query_request.len(),
                },
                &mut query_result,
            )
        },
        expected_code,
        expected_message,
    );
    assert!(query_result.is_empty());

    let watch_world = api.watch_world.expect("watch_world");
    let watch_request = serde_json::to_vec(
        &zircon_runtime_interface::world_sync::WatchRegistration::new(
            zircon_runtime_interface::world_sync::WatchKey::WorldStructure,
        ),
    )
    .expect("serialize world watch");
    let mut token = zircon_runtime_interface::world_sync::WatchToken::new(0);
    assert_session_status(
        unsafe {
            watch_world(
                session,
                ZrByteSlice {
                    data: watch_request.as_ptr(),
                    len: watch_request.len(),
                },
                &mut token,
            )
        },
        expected_code,
        expected_message,
    );
    assert!(!token.is_valid());

    let unwatch_world = api.unwatch_world.expect("unwatch_world");
    let mut removed = u8::MAX;
    assert_session_status(
        unsafe {
            unwatch_world(
                session,
                zircon_runtime_interface::world_sync::WatchToken::new(1),
                &mut removed,
            )
        },
        expected_code,
        expected_message,
    );
    assert_eq!(removed, u8::MAX);

    let drain_world_invalidations = api
        .drain_world_invalidations
        .expect("drain_world_invalidations");
    let mut batches = ZrOwnedByteBuffer::empty();
    assert_session_status(
        unsafe { drain_world_invalidations(session, &mut batches) },
        expected_code,
        expected_message,
    );
    assert!(batches.is_empty());
}
