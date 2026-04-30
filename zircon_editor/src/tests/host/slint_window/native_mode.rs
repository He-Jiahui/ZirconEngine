fn source(relative: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative))
        .unwrap_or_else(|error| panic!("read `{relative}`: {error}"))
}

#[test]
fn native_floating_window_mode_uses_rust_owned_host_window_contract() {
    let native_windows = source("src/ui/slint_host/app/native_windows.rs");
    let window = source("src/ui/slint_host/host_contract/window.rs");
    let host_components = source("src/ui/slint_host/host_contract/data/host_components.rs");

    for required in [
        "configure_native_floating_window_presentation",
        "native_floating_window_mode = true",
        "native_floating_window_id",
        "native_window_title",
        "native_window_bounds",
        "UiHostWindow::new()",
        "UiHostWindow::clone_strong",
    ] {
        assert!(native_windows.contains(required), "native window path missing `{required}`");
    }
    for required in ["set_size", "is_maximized", "set_maximized", "get_host_window_bootstrap"] {
        assert!(window.contains(required), "UiHostWindow contract missing `{required}`");
    }
    assert!(host_components.contains("pub(crate) struct HostNativeFloatingWindowSurfaceData"));
}
