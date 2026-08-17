use super::support::*;

#[test]
fn tick_frame_rejects_unknown_session() {
    let api = runtime_api();
    let tick_frame = api.tick_frame.expect("tick_frame");

    let mut demand = ZrRuntimeFrameDemandV1::idle();
    let status = unsafe { tick_frame(ZrRuntimeSessionHandle::new(99_999), &mut demand) };

    assert_eq!(status.status_code(), ZrStatusCode::NotFound);
    assert_eq!(status_message(status), "runtime session not found");
}

#[test]
fn tick_frame_accepts_valid_session() {
    let api = runtime_api();
    let tick_frame = api.tick_frame.expect("tick_frame");
    let session = create_test_session(api);

    let mut demand = ZrRuntimeFrameDemandV1::idle();
    let status = unsafe { tick_frame(session, &mut demand) };

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
fn destroy_session_removes_registry_entry_only_after_event_mirror_quiescent_teardown() {
    let session_source = include_str!("../session/ffi.rs");
    let session_state_source = include_str!("../session/state.rs");
    let session_store_source = include_str!("../session/registry/session_store.rs");
    let slot_source = include_str!("../session/registry/session_slot.rs");
    let exports_source = include_str!("../exports.rs");
    let destroy_start = session_source
        .find("pub(in crate::dynamic_api) unsafe fn destroy_session")
        .expect("destroy_session rust owner");
    let next_entry = session_source[destroy_start..]
        .find("\npub(in crate::dynamic_api) unsafe fn handle_event")
        .map(|offset| destroy_start + offset)
        .expect("entry point after destroy_session");
    let destroy_body = &session_source[destroy_start..next_entry];
    let export_start = exports_source
        .find("unsafe extern \"C\" fn destroy_session_ffi")
        .expect("destroy_session FFI wrapper");
    let next_export = exports_source[export_start..]
        .find("\nunsafe extern \"C\" fn handle_event_ffi")
        .map(|offset| export_start + offset)
        .expect("FFI entry point after destroy_session");
    let export_body = &exports_source[export_start..next_export];

    assert!(destroy_body.contains("destroy_session_slot(handle)"));
    assert!(session_store_source.contains("if !slot.begin_close()"));
    assert!(session_store_source.contains("slot.frame_activity().disable_wake_entries()"));
    assert!(session_store_source.contains("slot.wait_for_actions()"));
    assert!(session_store_source.contains("slot.frame_activity().wait_for_wake_callbacks()"));
    assert!(session_store_source.contains("RuntimeDynamicSession::shutdown_before_library_unload"));
    let shutdown = session_store_source
        .find("let session_shutdown = slot")
        .expect("session shutdown result");
    let incomplete = session_store_source
        .find("if !session_shutdown {")
        .expect("incomplete teardown branch");
    let preserve = session_store_source
        .find("slot.preserve_failed_teardown_for_retry()")
        .expect("failed teardown retry owner");
    let take = session_store_source
        .find("drop(slot.take_session())")
        .expect("successful teardown Session take");
    let remove = session_store_source
        .find("registry.sessions.remove(&handle.raw())")
        .expect("successful teardown registry removal");
    assert!(shutdown < incomplete);
    assert!(incomplete < preserve);
    assert!(preserve < take);
    assert!(take < remove);
    assert!(slot_source.contains("SessionSlotPhase::TeardownRetryPending"));
    assert!(slot_source.contains("if lifecycle.phase != SessionSlotPhase::Open"));
    assert!(session_state_source.contains("self.dynamic_process_log.as_mut()"));
    assert!(session_state_source.contains("let shutdown = process_log.shutdown()"));
    assert!(session_state_source.contains("if shutdown {"));
    assert!(session_state_source.contains("self.dynamic_process_log = None"));
    assert!(export_body.contains("catch_ffi_panic(|| unsafe { destroy_session(handle) })"));
}

#[test]
fn create_session_validates_abi_and_startup_config_before_acquiring_the_dynamic_log_lease() {
    let session_source = include_str!("../session/ffi.rs");
    let create_start = session_source
        .find("pub(in crate::dynamic_api) unsafe fn create_session(")
        .expect("create_session rust owner");
    let create_end = session_source[create_start..]
        .find("\nfn invalid_runtime_startup_config")
        .map(|offset| create_start + offset)
        .expect("create_session helper after owner");
    let create_body = &session_source[create_start..create_end];
    let abi_validation = create_body
        .find("if config.abi_version != ZIRCON_RUNTIME_ABI_VERSION_V3")
        .expect("ABI validation");
    let project_validation = create_body
        .find("let project_config = match RuntimeProjectConfig::from_abi_startup_config(")
        .expect("startup config validation");
    let lease_acquisition = create_body
        .find("acquire_dynamic_unity_process_log")
        .expect("dynamic log lease acquisition");

    assert!(abi_validation < lease_acquisition);
    assert!(project_validation < lease_acquisition);
}

#[test]
fn create_session_aborts_before_dynamic_library_unload_when_post_lease_bootstrap_cannot_stop_the_worker(
) {
    let session_source = include_str!("../session/ffi.rs");
    let create_start = session_source
        .find("pub(in crate::dynamic_api) unsafe fn create_session(")
        .expect("create_session rust owner");
    let create_end = session_source[create_start..]
        .find("\nfn invalid_runtime_startup_config")
        .map(|offset| create_start + offset)
        .expect("create_session helper after owner");
    let create_body = &session_source[create_start..create_end];
    let lease_acquisition = create_body
        .find("let mut dynamic_process_log =")
        .expect("dynamic log lease acquisition");
    let construction = create_body
        .find("match RuntimeDynamicSession::new(profile, project_config) {")
        .expect("runtime session construction");
    let failed_construction_cleanup = create_body
        .find("if !dynamic_process_log.shutdown() {")
        .expect("failed construction must explicitly release the dynamic log lease");
    let process_abort = create_body
        .find("std::process::abort();")
        .expect("unconfirmed worker shutdown must prevent dynamic library unload");

    assert!(lease_acquisition < construction);
    assert!(construction < failed_construction_cleanup);
    assert!(failed_construction_cleanup < process_abort);
}

#[test]
fn create_session_connects_the_runtime_wake_sink_to_the_asset_completion_producer() {
    let ffi_source = include_str!("../session/ffi.rs");
    let wake_source = include_str!("../session/registry/wake_registration.rs");
    let state_source = include_str!("../session/state.rs");
    let queue_source = include_str!("../../scene/dynamic_scene/asset_reload/queue.rs");

    let wake_adapter = wake_source
        .find("fn channel_wake(&self) -> ChannelWakeCallback")
        .expect("runtime wake callback adapter");
    let producer_injection = ffi_source
        .find("session.with_runtime_frame_wake(wake.channel_wake())")
        .expect("session wake producer injection");
    let session_registration = ffi_source
        .find("let handle = insert_session_with_wake(")
        .expect("session registration after producer injection");

    assert!(wake_adapter > 0);
    assert!(producer_injection < session_registration);
    assert!(queue_source.contains("subscribe_project_generation_wake(wake)"));
    assert!(queue_source.contains("self.drain_runtime_frame_wake_token();"));
    assert!(queue_source.contains("pub(crate) fn has_pending_work(&self) -> bool"));
    assert!(state_source.contains("asset_reload_frame_demand("));
    assert!(state_source.contains("DynamicSceneAssetReloadQueue::has_pending_work"));
}

#[test]
fn every_session_action_enters_the_slot_before_payload_validation() {
    let session_source = include_str!("../session/ffi.rs");
    let operation_source = include_str!("../session/operation.rs");

    for name in [
        "handle_event",
        "capture_frame",
        "capture_accessibility_tree",
        "bind_viewport_surface",
        "unbind_viewport_surface",
        "present_viewport",
        "profile_control",
        "drain_host_requests",
        "subscribe_plugin_event",
        "unsubscribe_plugin_event",
        "drain_plugin_events",
    ] {
        assert_action_admission_precedes_return(
            function_body(
                session_source,
                &format!("pub(in crate::dynamic_api) unsafe fn {name}("),
                "\npub(in crate::dynamic_api) unsafe fn ",
            ),
            "with_session(handle, |session| {",
        );
    }
    assert_action_admission_precedes_return(
        function_body(
            session_source,
            "pub(in crate::dynamic_api) unsafe fn tick_frame(",
            "\npub(in crate::dynamic_api) unsafe fn ",
        ),
        "with_session_activity(handle, |session, activity| {",
    );
    for name in ["submit_operation", "poll_operation", "harvest_operation"] {
        assert_action_admission_precedes_return(
            function_body(
                operation_source,
                &format!("pub(crate) unsafe fn {name}("),
                "\npub(crate) unsafe fn ",
            ),
            "with_session(session, |runtime| {",
        );
    }
}

fn function_body<'a>(source: &'a str, signature: &str, next_signature: &str) -> &'a str {
    let start = source.find(signature).expect("session action owner");
    let tail = &source[start + signature.len()..];
    let end = tail.find(next_signature).unwrap_or(tail.len());
    &source[start..start + signature.len() + end]
}

fn assert_action_admission_precedes_return(body: &str, admission: &str) {
    let admission = body.find(admission).expect("session slot action admission");
    assert!(
        !body[..admission].contains("return "),
        "session payload rejected before action guard: {body}"
    );
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
fn create_session_requires_output_pointer() {
    let api = unsafe { &*zircon_runtime_get_api_v7(core::ptr::null()) };
    let create_session = api.create_session.expect("create_session");
    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV3::empty(),
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
            ZrRuntimeSessionConfigV3 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                profile: ZrByteSlice {
                    data: profile.as_ptr(),
                    len: profile.len(),
                },
                project_root: ZrByteSlice::empty(),
                play_scene: ZrByteSlice::empty(),
                play_report_pipe: ZrByteSlice::empty(),
                wake_sink: ZrRuntimeWakeSinkV1::disabled(),
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
            ZrRuntimeSessionConfigV3 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                profile: ZrByteSlice::from_static(b"runtime"),
                project_root: ZrByteSlice {
                    data: invalid_project_root.as_ptr(),
                    len: invalid_project_root.len(),
                },
                play_scene: ZrByteSlice::empty(),
                play_report_pipe: ZrByteSlice::empty(),
                wake_sink: ZrRuntimeWakeSinkV1::disabled(),
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(status_message(status), "invalid runtime project root");
    assert!(!session.is_valid());
}

#[test]
fn create_session_rejects_play_scene_without_project_before_runtime_bootstrap() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let scene = b".zircon/play/instance/play-scene.zrscene.json";

    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV3 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                profile: ZrByteSlice::from_static(b"runtime"),
                project_root: ZrByteSlice::empty(),
                play_scene: ZrByteSlice {
                    data: scene.as_ptr(),
                    len: scene.len(),
                },
                play_report_pipe: ZrByteSlice::empty(),
                wake_sink: ZrRuntimeWakeSinkV1::disabled(),
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "runtime Play scene requires a project root"
    );
    assert!(!session.is_valid());
}

#[test]
fn create_session_rejects_play_report_outlet_without_project_before_runtime_bootstrap() {
    let api = runtime_api();
    let create_session = api.create_session.expect("create_session");
    let mut session = ZrRuntimeSessionHandle::invalid();
    let outlet = b"zircon-play-report-instance";

    let status = unsafe {
        create_session(
            ZrRuntimeSessionConfigV3 {
                abi_version: ZIRCON_RUNTIME_ABI_VERSION_V3,
                profile: ZrByteSlice::from_static(b"runtime"),
                project_root: ZrByteSlice::empty(),
                play_scene: ZrByteSlice::empty(),
                play_report_pipe: ZrByteSlice {
                    data: outlet.as_ptr(),
                    len: outlet.len(),
                },
                wake_sink: ZrRuntimeWakeSinkV1::disabled(),
            },
            &mut session,
        )
    };

    assert_eq!(status.status_code(), ZrStatusCode::InvalidArgument);
    assert_eq!(
        status_message(status),
        "runtime Play report outlet requires a project root"
    );
    assert!(!session.is_valid());
}
