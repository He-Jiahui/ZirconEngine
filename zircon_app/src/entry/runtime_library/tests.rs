use std::path::PathBuf;

use super::library_path::{
    default_runtime_library_path, platform_runtime_library_name,
    runtime_library_path_for_executable,
};
use super::loaded_runtime::{
    runtime_api_field_available, runtime_api_required_prefix_available,
    runtime_api_supports_viewport_surface_present,
};
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeCaptureFrameFnV1, ZrRuntimeDrainHostRequestsFnV1, ZrRuntimeProfileControlFnV1,
    ZrRuntimeTickFrameFnV1,
};
use zircon_runtime_interface::{
    ZrByteSlice, ZrOwnedByteBuffer, ZrRuntimeApiV1, ZrRuntimeBindViewportSurfaceRequestV1,
    ZrRuntimeFrameRequestV1, ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrStatus,
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

fn runtime_library_temp_dir(case_name: &str) -> PathBuf {
    let temp = std::env::temp_dir().join(format!(
        "zircon-runtime-library-path-{}-{case_name}",
        std::process::id()
    ));
    remove_runtime_library_temp_dir(&temp);
    temp
}

fn remove_runtime_library_temp_dir(path: &std::path::Path) {
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
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
