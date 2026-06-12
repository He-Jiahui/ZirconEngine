use super::support::*;

#[test]
fn tick_frame_rejects_unknown_session() {
    let api = runtime_api();
    let tick_frame = api.tick_frame.expect("tick_frame");

    let status = unsafe { tick_frame(ZrRuntimeSessionHandle::new(99_999)) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime session not found");
}

#[test]
fn tick_frame_accepts_valid_session() {
    let api = runtime_api();
    let tick_frame = api.tick_frame.expect("tick_frame");
    let session = create_test_session(api);

    let status = unsafe { tick_frame(session) };

    destroy_test_session(api, session);
    assert_eq!(status.status_code(), ZrStatusCode::Ok);
}

#[test]
fn destroy_session_reports_explicit_not_found_for_missing_nonzero_handle() {
    let api = runtime_api();
    let destroy_session = api.destroy_session.expect("destroy_session");
    let missing_session = ZrRuntimeSessionHandle::new(99_999);

    assert_session_status(
        unsafe { destroy_session(missing_session) },
        ZrStatusCode::NotFound,
        "runtime session not found",
    );
}

#[test]
fn destroy_session_removes_registry_entry_so_destroyed_handles_become_missing() {
    let source = include_str!("../session.rs");
    let destroy_start = source
        .find("pub(super) unsafe extern \"C\" fn destroy_session")
        .expect("destroy_session entry point");
    let next_entry = source[destroy_start..]
        .find("\npub(super) unsafe extern \"C\" fn handle_event")
        .map(|offset| destroy_start + offset)
        .expect("entry point after destroy_session");
    let destroy_body = &source[destroy_start..next_entry];

    assert!(destroy_body.contains("if !handle.is_valid()"));
    assert!(destroy_body.contains("registry.sessions.remove(&handle.raw()).is_none()"));
    assert!(destroy_body.contains("return not_found(b\"runtime session not found\")"));
}

#[test]
fn session_destroy_reports_explicit_not_found_after_headless_destroy() {
    let api = runtime_api();
    let destroy_session = api.destroy_session.expect("destroy_session");
    let session = create_test_session(api);

    let first_destroy = unsafe { destroy_session(session) };
    assert_eq!(first_destroy.status_code(), ZrStatusCode::Ok);

    assert_session_status(
        unsafe { destroy_session(session) },
        ZrStatusCode::NotFound,
        "runtime session not found",
    );
}

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

#[test]
fn create_session_requires_output_pointer() {
    let api = unsafe { &*zircon_runtime_get_api_v1(core::ptr::null()) };
    let create_session = api.create_session.expect("create_session");
    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1),
            core::ptr::null_mut::<ZrRuntimeSessionHandle>(),
        )
    };

    assert!(!status.is_ok());
}

#[test]
fn create_session_rejects_unknown_profile_before_runtime_bootstrap() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let profile = b"unknown-profile";

    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                profile: ZrByteSlice {
                    data: profile.as_ptr(),
                    len: profile.len(),
                },
                project_manifest: ZrByteSlice::empty(),
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "unknown runtime session profile");
    assert!(!session.is_valid());
}

#[test]
fn create_session_rejects_invalid_project_root_before_runtime_bootstrap() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let invalid_project_root = [0xff, 0xfe];

    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV1 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
                profile: ZrByteSlice::from_static(b"runtime"),
                project_manifest: ZrByteSlice {
                    data: invalid_project_root.as_ptr(),
                    len: invalid_project_root.len(),
                },
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "invalid runtime project root");
    assert!(!session.is_valid());
}

#[test]
fn create_session_accepts_named_headless_profile_without_render_bridge() {
    let api = runtime_api();
    let session = create_test_session_with_profile(api, b"headless");

    assert!(session.is_valid());
    destroy_test_session(api, session);
}

#[test]
fn dev_profile_ticks_runtime_diagnostic_store_log_schedule() {
    let source = include_str!("../session.rs");

    assert!(source.contains("RUNTIME_SESSION_PROFILE_DEV => Some(Self::Dev)"));
    assert!(source.contains("DiagnosticStoreLogSchedule::repeating"));
    assert!(source.contains("DEFAULT_DIAGNOSTIC_STORE_LOG_WAIT"));
    assert!(source.contains("collect_runtime_diagnostics(&self.runtime.handle()).store"));
    assert!(source.contains("write_diagnostic_store_snapshot"));
}

#[test]
fn minimal_and_headless_profiles_skip_render_bridge_bootstrap() {
    let source = include_str!("../session.rs");

    assert!(source.contains("fn uses_render_bridge(self) -> bool"));
    assert!(source.contains("matches!(self, Self::Runtime | Self::Editor | Self::Dev)"));
    assert!(source.contains("runtime_dynamic_session_render_bridge_skipped"));
    assert!(source.contains("let Some(render_bridge) = &mut self.render_bridge else"));
}

#[test]
fn tick_frame_drives_loaded_level_before_clearing_frame_input() {
    let source = include_str!("../session.rs");
    let tick_start = source
        .find("fn tick_frame(&mut self) -> Result<(), String>")
        .expect("runtime dynamic session tick_frame implementation");
    let level_tick = source[tick_start..]
        .find(".tick(&self.runtime.handle(), advance)")
        .expect("runtime frame should tick the loaded LevelSystem");
    let input_begin_frame = source[tick_start..]
        .find("self.input_manager.begin_frame();")
        .expect("runtime frame should clear per-frame input after gameplay tick");

    assert!(
        !source[tick_start..].contains(".tick(&self.runtime.handle(), advance.real_delta()"),
        "runtime frame should pass RuntimeTimeAdvance through instead of reducing it to raw delta"
    );
    assert!(
        level_tick < input_begin_frame,
        "runtime gameplay tick should observe the current frame's input before frame input is cleared"
    );
}

#[test]
fn session_ui_extract_remains_documented_dynamic_session_side_path() {
    let source = include_str!("../session.rs");
    let capture_start = source
        .find("fn capture_frame(")
        .expect("capture_frame implementation");
    let present_start = source
        .find("fn present_viewport(&mut self")
        .expect("present_viewport implementation");
    let ui_extract_start = source
        .find("fn current_ui_extract(&self)")
        .expect("current_ui_extract implementation");
    let resize_start = source[ui_extract_start..]
        .find("fn resize_viewport")
        .map(|offset| ui_extract_start + offset)
        .expect("method after current_ui_extract");
    let ui_extract_body = &source[ui_extract_start..resize_start];

    assert!(
        source[capture_start..present_start].contains("let ui = self.current_ui_extract();"),
        "capture_frame should keep the documented UI extract side path explicit"
    );
    assert!(
        source[present_start..ui_extract_start].contains("let ui = self.current_ui_extract();"),
        "present_viewport should keep the documented UI extract side path explicit"
    );
    assert!(ui_extract_body.contains("runtime_session_menu_extract(world, viewport_size)"));
    assert!(
        ui_extract_body.contains(".or_else(|| runtime_session_hud_extract(world, viewport_size))")
    );
    assert!(
        !ui_extract_body.contains("SystemStage::RenderExtract"),
        "current UI extract side path is not owned by the scheduled RenderExtract stage yet"
    );
}

#[test]
fn project_sessions_open_assets_before_loading_default_level() {
    let source = include_str!("../session.rs");
    let level_start = source
        .find("runtime_session_level")
        .expect("runtime dynamic session project level bootstrap");
    let open_assets = source[level_start..]
        .find("project_config.open_project_assets(&core)?;")
        .expect("project sessions should open and sync project assets");
    let load_scripts = source[level_start..]
        .find("project_config.load_startup_scripts(&core)?;")
        .expect("project sessions should load startup scripts");
    let load_level = source[level_start..]
        .find("project_config.load_default_level(&core)?")
        .expect("project sessions should load the default level");

    assert!(
        open_assets < load_scripts && open_assets < load_level,
        "project assets must be synchronized before scripts or scene rendering use project resources"
    );
}

fn assert_handle_entry_points_reject_session(
    api: &zircon_runtime_interface::ZrRuntimeApiV1,
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
    assert_session_status(
        unsafe { tick_frame(session) },
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
}

fn assert_session_status(status: ZrStatus, expected_code: ZrStatusCode, expected_message: &str) {
    assert_eq!(status.status_code(), expected_code, "{status:?}");
    assert_eq!(status_message(status), expected_message);
}

fn valid_viewport_resize_event() -> ZrRuntimeEventV1 {
    ZrRuntimeEventV1::viewport_resized(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        default_viewport(),
        valid_viewport_size(),
    )
}

fn valid_frame_request() -> ZrRuntimeFrameRequestV1 {
    ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        default_viewport(),
        valid_viewport_size(),
    )
}

fn valid_bind_viewport_surface_request() -> ZrRuntimeBindViewportSurfaceRequestV1 {
    ZrRuntimeBindViewportSurfaceRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        default_viewport(),
        valid_viewport_size(),
        ZrRuntimeNativeSurfaceTargetV1::win32(ZIRCON_RUNTIME_ABI_VERSION_V1, 1, 0),
    )
}

fn valid_profile_control_request_bytes() -> Vec<u8> {
    serde_json::to_vec(&zircon_runtime_interface::ProfileControlRequest {
        command: zircon_runtime_interface::ProfileControlCommand::Snapshot,
        config: None,
    })
    .unwrap()
}

fn default_viewport() -> ZrRuntimeViewportHandle {
    ZrRuntimeViewportHandle::new(1)
}

fn valid_viewport_size() -> ZrRuntimeViewportSizeV1 {
    ZrRuntimeViewportSizeV1::new(64, 48)
}
