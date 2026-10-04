use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::super::RuntimeLibraryError;
use super::{
    runtime_library_environment_override_request, runtime_library_override_path_from_executable,
    runtime_library_override_path_from_value, RuntimeLibraryPathError, RuntimeLibraryPathSelection,
};

#[test]
fn runtime_library_override_path_rejects_blank_unicode_value() {
    let error = runtime_library_override_path_from_value(Some(OsString::from("\u{2003}")))
        .expect_err("blank runtime override must not be treated as a loadable path");

    assert_eq!(
        error.to_string(),
        "runtime startup diagnostic: component=runtime_library requested_path=<environment override> cause=ZIRCON_RUNTIME_LIBRARY is blank recovery=unset ZIRCON_RUNTIME_LIBRARY or set it to a compatible product-relative or absolute path"
    );
}

#[test]
fn runtime_library_override_path_preserves_empty_fallback_and_absolute_paths() {
    let absolute = std::env::temp_dir().join("zircon-runtime-library-test.dll");

    assert!(
        runtime_library_override_path_from_value(Some(OsString::new()))
            .unwrap()
            .is_none()
    );
    let selected =
        runtime_library_override_path_from_value(Some(absolute.clone().into_os_string()))
            .unwrap()
            .expect("an absolute override must select a runtime library");
    assert_eq!(selected.path(), absolute);
}

#[test]
fn runtime_library_override_selection_keeps_a_relative_request_with_its_operation_path() {
    let relative = Path::new("plugins/zircon_runtime.dll");
    let selected = runtime_library_override_path_from_value(Some(OsString::from(
        "plugins/zircon_runtime.dll",
    )))
    .unwrap()
    .expect("a normal relative override must select a runtime library");

    assert_eq!(
        selected.environment_override_request(),
        Some("ZIRCON_RUNTIME_LIBRARY=plugins/zircon_runtime.dll")
    );

    let executable = std::env::current_exe().unwrap();
    let product_directory = zircon_runtime::asset::project::ProjectPaths::resolve_path(
        executable
            .parent()
            .expect("test executable must have a product directory"),
    )
    .unwrap();
    let expected = zircon_runtime::asset::project::ProjectPaths::resolve_path_from(
        &product_directory,
        relative,
    )
    .unwrap()
    .into_operation_path();
    assert_eq!(selected.path(), expected);
}

#[test]
fn runtime_library_override_path_resolves_relative_value_from_product_directory() {
    let executable = std::env::temp_dir()
        .join("zircon-runtime-library-distribution")
        .join("zircon_runtime.exe");

    let actual = runtime_library_override_path_from_executable(
        Path::new("plugins/zircon_runtime.dll"),
        &executable,
    )
    .expect("a normal relative override should resolve from the product directory");

    let expected = zircon_runtime::asset::project::ProjectPaths::resolve_path(
        executable
            .parent()
            .expect("test executable must have a distribution directory")
            .join("plugins/zircon_runtime.dll"),
    )
    .unwrap()
    .into_operation_path();
    assert_eq!(actual, expected);
}

#[cfg(windows)]
#[test]
fn runtime_library_override_path_preserves_windows_absolute_path_semantics() {
    for absolute in [
        PathBuf::from(r"C:\zircon\zircon_runtime.dll"),
        PathBuf::from(r"\\server\share\zircon_runtime.dll"),
    ] {
        let selected =
            runtime_library_override_path_from_value(Some(absolute.clone().into_os_string()))
                .unwrap()
                .expect("an absolute Windows override must select a runtime library");
        assert_eq!(selected.path(), absolute);
    }

    let executable = Path::new(r"C:\zircon\zircon_runtime.exe");
    for relative in [
        PathBuf::from(r"C:zircon_runtime.dll"),
        PathBuf::from(r"\zircon_runtime.dll"),
        PathBuf::from(r"/zircon_runtime.dll"),
    ] {
        let error = runtime_library_override_path_from_executable(&relative, executable)
            .expect_err("Windows drive-relative and rooted paths must remain unsupported");

        assert!(
            error
                .to_string()
                .contains(&runtime_library_environment_override_request(&relative)),
            "relative path diagnostic must retain the original requested value: {error}"
        );
    }
}

#[cfg(unix)]
#[test]
fn runtime_library_override_path_preserves_non_utf8_value() {
    use std::os::unix::ffi::OsStringExt;

    let value = OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xFF]);

    let selected = runtime_library_override_path_from_value(Some(value.clone()))
        .unwrap()
        .expect("an absolute non-UTF-8 override must select a runtime library");
    assert_eq!(selected.path(), PathBuf::from(value));
}

#[test]
fn runtime_library_path_error_preserves_override_diagnostic() {
    let error = RuntimeLibraryPathError::EnvironmentOverride(RuntimeLibraryError::new(
        "environment override diagnostic",
    ));

    assert_eq!(error.to_string(), "environment override diagnostic");
}

#[test]
fn runtime_library_override_selection_keeps_its_provenance_and_request_label() {
    let path = std::env::temp_dir()
        .join("zircon-runtime-library-product")
        .join("plugins")
        .join("custom.dll");
    let request = runtime_library_environment_override_request(Path::new("plugins/custom.dll"));
    let selection = RuntimeLibraryPathSelection::EnvironmentOverride {
        path: path.clone(),
        request: request.clone(),
    };

    assert_eq!(selection.path(), path);
    assert_eq!(
        selection.environment_override_request(),
        Some(request.as_str())
    );
}
