use std::path::PathBuf;

use super::library_path::{
    default_runtime_library_path, platform_runtime_library_name,
    runtime_library_path_for_executable,
};
use super::loaded_runtime::{
    runtime_api_field_available, runtime_api_required_layout_available,
    runtime_api_supports_viewport_surface_present, validate_runtime_api_pointer, LoadedRuntime,
};
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeDrainHostRequestsFnV1, ZrRuntimeProfileControlFnV1, ZrRuntimeTickFrameFnV1,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV2, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeOperationHandle,
    ZrRuntimePluginEventSubscriptionHandle, ZrRuntimeSessionConfigV1, ZrRuntimeSessionHandle,
    ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus,
};

#[cfg(feature = "target-editor-host")]
#[test]
fn runtime_session_satisfies_editor_gateway_thread_safety_contract() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<super::runtime_session::RuntimeSession>();
}

unsafe extern "C" fn fake_subscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    _out_subscription: *mut ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_unsubscribe_plugin_event(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_drain_plugin_events(
    _session: ZrRuntimeSessionHandle,
    _subscription: ZrRuntimePluginEventSubscriptionHandle,
    _out_deliveries: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    ZrStatus::ok()
}

#[cfg(feature = "target-editor-host")]
#[test]
fn editor_product_ticks_selected_navigation_plugin_into_typed_consumer() {
    use std::sync::Arc;

    use zircon_editor::core::gateway::EditorRuntimeGatewayHandle;
    use zircon_editor::core::runtime_event_consumer::EditorRuntimeEventConsumerHost;
    use zircon_runtime::builtin::RuntimePluginId;
    use zircon_runtime::core::framework::platform::RuntimeTargetMode;
    use zircon_runtime::core::framework::project::{ProjectPluginManifest, ProjectPluginSelection};

    let manifest = ProjectPluginManifest {
        selections: vec![ProjectPluginSelection::runtime_plugin(
            RuntimePluginId::Navigation,
            true,
            false,
        )
        .with_target_modes([RuntimeTargetMode::EditorHost])],
    };
    let runtime_registrations = crate::entry::first_party_runtime_plugin_registrations_for_manifest(
        RuntimeTargetMode::EditorHost,
        &manifest,
    );
    let mut editor_registrations =
        crate::entry::first_party_editor_plugin_registrations_for_manifest(
            RuntimeTargetMode::EditorHost,
            &manifest,
        );
    assert_eq!(runtime_registrations.len(), 1);
    assert_eq!(editor_registrations.len(), 1);

    let runtime = Arc::new(
        super::runtime_session::RuntimeSession::create_linked_with_profile_and_project(
            LoadedRuntime::linked().unwrap(),
            b"editor",
            None,
            runtime_registrations,
        )
        .unwrap(),
    );
    let host =
        EditorRuntimeEventConsumerHost::new(EditorRuntimeGatewayHandle::new(runtime.clone()));
    let editor_registration = editor_registrations.remove(0);
    let capability = editor_registration.runtime_event_consumers.manifests()[0]
        .required_capability
        .clone();
    host.register(editor_registration.runtime_event_consumers)
        .unwrap();
    host.begin_play_session(1, &[capability]).unwrap();

    assert!(runtime.tick_frame().unwrap());
    assert_eq!(host.pump().unwrap(), 0);
    assert!(runtime.tick_frame().unwrap());
    assert_eq!(host.pump().unwrap(), 1);

    host.reconcile_enabled_capabilities(&[]).unwrap();
    assert!(runtime.tick_frame().unwrap());
    assert_eq!(host.pump().unwrap(), 0);
    host.end_play_session(1).unwrap();
}

#[test]
fn runtime_library_path_uses_environment_override() {
    let previous = std::env::var_os("ZIRCON_RUNTIME_LIBRARY");
    let expected = PathBuf::from("custom-runtime-library");
    std::env::set_var("ZIRCON_RUNTIME_LIBRARY", &expected);

    let actual = default_runtime_library_path().unwrap();

    match previous {
        Some(previous) => std::env::set_var("ZIRCON_RUNTIME_LIBRARY", previous),
        None => std::env::remove_var("ZIRCON_RUNTIME_LIBRARY"),
    }
    assert_eq!(actual, expected);
}

#[test]
fn platform_runtime_library_name_matches_target() {
    let name = platform_runtime_library_name();

    #[cfg(target_os = "windows")]
    assert_eq!(name, "zircon_runtime.dll");
    #[cfg(target_os = "macos")]
    assert_eq!(name, "libzircon_runtime.dylib");
    #[cfg(all(unix, not(target_os = "macos")))]
    assert_eq!(name, "libzircon_runtime.so");
}

#[test]
fn runtime_library_path_prefers_executable_sibling_when_present() {
    let temp = runtime_library_temp_dir("sibling");
    let bin_dir = temp.join("debug");
    std::fs::create_dir_all(bin_dir.join("deps")).unwrap();
    let executable = bin_dir.join("zircon_editor-test-exe");
    let sibling = bin_dir.join(platform_runtime_library_name());
    let deps = bin_dir.join("deps").join(platform_runtime_library_name());
    std::fs::write(&sibling, []).unwrap();
    std::fs::write(&deps, []).unwrap();

    let actual = runtime_library_path_for_executable(&executable);

    remove_runtime_library_temp_dir(&temp);
    assert_eq!(actual, sibling);
}

#[test]
fn runtime_library_path_falls_back_to_cargo_deps_sibling() {
    let temp = runtime_library_temp_dir("deps");
    let bin_dir = temp.join("debug");
    std::fs::create_dir_all(bin_dir.join("deps")).unwrap();
    let executable = bin_dir.join("zircon_editor-test-exe");
    let deps = bin_dir.join("deps").join(platform_runtime_library_name());
    std::fs::write(&deps, []).unwrap();

    let actual = runtime_library_path_for_executable(&executable);

    remove_runtime_library_temp_dir(&temp);
    assert_eq!(actual, deps);
}

#[test]
fn runtime_api_field_availability_rejects_truncated_or_overflowing_fields() {
    assert!(runtime_api_field_available(16, 8, 8));
    assert!(!runtime_api_field_available(15, 8, 8));
    assert!(!runtime_api_field_available(usize::MAX, usize::MAX, 1));
}

#[test]
fn runtime_api_required_layout_must_cover_operation_harvest_field() {
    let required_size = core::mem::offset_of!(ZrRuntimeApiV2, harvest_operation)
        + core::mem::size_of::<Option<zircon_runtime_interface::ZrRuntimeHarvestOperationFnV1>>();

    assert!(runtime_api_required_layout_available(required_size));
    assert!(!runtime_api_required_layout_available(required_size - 1));
}

#[test]
fn runtime_api_pointer_rejects_null_from_entry_symbol() {
    let error = validate_runtime_api_pointer(core::ptr::null())
        .expect_err("null runtime API pointer should be rejected");

    assert_eq!(
        error.to_string(),
        "runtime library rejected host ABI version"
    );
}

#[test]
fn runtime_api_pointer_rejects_version_mismatch_before_session_creation() {
    let api = valid_runtime_api_table(zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V2 + 1);

    let error = validate_runtime_api_pointer(&api)
        .expect_err("unsupported runtime API table version should be rejected");

    assert_eq!(error.to_string(), "unsupported runtime API table version 3");
}

#[test]
fn runtime_api_pointer_rejects_missing_required_functions_before_session_creation() {
    let mut api = valid_runtime_api_table(zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V2);
    api.create_session = None;

    let error = validate_runtime_api_pointer(&api)
        .expect_err("missing required runtime API functions should be rejected");

    assert_eq!(
        error.to_string(),
        "runtime API table is missing required functions"
    );
}

#[test]
fn runtime_api_pointer_rejects_missing_required_operation_functions() {
    let mut api = valid_runtime_api_table(zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V2);
    api.harvest_operation = None;

    let error = validate_runtime_api_pointer(&api)
        .expect_err("V2 runtime API must provide the operation lifecycle");

    assert_eq!(
        error.to_string(),
        "runtime API table is missing required functions"
    );
}

#[test]
fn runtime_session_does_not_recheck_required_v2_mirror_or_operation_capabilities() {
    let session_source = include_str!("runtime_session.rs");
    let operation_source = include_str!("runtime_session/operation.rs");

    for forbidden in [
        "let Some(subscribe) = self.runtime.subscribe_plugin_event()",
        "let Some(unsubscribe) = self.runtime.unsubscribe_plugin_event()",
        "let Some(drain) = self.runtime.drain_plugin_events()",
        "CapabilityMissing {\n                    capability: \"runtime.operation.submit\"",
        "CapabilityMissing {\n                    capability: \"runtime.operation.poll\"",
        "CapabilityMissing {\n                    capability: \"runtime.operation.harvest\"",
    ] {
        assert!(
            !session_source.contains(forbidden),
            "validated V2 required entry must not retain capability fallback `{forbidden}`"
        );
    }
    for forbidden in [
        "let Some(submit) = self.runtime.submit_operation()",
        "let Some(poll) = self.runtime.poll_operation()",
        "let Some(harvest) = self.runtime.harvest_operation()",
    ] {
        assert!(
            !operation_source.contains(forbidden),
            "validated V2 required entry must not retain capability fallback `{forbidden}`"
        );
    }
}

#[test]
fn runtime_library_loader_reports_missing_entry_symbol_source_path() {
    let source = include_str!("loaded_runtime.rs");

    assert!(source.contains(".get::<ZrRuntimeGetApiFnV2>(ZR_RUNTIME_GET_API_SYMBOL_V2)"));
    assert!(!source.contains("ZrRuntimeGetApiFnV1"));
    assert!(!source.contains("ZR_RUNTIME_GET_API_SYMBOL_V1"));
    assert!(!source.contains("RuntimeApi::V1"));
    assert!(source.contains("failed to resolve zircon runtime API V2 symbol"));
}

#[test]
fn runtime_library_loader_reports_missing_entry_symbol_from_dynamic_library() {
    let result = LoadedRuntime::load(missing_entry_symbol_fixture_library());
    let error = match result {
        Ok(_) => panic!("system library should not export the zircon runtime API symbol"),
        Err(error) => error,
    };
    let message = error.to_string();

    assert!(
        message.contains("failed to resolve zircon runtime API V2 symbol"),
        "{message}"
    );
}

#[test]
fn runtime_session_create_reports_first_call_failure_context() {
    let source = include_str!("runtime_session.rs");
    let create_start = source
        .find("pub(crate) fn create_with_profile_and_project")
        .expect("runtime session create path");
    let create_body = &source[create_start..];

    assert!(create_body.contains("create_session("));
    assert!(create_body.contains("ensure_status(status, \"create runtime session\")?;"));
}

#[test]
fn runtime_surface_present_support_requires_all_optional_fields_in_size() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV2>();
    let before_bind = core::mem::offset_of!(ZrRuntimeApiV2, bind_viewport_surface);
    let before_unbind = core::mem::offset_of!(ZrRuntimeApiV2, unbind_viewport_surface);
    let before_present = core::mem::offset_of!(ZrRuntimeApiV2, present_viewport);
    let bind = Some(fake_bind_viewport_surface as _);
    let unbind = Some(fake_unbind_viewport_surface as _);
    let present = Some(fake_present_viewport as _);

    assert!(runtime_api_supports_viewport_surface_present(
        full_size, bind, unbind, present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        before_bind,
        bind,
        unbind,
        present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        before_unbind,
        bind,
        unbind,
        present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        before_present,
        bind,
        unbind,
        present
    ));
    assert!(!runtime_api_supports_viewport_surface_present(
        full_size, bind, None, present
    ));
}

#[test]
fn runtime_api_profile_control_is_optional_after_present_prefix() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV2>();
    let before_profile = core::mem::offset_of!(ZrRuntimeApiV2, profile_control);
    let api = ZrRuntimeApiV2 {
        profile_control: Some(fake_profile_control as _),
        ..ZrRuntimeApiV2::empty()
    };

    assert!(runtime_api_field_available(
        full_size,
        before_profile,
        core::mem::size_of_val(&api.profile_control)
    ));
    assert!(!runtime_api_field_available(
        before_profile,
        before_profile,
        core::mem::size_of_val(&api.profile_control)
    ));
}

#[test]
fn runtime_api_tick_frame_is_optional_after_profile_control() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV2>();
    let before_tick = core::mem::offset_of!(ZrRuntimeApiV2, tick_frame);
    let api = ZrRuntimeApiV2 {
        tick_frame: Some(fake_tick_frame as _),
        ..ZrRuntimeApiV2::empty()
    };

    assert_eq!(
        before_tick,
        core::mem::offset_of!(ZrRuntimeApiV2, profile_control)
            + core::mem::size_of::<Option<ZrRuntimeProfileControlFnV1>>()
    );
    assert!(runtime_api_field_available(
        full_size,
        before_tick,
        core::mem::size_of::<Option<ZrRuntimeTickFrameFnV1>>()
    ));
    assert!(!runtime_api_field_available(
        before_tick,
        before_tick,
        core::mem::size_of_val(&api.tick_frame)
    ));
}

#[test]
fn runtime_api_drain_host_requests_is_optional_after_tick_frame() {
    let full_size = core::mem::size_of::<ZrRuntimeApiV2>();
    let before_drain = core::mem::offset_of!(ZrRuntimeApiV2, drain_host_requests);
    let api = ZrRuntimeApiV2 {
        drain_host_requests: Some(fake_drain_host_requests as _),
        ..ZrRuntimeApiV2::empty()
    };

    assert_eq!(
        before_drain,
        core::mem::offset_of!(ZrRuntimeApiV2, tick_frame)
            + core::mem::size_of::<Option<ZrRuntimeTickFrameFnV1>>()
    );
    assert!(runtime_api_field_available(
        full_size,
        before_drain,
        core::mem::size_of::<Option<ZrRuntimeDrainHostRequestsFnV1>>()
    ));
    assert!(!runtime_api_field_available(
        before_drain,
        before_drain,
        core::mem::size_of_val(&api.drain_host_requests)
    ));
}

#[test]
fn runtime_operation_api_is_the_v2_table_tail() {
    let api = ZrRuntimeApiV2::empty();
    let full_size = core::mem::size_of::<ZrRuntimeApiV2>();
    for (offset, field_size) in [
        (
            core::mem::offset_of!(ZrRuntimeApiV2, submit_operation),
            core::mem::size_of_val(&api.submit_operation),
        ),
        (
            core::mem::offset_of!(ZrRuntimeApiV2, poll_operation),
            core::mem::size_of_val(&api.poll_operation),
        ),
        (
            core::mem::offset_of!(ZrRuntimeApiV2, harvest_operation),
            core::mem::size_of_val(&api.harvest_operation),
        ),
    ] {
        assert!(runtime_api_field_available(full_size, offset, field_size));
        assert!(!runtime_api_field_available(offset, offset, field_size));
    }
}

#[test]
#[cfg_attr(
    not(feature = "backend-zr-vm"),
    ignore = "requires backend-zr-vm, ZIRCON_RUNTIME_LIBRARY, and ZR_VM_RUST_BINDING_LIB_DIR"
)]
fn runtime_library_project_capture_frame_draws_vampire_hud() {
    let runtime = LoadedRuntime::load_default().unwrap();
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_path_buf();
    let project_root = workspace_root.join("examples").join("vampire");
    let session = super::runtime_session::RuntimeSession::create_with_profile_and_project(
        runtime,
        b"runtime",
        Some(project_root.as_path()),
    )
    .unwrap();

    for _ in 0..3 {
        session.tick_frame().unwrap();
    }

    let frame = session
        .capture_frame(
            ZrRuntimeViewportHandle::new(1),
            ZrRuntimeViewportSizeV1::new(640, 360),
        )
        .unwrap();
    let hud_panel_pixels =
        count_vampire_hud_panel_pixels(frame.rgba(), frame.width(), frame.height());

    assert!(
        hud_panel_pixels > 48,
        "runtime library capture should include the vampire HUD panel, found {hud_panel_pixels}"
    );
}

fn runtime_library_temp_dir(case_name: &str) -> PathBuf {
    let temp = std::env::temp_dir().join(format!(
        "zircon-runtime-library-path-{}-{case_name}",
        std::process::id()
    ));
    remove_runtime_library_temp_dir(&temp);
    temp
}

fn count_vampire_hud_panel_pixels(rgba: &[u8], width: u32, height: u32) -> usize {
    let width = width as usize;
    let height = height as usize;
    let y_start = 16usize.min(height);
    let y_end = 80usize.min(height);
    let x_start = 16usize.min(width);
    let x_end = 260usize.min(width);
    let mut count = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            if pixel[0] <= 48 && pixel[1] <= 58 && pixel[2] <= 76 && pixel[3] >= 180 {
                count += 1;
            }
        }
    }
    count
}

fn remove_runtime_library_temp_dir(path: &std::path::Path) {
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
}

fn missing_entry_symbol_fixture_library() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "kernel32.dll"
    }
    #[cfg(target_os = "macos")]
    {
        "/usr/lib/libSystem.B.dylib"
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        "libc.so.6"
    }
}

fn valid_runtime_api_table(api_version: u32) -> ZrRuntimeApiV2 {
    let mut api = ZrRuntimeApiV2::empty();
    api.abi_version = api_version;
    api.size_bytes = core::mem::size_of::<ZrRuntimeApiV2>();
    api.create_session = Some(fake_create_session);
    api.destroy_session = Some(fake_destroy_session);
    api.handle_event = Some(fake_handle_event);
    api.capture_frame = Some(fake_capture_frame);
    api.subscribe_plugin_event = Some(fake_subscribe_plugin_event);
    api.unsubscribe_plugin_event = Some(fake_unsubscribe_plugin_event);
    api.drain_plugin_events = Some(fake_drain_plugin_events);
    api.submit_operation = Some(fake_submit_operation);
    api.poll_operation = Some(fake_poll_operation);
    api.harvest_operation = Some(fake_harvest_operation);
    api
}

unsafe extern "C" fn fake_submit_operation(
    _session: ZrRuntimeSessionHandle,
    _request: ZrByteSlice,
    _out_operation: *mut ZrRuntimeOperationHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_poll_operation(
    _session: ZrRuntimeSessionHandle,
    _operation: ZrRuntimeOperationHandle,
    _out_progress: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_harvest_operation(
    _session: ZrRuntimeSessionHandle,
    _operation: ZrRuntimeOperationHandle,
    _out_result: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_create_session(
    _config: ZrRuntimeSessionConfigV1,
    _out_session: *mut ZrRuntimeSessionHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_destroy_session(_session: ZrRuntimeSessionHandle) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_handle_event(
    _session: ZrRuntimeSessionHandle,
    _event: ZrRuntimeEventV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_capture_frame(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeFrameRequestV1,
    _out_frame: *mut ZrRuntimeFrameV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_bind_viewport_surface(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeBindViewportSurfaceRequestV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_unbind_viewport_surface(
    _session: ZrRuntimeSessionHandle,
    _viewport: ZrRuntimeViewportHandle,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_present_viewport(
    _session: ZrRuntimeSessionHandle,
    _request: ZrRuntimeFrameRequestV1,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_profile_control(
    _session: ZrRuntimeSessionHandle,
    _request_json: ZrByteSlice,
    _out_json: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_tick_frame(_session: ZrRuntimeSessionHandle) -> ZrStatus {
    ZrStatus::ok()
}

unsafe extern "C" fn fake_drain_host_requests(
    _session: ZrRuntimeSessionHandle,
    _out_requests: *mut ZrOwnedByteBuffer,
) -> ZrStatus {
    ZrStatus::ok()
}
