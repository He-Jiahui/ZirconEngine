use std::path::PathBuf;

use super::library_path::{
    default_runtime_library_path, platform_runtime_library_name,
    runtime_library_path_for_executable,
};
use super::loaded_runtime::{
    runtime_api_field_available, runtime_api_supports_viewport_surface_present,
};
use zircon_runtime_interface::{
    ZrRuntimeApiV1, ZrRuntimeBindViewportSurfaceRequestV1, ZrRuntimeFrameRequestV1,
    ZrRuntimeSessionHandle, ZrRuntimeViewportHandle, ZrStatus, ZIRCON_RUNTIME_ABI_VERSION_V1,
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
fn runtime_surface_present_support_requires_all_optional_fields_in_size() {
    let mut api = ZrRuntimeApiV1::empty(ZIRCON_RUNTIME_ABI_VERSION_V1);
    api.bind_viewport_surface = Some(fake_bind_viewport_surface);
    api.unbind_viewport_surface = Some(fake_unbind_viewport_surface);
    api.present_viewport = Some(fake_present_viewport);

    assert!(runtime_api_supports_viewport_surface_present(&api));

    api.size_bytes = core::mem::offset_of!(ZrRuntimeApiV1, present_viewport);
    assert!(!runtime_api_supports_viewport_surface_present(&api));

    api.size_bytes = core::mem::size_of::<ZrRuntimeApiV1>();
    api.unbind_viewport_surface = None;
    assert!(!runtime_api_supports_viewport_surface_present(&api));
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
