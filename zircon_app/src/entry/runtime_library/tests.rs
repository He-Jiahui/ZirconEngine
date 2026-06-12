use std::path::PathBuf;

use super::library_path::{
    default_runtime_library_path, platform_runtime_library_name,
    runtime_library_path_for_executable,
};
use super::loaded_runtime::{
    runtime_api_field_available, runtime_api_required_prefix_available,
    runtime_api_supports_viewport_surface_present, validate_runtime_api_pointer, LoadedRuntime,
};
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeCaptureFrameFnV1, ZrRuntimeDrainHostRequestsFnV1, ZrRuntimeProfileControlFnV1,
    ZrRuntimeTickFrameFnV1,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV1, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeFrameV1, ZrRuntimeSessionConfigV1,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZrStatus,
};

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
fn runtime_api_required_prefix_must_cover_required_capture_field() {
    let required_size = core::mem::offset_of!(ZrRuntimeApiV1, capture_frame)
        + core::mem::size_of::<Option<ZrRuntimeCaptureFrameFnV1>>();

    assert!(runtime_api_required_prefix_available(required_size));
    assert!(!runtime_api_required_prefix_available(required_size - 1));
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
    let api = valid_runtime_api_table(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1 + 1);

    let error = validate_runtime_api_pointer(&api)
        .expect_err("unsupported runtime ABI version should be rejected");

    assert_eq!(error.to_string(), "unsupported runtime ABI version 2");
}

#[test]
fn runtime_api_pointer_rejects_missing_required_functions_before_session_creation() {
    let mut api = valid_runtime_api_table(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1);
    api.create_session = None;

    let error = validate_runtime_api_pointer(&api)
        .expect_err("missing required runtime API functions should be rejected");

    assert_eq!(
        error.to_string(),
        "runtime API table is missing required functions"
    );
}

#[test]
fn runtime_library_loader_reports_missing_entry_symbol_source_path() {
    let source = include_str!("loaded_runtime.rs");

    assert!(source.contains(".get::<ZrRuntimeGetApiFnV1>(ZR_RUNTIME_GET_API_SYMBOL_V1)"));
    assert!(source.contains("failed to resolve zircon runtime API symbol"));
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
        message.contains("failed to resolve zircon runtime API symbol"),
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
    let full_size = core::mem::size_of::<ZrRuntimeApiV1>();
    let before_bind = core::mem::offset_of!(ZrRuntimeApiV1, bind_viewport_surface);
    let before_unbind = core::mem::offset_of!(ZrRuntimeApiV1, unbind_viewport_surface);
    let before_present = core::mem::offset_of!(ZrRuntimeApiV1, present_viewport);
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
    let full_size = core::mem::size_of::<ZrRuntimeApiV1>();
    let before_profile = core::mem::offset_of!(ZrRuntimeApiV1, profile_control);
    let api = ZrRuntimeApiV1 {
        profile_control: Some(fake_profile_control as _),
        ..ZrRuntimeApiV1::empty(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1)
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
    let full_size = core::mem::size_of::<ZrRuntimeApiV1>();
    let before_tick = core::mem::offset_of!(ZrRuntimeApiV1, tick_frame);
    let api = ZrRuntimeApiV1 {
        tick_frame: Some(fake_tick_frame as _),
        ..ZrRuntimeApiV1::empty(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1)
    };

    assert_eq!(
        before_tick,
        core::mem::offset_of!(ZrRuntimeApiV1, profile_control)
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
    let full_size = core::mem::size_of::<ZrRuntimeApiV1>();
    let before_drain = core::mem::offset_of!(ZrRuntimeApiV1, drain_host_requests);
    let api = ZrRuntimeApiV1 {
        drain_host_requests: Some(fake_drain_host_requests as _),
        ..ZrRuntimeApiV1::empty(zircon_runtime_interface::ZIRCON_RUNTIME_ABI_VERSION_V1)
    };

    assert_eq!(
        before_drain,
        core::mem::offset_of!(ZrRuntimeApiV1, tick_frame)
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

fn valid_runtime_api_table(abi_version: u32) -> ZrRuntimeApiV1 {
    let mut api = ZrRuntimeApiV1::empty(abi_version);
    api.size_bytes = core::mem::size_of::<ZrRuntimeApiV1>();
    api.create_session = Some(fake_create_session);
    api.destroy_session = Some(fake_destroy_session);
    api.handle_event = Some(fake_handle_event);
    api.capture_frame = Some(fake_capture_frame);
    api
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
