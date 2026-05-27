use super::sources::{
    entry_root, runtime_app_source, runtime_application_handler_source,
    runtime_surface_present_source, runtime_window_surface_source,
};

#[test]
fn runtime_surface_present_sources_stay_folder_backed() {
    let runtime_app_source = runtime_app_source();
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_surface_present_source = runtime_surface_present_source();
    let runtime_window_surface_source = runtime_window_surface_source();
    let root = entry_root();

    assert!(
        runtime_app_source.contains("surface_present_failed: bool"),
        "runtime entry app should remember failed surface-present state"
    );
    assert!(
        runtime_app_source.contains("mod surface_present;"),
        "runtime entry app should keep native surface-present binding and fallback in a child module"
    );
    assert!(
        runtime_app_source.contains("mod window_surface;"),
        "runtime entry app should keep native window-surface target extraction in a child module"
    );
    assert!(
        runtime_surface_present_source.contains("mod binding;")
            && runtime_surface_present_source.contains("mod fallback;")
            && runtime_surface_present_source.contains("mod lifecycle;")
            && runtime_surface_present_source.contains("mod redraw;")
            && runtime_surface_present_source.contains("mod resize;"),
        "runtime surface-present root should stay structural and declare focused helper families"
    );
    assert!(
        runtime_window_surface_source.contains("mod native_target;")
            && runtime_window_surface_source.contains(
                "pub(in crate::entry::runtime_entry_app) use native_target::runtime_native_surface_target;",
            ),
        "runtime window-surface root should stay structural and expose the native target helper"
    );
    assert!(
        runtime_app_source.contains("mod window_creation;"),
        "runtime entry app should keep primary winit window creation in a child module"
    );
    assert!(
        runtime_app_source.contains("mod window_events;"),
        "runtime entry app should keep concrete winit window-event dispatch in a child module"
    );
    assert!(
        !runtime_handler_source.contains("fn bind_window_surface"),
        "runtime winit event handling should not own native surface binding helper implementations"
    );
    assert!(
        !root.join("tests/runtime_entry_surface_present_guards.rs")
            .exists(),
        "runtime surface-present guards should stay folder-backed instead of returning to an umbrella runtime_entry_surface_present_guards.rs file"
    );
    for relative in [
        "runtime_entry_app/surface_present.rs",
        "runtime_entry_app/window_surface.rs",
    ] {
        assert!(
            !root.join(relative).exists(),
            "runtime entry surface-present ownership should stay folder-backed instead of returning to `{relative}`"
        );
    }
}

#[test]
fn runtime_window_surface_target_extraction_stays_app_side_and_wgpu_free() {
    let runtime_window_surface_source = runtime_window_surface_source();

    for native_surface_path in [
        "HasWindowHandle",
        "HasDisplayHandle",
        "RawWindowHandle::Win32",
        "RawDisplayHandle::Windows",
        "ZrRuntimeNativeSurfaceTargetV1::win32",
        "ZIRCON_RUNTIME_ABI_VERSION_V1",
    ] {
        assert!(
            runtime_window_surface_source.contains(native_surface_path),
            "runtime window-surface native target helper should preserve `{native_surface_path}`"
        );
    }
    assert!(
        !runtime_window_surface_source.contains("wgpu::"),
        "runtime window-surface target extraction should not create or configure render surfaces directly"
    );
}
