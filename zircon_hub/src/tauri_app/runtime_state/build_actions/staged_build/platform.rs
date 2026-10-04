/// Returns the staged Editor artifact name for the current host platform.
pub(super) fn editor_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "zircon_editor.exe"
    } else {
        "zircon_editor"
    }
}

/// Returns the staged standalone runtime artifact name for the current host platform.
pub(super) fn runtime_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "zircon_runtime.exe"
    } else {
        "zircon_runtime"
    }
}

/// Returns the staged dynamic runtime library name for the current host platform.
pub(super) fn runtime_library_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "zircon_runtime.dll"
    } else if cfg!(target_os = "macos") {
        "libzircon_runtime.dylib"
    } else {
        "libzircon_runtime.so"
    }
}
