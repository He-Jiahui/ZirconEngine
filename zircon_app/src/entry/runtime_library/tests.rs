use std::path::PathBuf;

use super::library_path::{
    default_runtime_library_path, platform_runtime_library_name,
    runtime_library_path_for_executable,
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
